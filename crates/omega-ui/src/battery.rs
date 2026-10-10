use omega_gfx::{Draw, TexId};
use omega_power::{Battery, Charge};

use crate::footer::Printed;
use crate::hud::HUD_INK;
use crate::plate::{word_cap_top, UndoFace, HINT_H};
use crate::sprite::{sprite_face, Rows, CELL_PX, SPRITE_PX};

/// The capsule is drawn a sprite pixel at a time, as tall as an icon's cell, so it sits on the
/// same grid as the bolt and the figure beside it.
const PX: f32 = SPRITE_PX as f32;

pub const GAUGE_W: f32 = 12.0 * PX;
pub const GAUGE_H: f32 = CELL_PX;
const _: () = assert!(GAUGE_H < GAUGE_W);
const NUB_W: f32 = PX;
const NUB_H: f32 = 3.0 * PX;
pub const WALL: f32 = PX;
const GAP: f32 = 4.0 * PX;

const BOLT_W: f32 = CELL_PX;
const BOLT_GAP: f32 = 2.0 * PX;
const _: () = assert!(BOLT_GAP > 0.0);

#[rustfmt::skip]
const BOLT: Rows = [
    "....##.",
    "...##..",
    "..##...",
    ".#####.",
    "...##..",
    "..##...",
    ".##....",
];

const INK: [f32; 4] = [
    HUD_INK[0] as f32 / 255.0,
    HUD_INK[1] as f32 / 255.0,
    HUD_INK[2] as f32 / 255.0,
    1.0,
];

/// The bolt shown beside the capsule while the handheld charges, one cell square.
pub fn bolt_face() -> UndoFace {
    sprite_face(&BOLT, HUD_INK)
}

/// How wide the gauge is drawn with `percent` beside it, from the room left for the bolt
/// on its left to the end of the figure.
pub fn gauge_width(percent: Printed) -> f32 {
    let figure = if percent.w > 0 {
        GAP + percent.w as f32
    } else {
        0.0
    };
    BOLT_W + BOLT_GAP + GAUGE_W + NUB_W + figure
}

/// How many of the capsule's `cols` columns of fill `percent` lights: whole sprite pixels, so
/// the fill steps like the rest of the art, and never none while there is any charge left.
fn filled(percent: u8, cols: u32) -> u32 {
    match u32::from(percent.min(100)) {
        0 => 0,
        p => ((p * cols + 50) / 100).max(1),
    }
}

/// The gauge with the top of its capsule at `y`; the bolt and the figure stand on the same
/// line, all three exactly `GAUGE_H` tall.
pub fn draw_gauge(
    x: f32,
    y: f32,
    battery: Option<Battery>,
    percent: Printed,
    bolt: Option<TexId>,
    out: &mut Vec<Draw>,
) {
    let Some(b) = battery else {
        return;
    };

    let rect = |x: f32, y: f32, w: f32, h: f32, out: &mut Vec<Draw>| {
        out.push(Draw::Rect {
            x,
            y,
            w,
            h,
            colour: INK,
        });
    };

    let cx = x + BOLT_W + BOLT_GAP;

    rect(cx, y, GAUGE_W, WALL, out);
    rect(cx, y + GAUGE_H - WALL, GAUGE_W, WALL, out);
    rect(cx, y, WALL, GAUGE_H, out);
    rect(cx + GAUGE_W - WALL, y, WALL, GAUGE_H, out);
    rect(cx + GAUGE_W, y + (GAUGE_H - NUB_H) / 2.0, NUB_W, NUB_H, out);

    let cols = ((GAUGE_W - 4.0 * WALL) / PX) as u32;
    let fill = filled(b.percent, cols) as f32 * PX;
    if fill > 0.0 {
        rect(
            cx + 2.0 * WALL,
            y + 2.0 * WALL,
            fill,
            GAUGE_H - 4.0 * WALL,
            out,
        );
    }

    if let (Charge::Charging, Some(tex)) = (b.charge, bolt) {
        out.push(Draw::Tex {
            x,
            y,
            w: BOLT_W,
            h: CELL_PX,
            tex,
            alpha: 1.0,
        });
    }

    if percent.w > 0 {
        let px = cx + GAUGE_W + NUB_W + GAP;
        let py = y - word_cap_top();
        out.push(match percent.face {
            Some(tex) => Draw::Tex {
                x: px,
                y: py,
                w: percent.w as f32,
                h: HINT_H as f32,
                tex,
                alpha: 1.0,
            },
            None => Draw::Rect {
                x: px,
                y: py,
                w: percent.w as f32,
                h: HINT_H as f32,
                colour: [1.0, 1.0, 1.0, 0.08],
            },
        });
    }
}
