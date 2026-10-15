//! A cart's label scans cut once to the size its face shows them at, and kept on the card beside
//! them. The scans are kept big enough for a cart drawn large, a few hundred kilobytes each, and
//! decoding hundreds of them held a boot on the boot logo long enough to look frozen; the cuts
//! are small and a face drawn from them is ready at once.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use omega_store::Cart;

use crate::art;
use crate::cart::label_scans;

/// The folder beside a system's label scans that their cuts are kept in.
pub const CUT_DIR: &str = ".cut";

/// Where the cut of the label scan at `scan` is kept.
pub fn cut_path(scan: &Path) -> Option<PathBuf> {
    Some(scan.parent()?.join(CUT_DIR).join(scan.file_name()?))
}

fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// The cut kept for `scan`, when it is `w` by `h` and was cut from the scan there now: a cut
/// carries its scan's time, so a scan put in its place, older or newer, has it cut again.
fn kept(scan: &Path, w: u32, h: u32) -> Option<PathBuf> {
    let cut = cut_path(scan)?;
    let current = modified(&cut).is_some_and(|t| Some(t) == modified(scan));
    (current && art::size(&cut) == Some((w, h))).then_some(cut)
}

/// The scan at `scan` as drawn at `w` by `h`: from its cut when one is kept, from the scan itself
/// when not.
pub(crate) fn label_art(scan: &Path, w: u32, h: u32) -> Option<Vec<u8>> {
    kept(scan, w, h)
        .and_then(|cut| art::exact(&cut, w, h))
        .or_else(|| art::cover(scan, w, h))
}

/// Whether `cart` has a label scan with no cut of it for its face, so drawing the face would
/// decode the whole scan.
pub fn label_uncut(cart: &Cart) -> bool {
    label_scans(cart)
        .iter()
        .any(|(scan, w, h)| kept(scan, *w, *h).is_none())
}

/// Cuts each of `cart`'s label scans that has no cut for its face and keeps the cuts beside them;
/// true when its face now has a cut to be drawn from.
pub fn cut_label(cart: &Cart) -> bool {
    let mut any = false;
    for (scan, w, h) in label_scans(cart) {
        if kept(&scan, w, h).is_some() || cut(&scan, w, h) {
            any = true;
        }
    }
    any
}

/// Cuts the scan at `scan` to `w` by `h` and keeps the cut, with the scan's time; false when the
/// scan does not decode or the cut could not be kept.
fn cut(scan: &Path, w: u32, h: u32) -> bool {
    let (Some(cut), Some(scanned)) = (cut_path(scan), modified(scan)) else {
        return false;
    };
    let Some(png) = art::cover(scan, w, h).and_then(|rgba| art::encode(&rgba, w, h)) else {
        return false;
    };
    let kept = cut
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .and_then(|()| omega_store::atomic_write(&cut, &png))
        .and_then(|()| {
            std::fs::File::options()
                .write(true)
                .open(&cut)?
                .set_modified(scanned)
        });
    match kept {
        Ok(()) => true,
        Err(e) => {
            eprintln!("omega-ui: label cut: {}: {e}", cut.display());
            false
        }
    }
}
