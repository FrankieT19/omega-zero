use crate::canvas::{out_h, out_w};

const LINE_PX: f32 = 2.0;

const DOT_T: f32 = 0.12;

const OVERSHOOT: f32 = 0.6;

pub fn screen_scale(t: f32) -> f32 {
    let left = 1.0 - t.clamp(0.0, 1.0);
    1.0 - (1.0 - LINE_PX / out_h() as f32) * left * left
}

pub fn screen_width(t: f32) -> f32 {
    let left = 1.0 - (t.clamp(0.0, 1.0) / DOT_T).min(1.0);
    1.0 - (1.0 - LINE_PX / out_w() as f32) * left
}

pub fn screen_brightness(t: f32) -> f32 {
    let left = 1.0 - t.clamp(0.0, 1.0);
    1.0 + OVERSHOOT * left * left
}

/// Where a picture that fills `r` when the screen is on stands at power `t`, collapsing to a
/// line across the middle of `r` as the screen goes off.
pub fn screen_rect_in(t: f32, r: [f32; 4]) -> (f32, f32, f32, f32) {
    let w = r[2] * screen_width(t);
    let h = r[3] * screen_scale(t);
    (r[0] + (r[2] - w) / 2.0, r[1] + (r[3] - h) / 2.0, w, h)
}
