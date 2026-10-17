#![cfg(target_os = "macos")]
use crate::common;
use omega_gfx::{out_h, out_w, Compositor, HeadlessSurface};
use omega_zero::frontend::Frontend;
#[test]
fn first_frame_is_unchanged_by_an_intro_request() {
    let Ok(surface) = HeadlessSurface::new() else {
        return;
    };
    let mut compositor = Compositor::new(&surface).unwrap();
    let d = common::tmp_root_with_carts(&["Emerald", "Fusion"]);
    common::clocked(d.path());
    let mut frontend = Frontend::boot(Box::new(omega_power::SimPlatform::at(
        d.path().to_path_buf(),
    )));
    frontend.upload_faces(&mut compositor);
    frontend.compose(&mut compositor);
    let before = compositor.read_frame();
    let logo = compositor.create_texture(
        out_w(),
        out_h(),
        &vec![255; (out_w() * out_h() * 4) as usize],
    );
    frontend.app_mut().set_boot_logo(logo, out_w(), out_h());
    frontend
        .app_mut()
        .start_opening(&omega_zero::opening::Facts::default());
    frontend.compose(&mut compositor);
    assert_eq!(compositor.read_frame(), before);
}
