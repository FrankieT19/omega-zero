//! The frontend never overlays a splash on the cart selector.
use crate::common::{app_booting_at, tmp_root_with_carts, CLOCK_IS_SET};
use omega_store::{write_app_state, AppState};
use omega_ui::TexId;
#[test]
fn boot_logo_does_not_cover_the_selector() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    write_app_state(
        d.path(),
        &AppState {
            clock_set: true,
            ..Default::default()
        },
    )
    .unwrap();
    let (mut app, _) = app_booting_at(d.path(), CLOCK_IS_SET);
    app.set_boot_logo(TexId::from_raw(900), 640, 480);
    assert!(!app.boot_fades());
    assert_eq!(app.boot_logo_alpha(), None);
}
