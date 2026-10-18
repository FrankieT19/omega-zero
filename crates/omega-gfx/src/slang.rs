//! Slang is compiled with glslang/SPIRV-Cross, not rewritten by hand. LCD colour modulation
//! is linear, so evaluate the genuine shader on white once and cache its coefficients.
use crate::{quad::Quad, surface::GfxError};
const VERT: &str = r#"
attribute vec2 a_pos;
uniform vec4 u_rect,u_crop;
uniform vec2 u_target;
varying highp vec2 vTexCoord;
void main() {
    vTexCoord=u_crop.xy+a_pos*u_crop.zw;
    vec2 p=(u_rect.xy+a_pos*u_rect.zw)/u_target;
    gl_Position=vec4(p.x*2.0-1.0,1.0-p.y*2.0,0.0,1.0);
}
"#;
pub(crate) struct Slang {
    programs: [u32; 2],
    tables: [Option<Vec<u8>>; 2],
}
impl Slang {
    pub fn new() -> Result<Self, GfxError> {
        Ok(Self {
            programs: [
                crate::shaders::program(VERT, include_str!("../shaders/lcd1x.generated.glsl"))?,
                crate::shaders::program(VERT, include_str!("../shaders/lcd3x.generated.glsl"))?,
            ],
            tables: [None, None],
        })
    }
    pub fn draw(
        &self,
        kind: u8,
        source: u32,
        size: (u32, u32),
        view: crate::pipeline::GameView,
        target: (u32, u32),
        quad: &Quad,
    ) {
        let rect = view.rect;
        let crop = view.uv;
        let p = self.programs[kind as usize];
        let loc = |name| crate::gl::uniform_location(p, name);
        unsafe {
            gl::UseProgram(p);
            gl::Uniform1i(loc("Source"), 0);
            gl::Uniform4f(loc("u_rect"), rect[0], rect[1], rect[2], rect[3]);
            gl::Uniform4f(loc("u_crop"), crop[0], crop[1], crop[2], crop[3]);
            gl::Uniform2f(loc("u_target"), target.0 as f32, target.1 as f32);
            for name in ["registers.OriginalSize", "params.OriginalSize"] {
                gl::Uniform4f(
                    loc(name),
                    size.0 as f32,
                    size.1 as f32,
                    1.0 / size.0 as f32,
                    1.0 / size.1 as f32,
                );
            }
            for name in ["registers.BRIGHTEN_SCANLINES", "params.brighten_scanlines"] {
                gl::Uniform1f(loc(name), 16.0);
            }
            for name in ["registers.BRIGHTEN_LCD", "params.brighten_lcd"] {
                gl::Uniform1f(loc(name), 4.0);
            }
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, source);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            quad.draw();
        }
    }
    pub fn table(&mut self, kind: u8, quad: &Quad) -> &[u8] {
        if self.tables[kind as usize].is_none() {
            const N: u32 = 384;
            let mut data = vec![0u8; (N * N * 4) as usize];
            unsafe {
                let mut old = 0;
                let mut viewport = [0; 4];
                gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut old);
                gl::GetIntegerv(gl::VIEWPORT, viewport.as_mut_ptr());
                let blend = gl::IsEnabled(gl::BLEND) != 0;
                gl::Disable(gl::BLEND);
                gl::ActiveTexture(gl::TEXTURE0);
                let white = crate::gl::texture(
                    1,
                    1,
                    gl::NEAREST,
                    gl::CLAMP_TO_EDGE,
                    gl::RGBA,
                    Some(&[255; 4]),
                );
                let tex = crate::gl::texture(N, N, gl::NEAREST, gl::CLAMP_TO_EDGE, gl::RGBA, None);
                let mut fbo = 0;
                gl::GenFramebuffers(1, &mut fbo);
                gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gl::TEXTURE_2D,
                    tex,
                    0,
                );
                gl::Viewport(0, 0, N as i32, N as i32);
                self.draw(
                    kind,
                    white,
                    (1, 1),
                    crate::pipeline::GameView {
                        rect: [0.0, 0.0, N as f32, N as f32],
                        uv: [0.0, 0.0, 1.0, 1.0],
                        filter: crate::pipeline::Filter::Sharp,
                    },
                    (N, N),
                    quad,
                );
                gl::ReadPixels(
                    0,
                    0,
                    N as i32,
                    N as i32,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    data.as_mut_ptr().cast(),
                );
                // The rectangle vertex maps top to low texture coordinates; ReadPixels is bottom-up.
                let row = (N * 4) as usize;
                for y in 0..N as usize / 2 {
                    for x in 0..row {
                        data.swap(y * row + x, (N as usize - 1 - y) * row + x);
                    }
                }
                gl::BindFramebuffer(gl::FRAMEBUFFER, old as u32);
                gl::Viewport(viewport[0], viewport[1], viewport[2], viewport[3]);
                gl::DeleteFramebuffers(1, &fbo);
                gl::DeleteTextures(1, &white);
                gl::DeleteTextures(1, &tex);
                if blend {
                    gl::Enable(gl::BLEND);
                }
            }
            self.tables[kind as usize] = Some(data);
        }
        self.tables[kind as usize].as_deref().unwrap()
    }
}
impl Drop for Slang {
    fn drop(&mut self) {
        unsafe {
            for p in self.programs {
                gl::DeleteProgram(p);
            }
        }
    }
}
