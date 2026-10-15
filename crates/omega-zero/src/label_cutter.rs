//! The threads that cut label scans to their carts' faces, once each: at boot for the scans
//! with no cut yet, and for each label a sync saves. The shelf comes up without them, those
//! carts in generated labels, and each one is drawn again as its cut is kept. Each scan is then
//! shrunk to what its faces need, and a boot hands over the carts already cut as well, for the
//! scans kept before scans were shrunk.

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use omega_store::{Cart, Platform};
use omega_ui::{cut_label, shrink_label};

/// Cutting a scan is mostly decoding it, on one core; the rest are left to the shelf.
const MAX_WORKERS: usize = 3;

enum Job {
    /// Cut the cart's scans, say so, then shrink them.
    Cut(Cart),
    /// Shrink the scans of a cart whose face is drawn from its cuts already.
    Shrink(Cart),
}

pub struct LabelCutter {
    requests: Sender<Job>,
    cut: Receiver<(Platform, String)>,
}

impl LabelCutter {
    pub fn spawn() -> Self {
        let (requests, inbox) = mpsc::channel::<Job>();
        let (outbox, cut) = mpsc::channel();
        let inbox = Arc::new(Mutex::new(inbox));
        let workers = thread::available_parallelism()
            .map_or(1, |n| n.get().saturating_sub(1))
            .clamp(1, MAX_WORKERS);
        for i in 0..workers {
            let inbox = Arc::clone(&inbox);
            let outbox = outbox.clone();
            let spawned = thread::Builder::new()
                .name(format!("omega-cut-{i}"))
                .spawn(move || loop {
                    let next = inbox.lock().map(|rx| rx.recv());
                    let Ok(Ok(job)) = next else {
                        return;
                    };
                    match job {
                        Job::Cut(cart) => {
                            let cut = cut_label(&cart);
                            if cut && outbox.send((cart.platform, cart.stem.clone())).is_err() {
                                return;
                            }
                            shrink_label(&cart);
                        }
                        Job::Shrink(cart) => shrink_label(&cart),
                    }
                });
            if let Err(e) = spawned {
                eprintln!("omega-zero: labels: cutter thread failed to start: {e}");
            }
        }
        LabelCutter { requests, cut }
    }

    /// Cuts `cart`'s label scan, after the carts asked for before it, and then shrinks it.
    pub fn request(&self, cart: Cart) {
        let _ = self.requests.send(Job::Cut(cart));
    }

    /// Shrinks `cart`'s label scans, after the carts asked for before it, its face not drawn
    /// again.
    pub fn shrink(&self, cart: Cart) {
        let _ = self.requests.send(Job::Shrink(cart));
    }

    /// The carts whose labels were cut since the last time this was asked.
    pub fn take(&self) -> Vec<(Platform, String)> {
        self.cut.try_iter().collect()
    }
}
