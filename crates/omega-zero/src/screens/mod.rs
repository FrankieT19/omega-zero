//! Each menu screen's input and drawing, as `impl App` blocks: `App::apply` hands a press to
//! the screen it is on and `App::draw` draws the screen it shows, and what each screen does
//! with them lives in its own file here.

use omega_gfx::{out_h, out_w, Draw};
use omega_ui::opening;

pub mod about;
pub mod background;
pub mod clock;
pub mod controls;
pub mod keyboard;
pub mod labels;
pub mod lone;
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

/// `d` moved `dx` across and `dy` down, a spun face's clip with it. It moves to omega-gfx as
/// `Draw::shifted` once that lands.
fn shift(d: Draw, dx: f32, dy: f32) -> Draw {
    match d {
        Draw::Rect { x, y, w, h, colour } => Draw::Rect {
            x: x + dx,
            y: y + dy,
            w,
            h,
            colour,
        },
        Draw::Tex {
            x,
            y,
            w,
            h,
            tex,
            alpha,
        } => Draw::Tex {
            x: x + dx,
            y: y + dy,
            w,
            h,
            tex,
            alpha,
        },
        Draw::Turned {
            x,
            y,
            w,
            h,
            tex,
            alpha,
            turn,
        } => Draw::Turned {
            x: x + dx,
            y: y + dy,
            w,
            h,
            tex,
            alpha,
            turn,
        },
        Draw::Spun {
            x,
            y,
            w,
            h,
            tex,
            colour,
            turn,
            yaw,
            depth,
            clip,
        } => Draw::Spun {
            x: x + dx,
            y: y + dy,
            w,
            h,
            tex,
            colour,
            turn,
            yaw,
            depth,
            clip: clip.map(|[cx, cy, cw, ch]| [cx + dx, cy + dy, cw, ch]),
        },
        // the game and a shot fill the screen wherever they are drawn
        Draw::Game | Draw::Shot { .. } => d,
    }
}

/// Moves everything in `drawn` by `dx` across and `dy` down.
pub(crate) fn shift_all(drawn: &mut [Draw], dx: f32, dy: f32) {
    if dx == 0.0 && dy == 0.0 {
        return;
    }
    for d in drawn {
        *d = shift(*d, dx, dy);
    }
}
