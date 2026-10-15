#![cfg(target_os = "macos")]

//! Label scans are cut to their faces once, off the main thread: a boot that finds scans with no
//! cut comes up without decoding them and draws those carts again as their cuts are kept, the
//! next boot draws them from the cuts at once, and a label a sync saves is cut the same way.

use crate::common;

use std::collections::VecDeque;
use std::path::Path;
use std::time::{Duration, Instant};

use common::{clocked, tmp_root_with_carts};
use omega_gfx::{Compositor, HeadlessSurface};
use omega_input::{InputSource, Millis, RawEvent};
use omega_power::SimPlatform;
use omega_store::Platform;
use omega_ui::label_uncut;
use omega_zero::frontend::Frontend;
use omega_zero::label_sync::{LabelJob, LabelJobs, LabelNews};

/// A colour no generated label and no backdrop is drawn in.
const MAGENTA: [u8; 3] = [0xff, 0x00, 0xff];

struct Idle;

impl InputSource for Idle {
    fn poll(&mut self, _now: Millis) -> Vec<RawEvent> {
        Vec::new()
    }
}

/// A ScreenScraper that has just saved the labels it is given.
struct Saved(VecDeque<LabelNews>);

impl LabelJobs for Saved {
    fn present(&self) -> bool {
        true
    }
    fn ask(&mut self, _job: LabelJob) {}
    fn news(&mut self) -> Vec<LabelNews> {
        self.0.drain(..).collect()
    }
}

fn write_scan(root: &Path, stem: &str) {
    let (w, h) = (600, 310);
    let rgb: Vec<u8> = (0..w * h).flat_map(|_| MAGENTA).collect();
    let dir = root.join("Labels/GBA");
    std::fs::create_dir_all(&dir).expect("labels dir");
    let f = std::fs::File::create(dir.join(format!("{stem}.png"))).expect("create scan");
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), w, h);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()
        .expect("png header")
        .write_image_data(&rgb)
        .expect("png data");
}

/// How many of the frame's pixels are the label's magenta, lit or shaded.
fn magenta_px(frame: &[u8]) -> usize {
    frame
        .chunks_exact(4)
        .filter(|p| p[0] > 110 && p[2] > 110 && p[1] < 40 && p[0].abs_diff(p[2]) < 40)
        .count()
}

/// Composes frames until `done`, for up to ten seconds; false if it never was.
fn compose_until(
    f: &mut Frontend,
    c: &mut Compositor,
    mut done: impl FnMut(&mut Frontend, &[u8]) -> bool,
) -> bool {
    let began = Instant::now();
    while began.elapsed() < Duration::from_secs(10) {
        f.advance(&mut Idle);
        f.app_mut().finish_slide();
        f.compose(c);
        if done(f, &c.read_frame()) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    false
}

#[test]
fn a_boot_cuts_the_scans_it_finds_uncut_and_the_next_boot_draws_from_the_cuts() {
    let Ok(surface) = HeadlessSurface::new() else {
        return;
    };
    let Ok(mut c) = Compositor::new(&surface) else {
        return;
    };
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    clocked(d.path());
    write_scan(d.path(), "Emerald");
    let cut = d.path().join("Labels/GBA/.cut/Emerald.png");

    let mut f = Frontend::boot(Box::new(SimPlatform::at(d.path().to_path_buf())));
    f.upload_faces(&mut c);
    assert!(
        compose_until(&mut f, &mut c, |_, px| cut.is_file()
            && magenta_px(px) > 500),
        "the scan was never cut and its cart never drawn in it"
    );
    drop(f);

    let made = std::fs::metadata(&cut).and_then(|m| m.modified()).unwrap();
    let mut f = Frontend::boot(Box::new(SimPlatform::at(d.path().to_path_buf())));
    assert!(
        f.app().carts().all(|c| !label_uncut(c)),
        "the second boot found a scan to cut"
    );
    f.upload_faces(&mut c);
    assert!(
        compose_until(&mut f, &mut c, |_, px| magenta_px(px) > 500),
        "the second boot did not draw the cart from its cut"
    );
    assert_eq!(
        std::fs::metadata(&cut).and_then(|m| m.modified()).unwrap(),
        made,
        "the second boot cut the scan again"
    );
}

#[test]
fn a_label_a_sync_saves_is_cut_and_its_cart_drawn_in_it() {
    let Ok(surface) = HeadlessSurface::new() else {
        return;
    };
    let Ok(mut c) = Compositor::new(&surface) else {
        return;
    };
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    clocked(d.path());
    let mut f = Frontend::boot(Box::new(SimPlatform::at(d.path().to_path_buf())));
    f.upload_faces(&mut c);
    f.app_mut().finish_slide();
    f.compose(&mut c);
    assert_eq!(
        magenta_px(&c.read_frame()),
        0,
        "a magenta cart before any label"
    );

    write_scan(d.path(), "Emerald");
    f.app_mut()
        .set_label_jobs(Box::new(Saved(VecDeque::from([LabelNews::Saved {
            platform: Platform::Gba,
            stem: "Emerald".into(),
        }]))));
    let cut = d.path().join("Labels/GBA/.cut/Emerald.png");
    assert!(
        compose_until(&mut f, &mut c, |_, px| cut.is_file()
            && magenta_px(px) > 500),
        "the saved label was never cut and its cart never drawn in it"
    );
}
