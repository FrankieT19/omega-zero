//! Each menu screen's input and drawing, as `impl App` blocks: `App::apply` hands a press to
//! the screen it is on and `App::draw` draws the screen it shows, and what each screen does
//! with them lives in its own file here.

use omega_gfx::{out_h, out_w, Draw};
use omega_ui::opening;

pub mod about;
pub mod background;
pub mod bar;
pub mod clock;
pub mod controls;
pub mod keyboard;
pub mod labels;
pub mod network;
pub mod settings;
pub mod shelf;
pub mod ssh;

/// How much of the backdrop a menu's plate lets through.
const PLATE_ALPHA: f32 = 0.6;

/// A dark plate the size of the screen behind a menu, so its text reads over any backdrop.
/// It slides with the menu. The menus' own panels take its place next.
pub(crate) fn draw_plate(out: &mut Vec<Draw>) {
    let [r, g, b, _] = opening();
    out.push(Draw::Rect {
        x: 0.0,
        y: 0.0,
        w: out_w() as f32,
        h: out_h() as f32,
        colour: [r, g, b, PLATE_ALPHA],
    });
}
