//! Cached viewport masks sampled at exact pixel centres. Slang LCD equations are evaluated
//! only when geometry changes, never once per display pixel on every frame.
use crate::{quad::Quad, slang::Slang, surface::GfxError};

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
    pub lcd: Slang,
}
impl Mask {
    pub fn new() -> Result<Self, GfxError> {
        Ok(Self {
            key: None,
            tex: 0,
            lcd: Slang::new()?,
        })
    }
    pub fn get(&mut self, key: Key, quad: &Quad) -> u32 {
        if self.key == Some(key) {
            return self.tex;
        }
        let table = if key.kind == 2 {
            None
        } else {
            Some(self.lcd.table(key.kind, quad))
        };
        let data = pixels(key, table);
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
    let p = ((at as f32 + 0.5) * raster / n as f32 + origin).rem_euclid(1.0);
    if snap {
        (p * 3.0).floor().min(2.0) / 3.0 + 1.0 / 6.0
    } else {
        p
    }
}
fn pixels(key: Key, table: Option<&[u8]>) -> Vec<u8> {
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
            if let Some(table) = table {
                // Bilinear interpolation of coefficients only; game pixels always use nearest.
                let xx = x * 384.0 - 0.5;
                let yy = y * 384.0 - 0.5;
                let ix = xx.floor() as i32;
                let iy = yy.floor() as i32;
                let fx = xx - xx.floor();
                let fy = yy - yy.floor();
                for c in 0..3 {
                    let at = |dx: i32, dy: i32| {
                        table[((iy + dy).rem_euclid(384) as usize * 384
                            + (ix + dx).rem_euclid(384) as usize)
                            * 4
                            + c] as f32
                    };
                    let a = at(0, 0) * (1.0 - fx) + at(1, 0) * fx;
                    let b = at(0, 1) * (1.0 - fx) + at(1, 1) * fx;
                    data.push((a * (1.0 - fy) + b * fy).round() as u8);
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
    fn cropped_mask_phase_uses_the_same_source_pixel_boundaries_as_the_game() {
        let a = phase(0, 720, 320.0 * 0.99, 3.2, false);
        assert!((a - ((0.5 * 320.0 * 0.99 / 720.0 + 3.2_f32) % 1.0)).abs() < 0.0001);
    }
}
