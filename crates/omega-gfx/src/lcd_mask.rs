//! Upstream LCD default coefficients evaluated at output pixel centres and cached on geometry
//! changes. Phases follow the visible source pixels; Grid follows physical display pixels.
use crate::quad::Quad;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Key {
    pub size: (u32, u32),
    pub raster: [u32; 2],
    pub origin: [u32; 2],
    pub kind: u8,
    pub source_grid: bool,
}
pub(crate) struct Mask {
    key: Option<Key>,
    tex: u32,
}
impl Mask {
    pub fn new() -> Self {
        Self { key: None, tex: 0 }
    }
    pub fn get(&mut self, key: Key, _quad: &Quad) -> u32 {
        if self.key == Some(key) {
            return self.tex;
        }
        let data = pixels(key);
        unsafe {
            if self.tex != 0 {
                gl::DeleteTextures(1, &self.tex);
            }
            self.tex = crate::gl::texture(
                key.size.0,
                key.size.1,
                gl::NEAREST,
                gl::CLAMP_TO_EDGE,
                gl::RGBA,
                Some(&data),
            );
        }
        self.key = Some(key);
        self.tex
    }
}
impl Drop for Mask {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.tex);
        }
    }
}
fn phase(at: u32, n: u32, raster: f32, origin: f32, snap: bool) -> f32 {
    let pitch = n as f32 / raster;
    // Exact integer cells must repeat bit-for-bit, including half-step mask quantization.
    let p = if (pitch - pitch.round()).abs() < 0.00001 && pitch.round() >= 1.0 {
        (((at % pitch.round() as u32) as f32 + 0.5) / pitch.round() + origin.rem_euclid(1.0))
            .rem_euclid(1.0)
    } else {
        ((at as f32 + 0.5) * raster / n as f32 + origin).rem_euclid(1.0)
    };
    if snap {
        (p * 3.0).floor().min(2.0) / 3.0 + 1.0 / 6.0
    } else {
        p
    }
}
fn pixels(key: Key) -> Vec<u8> {
    let (w, h) = key.size;
    let raster = key.raster.map(f32::from_bits);
    let origin = key.origin.map(f32::from_bits);
    let xs: Vec<_> = (0..w)
        .map(|x| phase(x, w, raster[0], origin[0], key.source_grid))
        .collect();
    let ys: Vec<_> = (0..h)
        .map(|y| phase(y, h, raster[1], origin[1], key.source_grid))
        .collect();
    let mut data = Vec::with_capacity((w * h * 4) as usize);
    for y in ys {
        for x in &xs {
            if key.kind < 2 {
                // Upstream LCD1x has a quarter-cell offset on both axes. LCD3x uses
                // a half-pi RGB phase followed by negative two-thirds-pi channel steps.
                // Keep continuous phase at noninteger scales; never snap to a 3x3 table.
                let tau = std::f32::consts::TAU;
                let (xp, yp) = if key.kind == 0 {
                    (*x - 0.25, y - 0.25)
                } else {
                    (*x, y)
                };
                let yf = (16.0 + (tau * yp).sin()) / 17.0;
                for c in 0..3 {
                    let offset = if key.kind == 1 {
                        std::f32::consts::PI * (0.5 - c as f32 * 2.0 / 3.0)
                    } else {
                        0.0
                    };
                    let xf = (4.0 + (tau * xp + offset).sin()) / 5.0;
                    data.push((255.0 * xf * yf).round() as u8);
                }
            } else {
                data.extend_from_slice(
                    &[if *x >= 2.0 / 3.0 || y >= 2.0 / 3.0 {
                        0
                    } else {
                        255
                    }; 3],
                );
            }
            data.push(255);
        }
    }
    data
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn integer_pixel_cells_repeat_exactly_even_at_half_step_quantization() {
        for x in 0..720 {
            assert_eq!(
                phase(x, 720, 240.0, 0.0, false),
                phase(x % 3, 720, 240.0, 0.0, false)
            );
        }
    }
    #[test]
    fn cropped_mask_phase_uses_the_same_source_pixel_boundaries_as_the_game() {
        let a = phase(0, 720, 320.0 * 0.99, 3.2, false);
        assert!((a - ((0.5 * 320.0 * 0.99 / 720.0 + 3.2_f32) % 1.0)).abs() < 0.0001);
    }
}
