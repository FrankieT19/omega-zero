use std::path::Path;

use omega_gfx::{out_h, out_w, Draw, TexId};
use omega_store::{BackdropStyle, BACKDROP_COLOURS};

use crate::art;
use crate::plate::UndoFace;

const SCRIM: f32 = 0.62;

pub fn wallpaper_face(path: &Path) -> Option<Vec<u8>> {
    art::cover(path, out_w(), out_h())
}

pub fn draw_backdrop(face: Option<TexId>, out: &mut Vec<Draw>) {
    let Some(tex) = face else {
        return;
    };
    out.push(Draw::Tex {
        x: 0.0,
        y: 0.0,
        w: out_w() as f32,
        h: out_h() as f32,
        tex,
        alpha: 1.0,
    });
    out.push(Draw::Rect {
        x: 0.0,
        y: 0.0,
        w: out_w() as f32,
        h: out_h() as f32,
        colour: [0.0, 0.0, 0.0, SCRIM],
    });
}

/// What each of [`BACKDROP_COLOURS`] looks like: dark enough that a cart's label always
/// reads over it, the first the near-black omega-zero drew before it had a choice.
pub const BACKDROP_PALETTE: [[u8; 3]; BACKDROP_COLOURS.len()] = [
    [0x05, 0x05, 0x08],
    [0x22, 0x25, 0x2c],
    [0x10, 0x1b, 0x36],
    [0x0b, 0x2b, 0x2e],
    [0x10, 0x2c, 0x16],
    [0x29, 0x14, 0x31],
    [0x30, 0x0f, 0x16],
    [0x30, 0x22, 0x08],
];

/// A colour's name to show, from the key omega-zero.state keeps it by.
pub fn backdrop_colour_name(colour: u8) -> &'static str {
    match BACKDROP_COLOURS.get(colour as usize).copied() {
        Some("slate") => "Slate",
        Some("navy") => "Navy",
        Some("teal") => "Teal",
        Some("green") => "Green",
        Some("plum") => "Plum",
        Some("crimson") => "Crimson",
        Some("amber") => "Amber",
        _ => "Night",
    }
}

pub fn backdrop_rgb(colour: u8) -> [u8; 3] {
    BACKDROP_PALETTE[(colour as usize).min(BACKDROP_PALETTE.len() - 1)]
}

/// How far a pattern goes before it looks the same again: the grid's square, the bits' tile.
/// Both divide 640, 720 and 480, so a still pattern sits square on every canvas.
pub fn backdrop_period(style: BackdropStyle) -> u32 {
    match style {
        BackdropStyle::Bits => 80,
        _ => 40,
    }
}

/// How fast a drifting pattern moves, in canvas pixels a second: slow enough to sit behind the
/// shelf rather than pull the eye from it, and a whole number of milliseconds to a period.
pub fn backdrop_speed(style: BackdropStyle) -> f64 {
    match style {
        BackdropStyle::Bits => 10.0,
        _ => 16.0,
    }
}

/// How far left a pattern has drifted `ms` milliseconds in, in whole pixels so its lines stay
/// sharp, starting over each period.
pub fn backdrop_shift(style: BackdropStyle, moving: bool, ms: u64) -> u32 {
    if !moving || !style.moves() {
        return 0;
    }
    let moved = ms as f64 / 1000.0 * backdrop_speed(style);
    (moved.floor() as u64 % backdrop_period(style) as u64) as u32
}

fn tint(base: [u8; 3], toward_white: f32) -> [u8; 4] {
    let c = |v: u8| (v as f32 + (255.0 - v as f32) * toward_white).round() as u8;
    [c(base[0]), c(base[1]), c(base[2]), 255]
}

const GRID_LINE: u32 = 2;
const BIT: u32 = 4;

/// Whether the bit at (`cx`, `cy`) of a tile is lit, and how brightly: a scatter that looks
/// random but is the same every time.
fn bit(cx: u32, cy: u32) -> Option<f32> {
    let mut h = cx.wrapping_mul(0x9e37_79b1) ^ cy.wrapping_mul(0x85eb_ca77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    match h % 100 {
        0..=5 => Some(0.07),
        6..=8 => Some(0.14),
        _ => None,
    }
}

/// The picture a drifting pattern is drawn from: the canvas and one period more, so sliding it
/// left by up to a period always covers the canvas. Nothing for a style with no pattern.
pub fn backdrop_pattern_face(style: BackdropStyle, colour: u8) -> Option<UndoFace> {
    let period = backdrop_period(style);
    let (w, h) = (out_w() + period, out_h());
    let base = backdrop_rgb(colour);
    let ground = tint(base, 0.0);
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let px = match style {
                BackdropStyle::Grid => {
                    let line = x % period < GRID_LINE || y % period < GRID_LINE;
                    if line {
                        tint(base, 0.09)
                    } else {
                        ground
                    }
                }
                BackdropStyle::Bits => {
                    let cells = period / BIT;
                    match bit((x / BIT) % cells, (y / BIT) % cells) {
                        Some(lift) => tint(base, lift),
                        None => ground,
                    }
                }
                BackdropStyle::Solid | BackdropStyle::Wallpaper => return None,
            };
            rgba.extend_from_slice(&px);
        }
    }
    Some(UndoFace { rgba, w, h })
}

/// The pattern `face` (made by [`backdrop_pattern_face`], `w` wide), drifted `shift` left.
pub fn draw_pattern(face: (TexId, u32), shift: u32, out: &mut Vec<Draw>) {
    out.push(Draw::Tex {
        x: -(shift as f32),
        y: 0.0,
        w: face.1 as f32,
        h: out_h() as f32,
        tex: face.0,
        alpha: 1.0,
    });
}

pub fn draw_solid(colour: u8, out: &mut Vec<Draw>) {
    let [r, g, b] = backdrop_rgb(colour);
    out.push(Draw::Rect {
        x: 0.0,
        y: 0.0,
        w: out_w() as f32,
        h: out_h() as f32,
        colour: [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0],
    });
}
