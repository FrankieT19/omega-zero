use std::ops::Deref;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

/// The emulator thread's pictures on their way to the screen: the newest one waiting, and the
/// buffers already drawn to be written again. A picture carries its own size, since a
/// PlayStation game changes resolution as it plays.
pub struct Frames {
    inner: Mutex<Inner>,
    size: usize,
    taken: AtomicU64,
}

struct Inner {
    ready: Option<(Vec<u8>, (u32, u32))>,
    spare: Vec<Vec<u8>>,
    allocated: usize,
}

impl Frames {
    /// Room for pictures of `size` bytes, what a buffer is first made with.
    pub fn new(size: usize) -> Arc<Self> {
        Arc::new(Frames {
            inner: Mutex::new(Inner {
                ready: None,
                spare: Vec::new(),
                allocated: 0,
            }),
            size,
            taken: AtomicU64::new(0),
        })
    }

    pub fn take_write(&self) -> Vec<u8> {
        let mut i = self.lock();
        match i.spare.pop() {
            Some(buf) => buf,
            None => {
                i.allocated += 1;
                Vec::with_capacity(self.size)
            }
        }
    }

    /// Hands on `buf`, a picture `size` pixels wide and high, XRGB8888.
    pub fn publish(&self, buf: Vec<u8>, size: (u32, u32)) {
        let mut i = self.lock();
        if let Some((dropped, _)) = i.ready.replace((buf, size)) {
            i.spare.push(dropped);
        }
    }

    pub fn latest(self: &Arc<Self>) -> Option<FrameRef> {
        let (buf, size) = self.lock().ready.take()?;
        self.taken.fetch_add(1, Ordering::Relaxed);
        Some(FrameRef {
            frames: self.clone(),
            buf,
            size,
        })
    }

    pub fn is_ready(&self) -> bool {
        self.lock().ready.is_some()
    }

    pub fn taken(&self) -> u64 {
        self.taken.load(Ordering::Relaxed)
    }

    pub fn allocated(&self) -> usize {
        self.lock().allocated
    }

    fn recycle(&self, buf: Vec<u8>) {
        self.lock().spare.push(buf);
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// A picture taken from `Frames`, its buffer going back for another when it is dropped. It reads
/// as its pixels, XRGB8888 in rows `size().0` long.
pub struct FrameRef {
    frames: Arc<Frames>,
    buf: Vec<u8>,
    size: (u32, u32),
}

impl FrameRef {
    /// The picture's width and height.
    pub fn size(&self) -> (u32, u32) {
        self.size
    }
}

impl Deref for FrameRef {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.buf
    }
}

impl Drop for FrameRef {
    fn drop(&mut self) {
        self.frames.recycle(std::mem::take(&mut self.buf));
    }
}

/// Centre-sampled nearest reduction before display effects. The emulator still renders normally.
/// Reuses the destination allocation and supports PS1 resolution changes between frames.
pub fn gba_raster(frame: &[u8], size: (u32, u32), output: &mut Vec<u8>) {
    let (w, h) = (size.0 as usize, size.1 as usize);
    output.resize(240 * 160 * 4, 0);
    if w == 0 || h == 0 || frame.len() < w * h * 4 {
        return;
    }
    for y in 0..160 {
        let sy = ((2 * y + 1) * h / 320).min(h - 1);
        for x in 0..240 {
            let sx = ((2 * x + 1) * w / 480).min(w - 1);
            let from = (sy * w + sx) * 4;
            let to = (y * 240 + x) * 4;
            output[to..to + 4].copy_from_slice(&frame[from..from + 4]);
        }
    }
}
#[cfg(test)]
mod raster_tests {
    #[test]
    fn reduction_preserves_orientation_channels_and_changes_in_source_resolution() {
        let mut output = Vec::new();
        for (w, h) in [(320, 240), (640, 480), (256, 224)] {
            let source: Vec<u8> = (0..h)
                .flat_map(|y| (0..w).flat_map(move |x| [x as u8, y as u8, 70, 0]))
                .collect();
            super::gba_raster(&source, (w, h), &mut output);
            assert_eq!(output.len(), 240 * 160 * 4);
            for (x, y) in [(0, 0), (119, 79), (239, 159)] {
                let sx = (2 * x + 1) * w as usize / 480;
                let sy = (2 * y + 1) * h as usize / 320;
                assert_eq!(
                    &output[(y * 240 + x) * 4..(y * 240 + x) * 4 + 4],
                    &[sx as u8, sy as u8, 70, 0]
                );
            }
        }
    }
}
