//! One ordered state-file writer. Navigation never waits for SD-card fsync; durable actions
//! use the same queue and wait for acknowledgment, so an old hover cannot overwrite them.
use omega_store::{write_app_state, AppState};
use std::{
    io,
    path::PathBuf,
    sync::mpsc::{self, Sender},
    thread::JoinHandle,
};
type Ack = Sender<io::Result<()>>;
enum Msg {
    Write(AppState, Option<Ack>),
    Stop,
}
pub(crate) struct StateWriter {
    tx: Sender<Msg>,
    join: Option<JoinHandle<()>>,
}
impl StateWriter {
    #[cfg_attr(not(feature = "device"), allow(dead_code))]
    pub fn new(root: PathBuf) -> io::Result<Self> {
        Self::with_sink(move |s| write_app_state(&root, s))
    }
    fn with_sink(
        mut sink: impl FnMut(&AppState) -> io::Result<()> + Send + 'static,
    ) -> io::Result<Self> {
        let (tx, rx) = mpsc::channel();
        let join = std::thread::Builder::new()
            .name("omega-state".into())
            .spawn(move || {
                while let Ok(msg) = rx.recv() {
                    let Msg::Write(mut state, ack) = msg else {
                        break;
                    };
                    let mut acks: Vec<_> = ack.into_iter().collect();
                    let mut stop = false;
                    while let Ok(next) = rx.try_recv() {
                        match next {
                            Msg::Write(new, ack) => {
                                state = new;
                                acks.extend(ack);
                            }
                            Msg::Stop => {
                                stop = true;
                                break;
                            }
                        }
                    }
                    let result = sink(&state);
                    if let Err(e) = &result {
                        eprintln!("omega-zero: state writer: {e}");
                    }
                    for ack in acks {
                        let _ = ack.send(match &result {
                            Ok(()) => Ok(()),
                            Err(e) => Err(io::Error::new(e.kind(), e.to_string())),
                        });
                    }
                    if stop {
                        break;
                    }
                }
            })?;
        Ok(Self {
            tx,
            join: Some(join),
        })
    }
    pub fn hover(&self, state: AppState) {
        let _ = self.tx.send(Msg::Write(state, None));
    }
    pub fn durable(&self, state: AppState) -> io::Result<()> {
        let (tx, rx) = mpsc::channel();
        self.tx
            .send(Msg::Write(state, Some(tx)))
            .map_err(|_| io::Error::other("state writer stopped"))?;
        rx.recv()
            .map_err(|_| io::Error::other("state writer stopped"))?
    }
}
impl Drop for StateWriter {
    fn drop(&mut self) {
        let _ = self.tx.send(Msg::Stop);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    #[test]
    fn slow_sd_writes_do_not_block_hover_and_durable_fences_keep_the_latest_state() {
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let writes = Arc::new(Mutex::new(Vec::new()));
        let log = writes.clone();
        let writer = StateWriter::with_sink(move |s| {
            if s.brightness == 1 {
                entered_tx.send(()).unwrap();
                release_rx.recv().unwrap();
            }
            log.lock().unwrap().push(s.brightness);
            Ok(())
        })
        .unwrap();
        let state = |b| AppState {
            brightness: b,
            ..Default::default()
        };
        writer.hover(state(1));
        entered_rx
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap();
        writer.hover(state(2));
        writer.hover(state(3));
        let (tx, rx) = mpsc::channel();
        writer.tx.send(Msg::Write(state(4), Some(tx))).unwrap();
        release_tx.send(()).unwrap();
        rx.recv_timeout(std::time::Duration::from_secs(2))
            .unwrap()
            .unwrap();
        assert_eq!(*writes.lock().unwrap(), vec![1, 4]);
        writer.durable(state(5)).unwrap();
        assert_eq!(writes.lock().unwrap().last(), Some(&5));
    }
}
