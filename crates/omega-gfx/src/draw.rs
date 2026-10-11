use crate::canvas::{out_h, out_w};
use crate::quad::Quad;
use crate::shaders::{SPRITE_FRAG, SPRITE_VERT, SPUN_FRAG, SPUN_VERT};
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

impl Draw {
    /// The same draw moved `dx` across and `dy` down the canvas, a spun face's clip with it, so
    /// a group laid out from (0, 0) can be put anywhere. The game and a shot fill their own
    /// rects, which stay put.
    pub fn shifted(self, dx: f32, dy: f32) -> Draw {
        match self {
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
            Draw::Game | Draw::Shot { .. } => self,
        }
    }
}

/// A linked sprite program and where its uniforms are.
struct Program {
    prog: gl::types::GLuint,
    u_rect: gl::types::GLint,
    u_colour: gl::types::GLint,
    u_turn: gl::types::GLint,
    u_spin: gl::types::GLint,
}

impl Program {
    fn new(vert: &str, frag: &str) -> Result<Self, GfxError> {
        let prog = crate::shaders::program(vert, frag)?;
        unsafe {
            gl::UseProgram(prog);
            gl::Uniform1i(crate::gl::uniform_location(prog, "u_tex"), 0);
            gl::Uniform2f(
                crate::gl::uniform_location(prog, "u_target"),
                out_w() as f32,
                out_h() as f32,
            );
            gl::Uniform1f(crate::gl::uniform_location(prog, "u_focal"), FOCAL);
            Ok(Program {
                prog,
                u_rect: crate::gl::uniform_location(prog, "u_rect"),
                u_colour: crate::gl::uniform_location(prog, "u_colour"),
                u_turn: crate::gl::uniform_location(prog, "u_turn"),
                u_spin: crate::gl::uniform_location(prog, "u_spin"),
            })
        }
    }
}

pub struct Sprites {
    /// Everything that does not spin, through the shader sprites always had.
    plain: Program,
    /// Faces spun about their upright axis, which cost a little more to fill.
    spun: Program,
    white: gl::types::GLuint,
    textures: Vec<gl::types::GLuint>,
}

impl Sprites {
    pub fn new() -> Result<Self, GfxError> {
        let plain = Program::new(SPRITE_VERT, SPRITE_FRAG)?;
        let spun = Program::new(SPUN_VERT, SPUN_FRAG)?;
        let white = crate::gl::texture(
            1,
            1,
            gl::NEAREST,
            gl::CLAMP_TO_EDGE,
            gl::RGBA,
            Some(&[255u8; 4]),
        );
        Ok(Sprites {
            plain,
            spun,
            white,
            textures: Vec::new(),
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
            gl::ActiveTexture(gl::TEXTURE0);
        }
        let mut clipped = false;
        let mut spinning = None;
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
            let spins = yaw != 0.0;
            let program = if spins { &self.spun } else { &self.plain };
            unsafe {
                if spinning != Some(spins) {
                    gl::UseProgram(program.prog);
                    spinning = Some(spins);
                }
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
                gl::Uniform4f(program.u_rect, x, y, w, h);
                gl::Uniform2f(program.u_turn, cos, sin);
                if spins {
                    let (yaw_sin, yaw_cos) = yaw.sin_cos();
                    gl::Uniform3f(program.u_spin, yaw_sin, yaw_cos, depth);
                }
                gl::Uniform4f(program.u_colour, colour[0], colour[1], colour[2], colour[3]);
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
            gl::DeleteProgram(self.plain.prog);
            gl::DeleteProgram(self.spun.prog);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEX: TexId = TexId(3);

    #[test]
    fn a_shifted_rect_tex_or_turned_face_moves_and_keeps_its_size() {
        let rect = Draw::Rect {
            x: 10.0,
            y: 20.0,
            w: 30.0,
            h: 40.0,
            colour: [0.1, 0.2, 0.3, 0.4],
        };
        assert_eq!(
            rect.shifted(4.0, -6.0),
            Draw::Rect {
                x: 14.0,
                y: 14.0,
                w: 30.0,
                h: 40.0,
                colour: [0.1, 0.2, 0.3, 0.4],
            }
        );
        let tex = |x, y| Draw::Tex {
            x,
            y,
            w: 30.0,
            h: 40.0,
            tex: TEX,
            alpha: 0.5,
        };
        assert_eq!(tex(10.0, 20.0).shifted(4.0, -6.0), tex(14.0, 14.0));
        let turned = |x, y| Draw::Turned {
            x,
            y,
            w: 30.0,
            h: 40.0,
            tex: TEX,
            alpha: 0.5,
            turn: 0.25,
        };
        assert_eq!(turned(10.0, 20.0).shifted(4.0, -6.0), turned(14.0, 14.0));
    }

    #[test]
    fn a_shifted_spun_face_takes_its_clip_with_it() {
        let spun = |x, y, clip| Draw::Spun {
            x,
            y,
            w: 30.0,
            h: 40.0,
            tex: TEX,
            colour: [1.0; 4],
            turn: 0.1,
            yaw: 0.2,
            depth: 5.0,
            clip,
        };
        assert_eq!(
            spun(10.0, 20.0, Some([0.0, 8.0, 100.0, 50.0])).shifted(2.0, 3.0),
            spun(12.0, 23.0, Some([2.0, 11.0, 100.0, 50.0]))
        );
        assert_eq!(
            spun(10.0, 20.0, None).shifted(2.0, 3.0),
            spun(12.0, 23.0, None)
        );
    }

    #[test]
    fn the_game_and_a_shot_stay_put() {
        assert_eq!(Draw::Game.shifted(5.0, 5.0), Draw::Game);
        let shot = Draw::Shot { tex: TEX };
        assert_eq!(shot.shifted(5.0, 5.0), shot);
    }
}
