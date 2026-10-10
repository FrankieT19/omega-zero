//! About: the handheld's sticker, opened from Settings. It only slides, in and back out like
//! every screen, over the backdrop that stays where it is under all of them.

use crate::common;

use common::{app_booting_at, tmp_root_with_carts, CLOCK_IS_SET};
use omega_input::{Action, Btn};
use omega_store::{write_app_state, AppState, BackdropStyle};
use omega_ui::{backdrop_shift, out_w, sticker_rect, Draw, SettingsRow, TexId};
use omega_zero::app::{App, Phase};
use omega_zero::nav::{offsets, progress, Dir, End, SLIDE_MS};
use tempfile::TempDir;

const STICKER: TexId = TexId::from_raw(800);
const PATTERN: TexId = TexId::from_raw(700);

fn on_settings() -> (TempDir, App) {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    write_app_state(
        d.path(),
        &AppState {
            clock_set: true,
            ..AppState::default()
        },
    )
    .unwrap();
    let (mut a, _) = app_booting_at(d.path(), CLOCK_IS_SET);
    a.tick_ms(16);
    a.set_sticker_face(STICKER);
    a.set_backdrop_face(PATTERN, out_w() + 40, BackdropStyle::Grid, 0);
    press(&mut a, Btn::Start);
    a.finish_slide();
    (d, a)
}

fn press(a: &mut App, btn: Btn) {
    a.apply(Action::GbaDown(btn));
    a.apply(Action::GbaUp(btn));
}

fn frame(a: &App) -> Vec<Draw> {
    let mut out = Vec::new();
    a.draw(&mut out);
    out
}

/// Where `tex` is first drawn across, and its place in the frame.
fn placed(out: &[Draw], tex: TexId) -> Option<(usize, f32)> {
    out.iter().enumerate().find_map(|(i, d)| match *d {
        Draw::Tex { x, tex: t, .. } if t == tex => Some((i, x)),
        _ => None,
    })
}

/// Halfway through the slide under way: the sticker is moved with its screen, and the backdrop
/// is drawn once, before it, where it is when nothing slides.
#[track_caller]
fn halfway(a: &mut App, dir: Dir, end: End) {
    let started = a.slide().expect("the move did not slide").started;
    a.tick_ms(started + SLIDE_MS / 2);
    let out = frame(a);
    let dx = offsets(dir, end, progress(started, a.now()), out_w() as f32).dx;
    assert_ne!(dx, 0.0, "halfway, the sticker is home");
    let (at, x) = placed(&out, STICKER).expect("the sticker is not drawn while it slides");
    assert_eq!(
        x,
        sticker_rect().0 + dx,
        "the sticker does not slide with its screen"
    );
    let (under, px) = placed(&out, PATTERN).expect("no backdrop under the slide");
    assert!(under < at, "the backdrop is drawn over the sticker");
    assert_eq!(
        px,
        -(backdrop_shift(BackdropStyle::Grid, true, a.now()) as f32),
        "the backdrop slid"
    );
    let patterns = out
        .iter()
        .filter(|d| matches!(**d, Draw::Tex { tex, .. } if tex == PATTERN))
        .count();
    assert_eq!(patterns, 1, "a backdrop for each end of the slide");
}

#[test]
fn the_sticker_only_slides_in_and_out_over_a_backdrop_that_stays() {
    let (_d, mut a) = on_settings();
    while a.settings_row() != Some(SettingsRow::About) {
        press(&mut a, Btn::Down);
    }
    press(&mut a, Btn::A);
    assert!(matches!(a.phase(), Phase::About));
    halfway(&mut a, Dir::Deeper, End::Arriving);

    a.finish_slide();
    let (_, x) = placed(&frame(&a), STICKER).expect("the sticker at rest");
    assert_eq!(x, sticker_rect().0, "the sticker did not come to rest home");

    press(&mut a, Btn::B);
    assert_eq!(a.settings_row(), Some(SettingsRow::About));
    halfway(&mut a, Dir::Back, End::Leaving);
}
