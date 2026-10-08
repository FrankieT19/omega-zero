use std::path::Path;

use slot_gfx::{out_h, out_w, Draw, TexId};

use crate::art;

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
