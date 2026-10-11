//! The save-state switcher: its screenshot over the whole game, the kit's SAVE STATES panel
//! (saves.rs) at the top in the middle under the top bar, and its keys on a line of their own
//! (kit/hints.rs) in the middle just above the bar's actions, each drawn at 2x on whole art
//! pixels; and its bar, B RETURN and A LOAD with nothing between them, held to Chrome's
//! omega-bar.

use std::path::Path;

use omega_gfx::{out_h, out_w, Draw, TexId};
use omega_store::{StateEntry, RING_MAX};
use omega_ui::kit::action::TALL;
use omega_ui::kit::bar::Hint;
use omega_ui::kit::hints::{hint_line_width, HINT_LINE_H};
use omega_ui::kit::saves::{saves_width, SAVES_H};
use omega_ui::{Polaroids, PHOTO_H, PHOTO_W, SPRITE_PX, TOP_BAR_H};

use crate::kit::{bar, matches};

fn entry(stamp: &str) -> StateEntry {
    StateEntry {
        stamp: stamp.to_string(),
        state: Path::new("/nonexistent").join(format!("{stamp}.state")),
        thumb: Path::new("/nonexistent").join(format!("{stamp}.png")),
    }
}

fn switcher_with(n: usize) -> Polaroids {
    Polaroids::new(
        (0..n)
            .map(|i| entry(&format!("2026-08-09_{:02}-00-00", 14 - i)))
            .collect(),
    )
}

fn draw(p: &Polaroids) -> Vec<Draw> {
    let mut out = Vec::new();
    p.draw(&mut out);
    out
}

#[test]
fn the_screenshot_is_the_first_thing_drawn_over_the_whole_screen() {
    let mut p = switcher_with(3);
    let out = draw(&p);
    assert!(
        matches!(out.first(), Some(Draw::Rect { x, y, w, h, colour })
            if (*x, *y, *w, *h) == (0.0, 0.0, out_w() as f32, out_h() as f32) && colour[3] == 1.0),
        "before its screenshot is up, the switcher hides the game: {:?}",
        out.first()
    );
    p.set_faces(vec![
        TexId::from_raw(1),
        TexId::from_raw(2),
        TexId::from_raw(3),
    ]);
    p.right();
    let out = draw(&p);
    assert_eq!(
        out.first(),
        Some(&Draw::Shot {
            tex: TexId::from_raw(2)
        })
    );
    if (out_w(), out_h()) == (720, 480) {
        assert_eq!(
            (out_w() / PHOTO_W, out_h() / PHOTO_H),
            (3, 3),
            "on 720x480 the screenshot is exactly 3x"
        );
    }
}

#[test]
fn the_panel_stands_in_the_middle_under_the_top_bar_on_whole_art_pixels() {
    let mut p = switcher_with(2);
    let (w, h) = (saves_width("Just now", 2), SAVES_H);
    p.set_panel_face(TexId::from_raw(9), w, h);
    let out = draw(&p);
    assert_eq!(out.len(), 2, "the screenshot and the panel, nothing else");
    let Draw::Tex {
        x,
        y,
        w: dw,
        h: dh,
        tex,
        alpha,
    } = out[1]
    else {
        panic!("the panel is not a face: {:?}", out[1]);
    };
    assert_eq!(tex, TexId::from_raw(9));
    assert_eq!(alpha, 1.0, "the panel is see through");
    assert_eq!((dw, dh), ((w * SPRITE_PX) as f32, (h * SPRITE_PX) as f32));
    for v in [x, y] {
        assert_eq!(
            v % SPRITE_PX as f32,
            0.0,
            "the panel is off the art grid at {v}"
        );
    }
    assert!(y >= TOP_BAR_H, "the panel reaches into the top bar");
    let (left, right) = (x, out_w() as f32 - x - dw);
    assert!(
        (0.0..=SPRITE_PX as f32).contains(&(right - left)),
        "the panel is not in the middle: {left} left, {right} right"
    );
    assert_eq!(Polaroids::panel_rect(w, h)[1] * SPRITE_PX, y as u32);
}

#[test]
fn nothing_the_switcher_draws_is_see_through() {
    let mut p = switcher_with(RING_MAX);
    p.set_faces(vec![TexId::from_raw(1)]);
    p.set_panel_face(TexId::from_raw(9), 90, SAVES_H);
    p.set_keys_face(TexId::from_raw(10), 176, HINT_LINE_H);
    let out = draw(&p);
    assert_eq!(out.len(), 3, "{out:?}");
    for d in out {
        match d {
            Draw::Rect { colour, .. } => assert_eq!(colour[3], 1.0, "{d:?}"),
            Draw::Tex { alpha, .. } => assert_eq!(alpha, 1.0, "{d:?}"),
            Draw::Shot { .. } => {}
            other => panic!("the switcher drew {other:?}"),
        }
    }
}

const DELETE: Hint = Hint {
    key: "Y",
    word: "DELETE",
};
const UNDO_SAVE: Hint = Hint {
    key: "X",
    word: "UNDO SAVE",
};

/// The line of keys is drawn last, in the middle of the screen on whole art pixels, 4 art pixels
/// above the tops of the bar's actions and clear of the panel: at its widest, SAVE with Y DELETE
/// and X UNDO SAVE, it fits across 640 as across 720.
#[test]
fn the_keys_stand_in_the_middle_above_the_bars_actions() {
    let mut p = switcher_with(3);
    let (pw, ph) = (saves_width("Just now", 3), SAVES_H);
    p.set_panel_face(TexId::from_raw(9), pw, ph);
    let w = hint_line_width("SAVE", &[DELETE, UNDO_SAVE]);
    assert_eq!(w, 176);
    p.set_keys_face(TexId::from_raw(10), w, HINT_LINE_H);
    let out = draw(&p);
    let Some(&Draw::Tex {
        x,
        y,
        w: dw,
        h: dh,
        tex,
        alpha,
    }) = out.last()
    else {
        panic!("the keys are not a face: {:?}", out.last());
    };
    assert_eq!((tex, alpha), (TexId::from_raw(10), 1.0));
    let px = SPRITE_PX as f32;
    assert_eq!((dw, dh), (w as f32 * px, HINT_LINE_H as f32 * px));
    for v in [x, y] {
        assert_eq!(v % px, 0.0, "the keys are off the art grid at {v}");
    }
    let (left, right) = (x, out_w() as f32 - x - dw);
    assert!(left >= 0.0 && right >= 0.0, "the keys run off the screen");
    assert!(
        (0.0..=px).contains(&(right - left)),
        "the keys are not in the middle: {left} left, {right} right"
    );
    let actions = (out_h() - TALL * SPRITE_PX) as f32;
    assert_eq!(actions - (y + dh), 4.0 * px, "not 4 above the actions");
    let [_, top, _, tall] = Polaroids::panel_rect(pw, ph);
    assert!(
        y > ((top + tall) * SPRITE_PX) as f32,
        "the keys reach the panel"
    );
    assert_eq!(
        Polaroids::keys_rect(w, HINT_LINE_H),
        [x as u32 / SPRITE_PX, y as u32 / SPRITE_PX, w, HINT_LINE_H]
    );
}

/// The switcher's bar has nothing between RETURN and LOAD: its keys are on their own line.
#[test]
fn the_switchers_bar_is_chromes() {
    let (left, right) = (("RETURN", "B"), ("LOAD", "A"));
    for w in [320, 360] {
        matches(&bar(w, left, right, &[]), &format!("bar-switcher-{w}"));
    }
}
