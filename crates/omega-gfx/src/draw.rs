use crate::canvas::{out_h, out_w};
use crate::quad::Quad;
use crate::shaders::{SPRITE_FRAG, SPRITE_VERT};
use crate::surface::GfxError;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct TexId(usize);

/// How far in front of the canvas the eye stands when it looks at a `Draw::Spun` face, in
/// canvas pixels: the nearer it stands, the more the side of a face that turns towards it grows.
pub const FOCAL: f32 = 1100.0;

/// Where the point `dx` across from the upright middle of a `Draw::Spun` face lands, measured
/// across the canvas from that middle, once the face is spun `yaw` and slid back `depth`.
pub fn spun_x(dx: f32, yaw: f32, depth: f32) -> f32 {
    let (sin, cos) = yaw.sin_cos();
    (dx * cos - depth * sin) * FOCAL / (FOCAL + dx * sin)
}

#[cfg(feature = "test-support")]
impl TexId {
    pub const fn from_raw(id: usize) -> Self {
        TexId(id)
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Draw {
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        colour: [f32; 4],
    },
    Tex {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        tex: TexId,
        alpha: f32,
    },
    Turned {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        tex: TexId,
        alpha: f32,
        turn: f32,
    },
    /// `tex` turned `turn` radians in the canvas like `Turned`, then spun `yaw` radians about
    /// its upright middle and seen in perspective. `depth` slides it that many canvas pixels
    /// straight back, unshrunk, the way a slice through something thick would sit behind its
    /// face. Only what falls inside `clip`, a canvas rect, is drawn.
    Spun {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        tex: TexId,
        colour: [f32; 4],
        turn: f32,
        yaw: f32,
        depth: f32,
        clip: Option<[f32; 4]>,
    },
    Game,
    Shot {
        tex: TexId,
    },
}

pub struct Sprites {
    prog: gl::types::GLuint,
    white: gl::types::GLuint,
    textures: Vec<gl::types::GLuint>,
    u_rect: gl::types::GLint,
    u_colour: gl::types::GLint,
    u_turn: gl::types::GLint,
    u_spin: gl::types::GLint,
    u_spun: gl::types::GLint,
}

impl Sprites {
    pub fn new() -> Result<Self, GfxError> {
        let prog = crate::shaders::program(SPRITE_VERT, SPRITE_FRAG)?;
        let white = crate::gl::texture(
            1,
            1,
            gl::NEAREST,
            gl::CLAMP_TO_EDGE,
            gl::RGBA,
            Some(&[255u8; 4]),
        );
        let (u_rect, u_colour, u_turn, u_spin, u_spun);
        unsafe {
            gl::UseProgram(prog);
            gl::Uniform1i(crate::gl::uniform_location(prog, "u_tex"), 0);
            gl::Uniform2f(
                crate::gl::uniform_location(prog, "u_target"),
                out_w() as f32,
                out_h() as f32,
            );
            u_rect = crate::gl::uniform_location(prog, "u_rect");
            u_colour = crate::gl::uniform_location(prog, "u_colour");
            u_turn = crate::gl::uniform_location(prog, "u_turn");
            u_spin = crate::gl::uniform_location(prog, "u_spin");
            u_spun = crate::gl::uniform_location(prog, "u_spun");
            gl::Uniform1f(crate::gl::uniform_location(prog, "u_focal"), FOCAL);
        }
        Ok(Sprites {
            prog,
            white,
            textures: Vec::new(),
            u_rect,
            u_colour,
            u_turn,
            u_spin,
            u_spun,
        })
    }

    pub fn create_texture(&mut self, w: u32, h: u32, rgba: &[u8]) -> TexId {
        self.push(w, h, rgba, gl::LINEAR)
    }

    pub fn create_texture_nearest(&mut self, w: u32, h: u32, rgba: &[u8]) -> TexId {
        self.push(w, h, rgba, gl::NEAREST)
    }

    fn push(&mut self, w: u32, h: u32, rgba: &[u8], filter: gl::types::GLenum) -> TexId {
        let tex = crate::gl::texture(w, h, filter, gl::CLAMP_TO_EDGE, gl::RGBA, Some(rgba));
        self.textures.push(tex);
        TexId(self.textures.len() - 1)
    }

    pub fn update_texture(&mut self, id: TexId, w: u32, h: u32, rgba: &[u8]) {
        let Some(tex) = self.textures.get(id.0) else {
            return;
        };
        if rgba.len() < (w * h * 4) as usize {
            return;
        }
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, *tex);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                crate::gl::internal_format(gl::RGBA),
                w as i32,
                h as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                rgba.as_ptr() as *const std::ffi::c_void,
            );
        }
    }

    pub fn source(&self, id: TexId) -> Option<gl::types::GLuint> {
        self.textures.get(id.0).copied()
    }

    pub fn draw(&self, items: &[Draw], quad: &Quad) {
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::UseProgram(self.prog);
            gl::ActiveTexture(gl::TEXTURE0);
        }
        let mut clipped = false;
        for item in items {
            let (mut yaw, mut depth, mut clip) = (0.0, 0.0, None);
            let (x, y, w, h, tex, colour, turn) = match *item {
                Draw::Rect { x, y, w, h, colour } => (x, y, w, h, self.white, colour, 0.0),
                Draw::Spun {
                    x,
                    y,
                    w,
                    h,
                    tex,
                    colour,
                    turn,
                    yaw: spun,
                    depth: back,
                    clip: within,
                } => match self.textures.get(tex.0) {
                    Some(t) => {
                        (yaw, depth, clip) = (spun, back, within);
                        (x, y, w, h, *t, colour, turn)
                    }
                    None => continue,
                },
                Draw::Tex {
                    x,
                    y,
                    w,
                    h,
                    tex,
                    alpha,
                } => match self.textures.get(tex.0) {
                    Some(t) => (x, y, w, h, *t, [1.0, 1.0, 1.0, alpha], 0.0),
                    None => continue,
                },
                Draw::Turned {
                    x,
                    y,
                    w,
                    h,
                    tex,
                    alpha,
                    turn,
                } => match self.textures.get(tex.0) {
                    Some(t) => (x, y, w, h, *t, [1.0, 1.0, 1.0, alpha], turn),
                    None => continue,
                },
                Draw::Game | Draw::Shot { .. } => continue,
            };
            let (cos, sin) = if turn == 0.0 {
                (1.0, 0.0)
            } else {
                (turn.cos(), turn.sin())
            };
            let (yaw_sin, yaw_cos) = if yaw == 0.0 {
                (0.0, 1.0)
            } else {
                yaw.sin_cos()
            };
            unsafe {
                match clip {
                    Some(r) => {
                        let (left, right) = (r[0].floor().max(0.0), (r[0] + r[2]).ceil());
                        let (top, bottom) = (r[1].floor().max(0.0), (r[1] + r[3]).ceil());
                        let bottom = bottom.min(out_h() as f32);
                        gl::Enable(gl::SCISSOR_TEST);
                        gl::Scissor(
                            left as i32,
                            (out_h() as f32 - bottom) as i32,
                            (right - left).max(0.0) as i32,
                            (bottom - top).max(0.0) as i32,
                        );
                        clipped = true;
                    }
                    None if clipped => {
                        gl::Disable(gl::SCISSOR_TEST);
                        clipped = false;
                    }
                    None => {}
                }
                gl::BindTexture(gl::TEXTURE_2D, tex);
                gl::Uniform4f(self.u_rect, x, y, w, h);
                gl::Uniform2f(self.u_turn, cos, sin);
                gl::Uniform3f(self.u_spin, yaw_sin, yaw_cos, depth);
                gl::Uniform1f(self.u_spun, if yaw == 0.0 { 0.0 } else { 1.0 });
                gl::Uniform4f(self.u_colour, colour[0], colour[1], colour[2], colour[3]);
            }
            quad.draw();
        }
        unsafe {
            if clipped {
                gl::Disable(gl::SCISSOR_TEST);
            }
            gl::Disable(gl::BLEND);
        }
    }
}

impl Drop for Sprites {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(self.textures.len() as i32, self.textures.as_ptr());
            gl::DeleteTextures(1, &self.white);
            gl::DeleteProgram(self.prog);
        }
    }
}
