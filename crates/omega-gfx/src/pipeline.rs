use crate::canvas::{out_h, out_w};
use crate::lcd3x::mask_texture_rgba8;
use crate::power::{screen_brightness, screen_glow, screen_rect_in};
use crate::quad::Quad;
use crate::shaders::{GAME_FRAG, GAME_SHARP_FRAG, RECT_VERT};
use crate::surface::GfxError;

/// The whole-number scale the LCD mask is drawn for, three panel pixels to a game pixel.
pub const SCALE: u32 = 3;
/// The game texture, a GBA frame. Game Boy pictures arrive centred inside it.
pub const SRC_W: u32 = 240;
pub const SRC_H: u32 = 160;

pub const WHOLE_TEXTURE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Filter {
    /// Three panel pixels to a game pixel, through the LCD mask.
    Lcd3x,
    /// Any other scale, each game pixel kept square with a one pixel blend where two meet.
    Sharp,
}

/// How the game is laid on the canvas: the part of the game texture shown, the rect it fills
/// with the screen on, and how it is scaled into it.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct GameView {
    pub rect: [f32; 4],
    pub uv: [f32; 4],
    pub filter: Filter,
}

impl GameView {
    /// `uv` of the game texture over the whole canvas, through the LCD mask: the only view
    /// there was on 720x480.
    pub fn canvas(uv: [f32; 4]) -> GameView {
        GameView {
            rect: [0.0, 0.0, out_w() as f32, out_h() as f32],
            uv,
            filter: Filter::Lcd3x,
        }
    }
}

struct Lcd {
    prog: gl::types::GLuint,
    u_rect: gl::types::GLint,
    u_bright: gl::types::GLint,
    u_glow: gl::types::GLint,
    u_uv: gl::types::GLint,
    u_src: gl::types::GLint,
}

struct Sharp {
    prog: gl::types::GLuint,
    u_rect: gl::types::GLint,
    u_bright: gl::types::GLint,
    u_glow: gl::types::GLint,
    u_uv: gl::types::GLint,
    u_scale: gl::types::GLint,
}

pub struct GamePass {
    lcd: Lcd,
    sharp: Sharp,
    game: gl::types::GLuint,
    mask: gl::types::GLuint,
    power: f32,
    view: Option<GameView>,
    still: Option<GameView>,
}

fn uniform(prog: gl::types::GLuint, name: &str) -> gl::types::GLint {
    crate::gl::uniform_location(prog, name)
}

impl GamePass {
    pub fn new() -> Result<Self, GfxError> {
        let lcd = crate::shaders::program(RECT_VERT, GAME_FRAG)?;
        let sharp = crate::shaders::program(RECT_VERT, GAME_SHARP_FRAG)?;
        let game = crate::gl::texture(SRC_W, SRC_H, gl::NEAREST, gl::CLAMP_TO_EDGE, gl::BGRA, None);
        let mask = crate::gl::texture(
            3,
            3,
            gl::NEAREST,
            gl::REPEAT,
            gl::RGBA,
            Some(&mask_texture_rgba8()),
        );
        unsafe {
            gl::UseProgram(lcd);
            gl::Uniform1i(uniform(lcd, "u_game"), 0);
            gl::Uniform1i(uniform(lcd, "u_mask"), 1);
            gl::Uniform2f(uniform(lcd, "u_target"), out_w() as f32, out_h() as f32);
            gl::UseProgram(sharp);
            gl::Uniform1i(uniform(sharp, "u_game"), 0);
            gl::Uniform2f(uniform(sharp, "u_src"), SRC_W as f32, SRC_H as f32);
            gl::Uniform2f(uniform(sharp, "u_target"), out_w() as f32, out_h() as f32);
        }
        Ok(GamePass {
            lcd: Lcd {
                prog: lcd,
                u_rect: uniform(lcd, "u_rect"),
                u_bright: uniform(lcd, "u_bright"),
                u_glow: uniform(lcd, "u_glow"),
                u_uv: uniform(lcd, "u_uv"),
                u_src: uniform(lcd, "u_src"),
            },
            sharp: Sharp {
                prog: sharp,
                u_rect: uniform(sharp, "u_rect"),
                u_bright: uniform(sharp, "u_bright"),
                u_glow: uniform(sharp, "u_glow"),
                u_uv: uniform(sharp, "u_uv"),
                u_scale: uniform(sharp, "u_scale"),
            },
            game,
            mask,
            power: 1.0,
            view: None,
            still: None,
        })
    }

    pub fn set_power(&mut self, t: f32) {
        self.power = t.clamp(0.0, 1.0);
    }

    /// Shows `rect` of the game texture over the whole canvas, as omega-zero did before a canvas
    /// could be any other shape.
    pub fn set_source_rect(&mut self, rect: [f32; 4]) {
        self.view = Some(GameView::canvas(rect));
    }

    pub fn set_view(&mut self, view: GameView) {
        self.view = Some(view);
    }

    /// The view a still, a save state's screenshot, is shown in.
    pub fn set_still_view(&mut self, view: GameView) {
        self.still = Some(view);
    }

    pub fn upload(&mut self, xrgb8888: &[u8]) {
        if xrgb8888.len() < (SRC_W * SRC_H * 4) as usize {
            return;
        }
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.game);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                0,
                0,
                SRC_W as i32,
                SRC_H as i32,
                gl::BGRA,
                gl::UNSIGNED_BYTE,
                xrgb8888.as_ptr() as *const std::ffi::c_void,
            );
        }
    }

    pub fn draw(&self, quad: &Quad) {
        let view = self.view.unwrap_or_else(|| GameView::canvas(WHOLE_TEXTURE));
        self.draw_source(self.game, quad, view);
    }

    pub fn draw_still(&self, tex: gl::types::GLuint, quad: &Quad) {
        let view = self
            .still
            .unwrap_or_else(|| GameView::canvas(WHOLE_TEXTURE));
        self.draw_source(tex, quad, view);
    }

    fn draw_source(&self, tex: gl::types::GLuint, quad: &Quad, view: GameView) {
        let (x, y, w, h) = screen_rect_in(self.power, view.rect);
        let uv = view.uv;
        match view.filter {
            Filter::Lcd3x => unsafe {
                gl::UseProgram(self.lcd.prog);
                gl::Uniform4f(self.lcd.u_rect, x, y, w, h);
                gl::Uniform4f(self.lcd.u_uv, uv[0], uv[1], uv[2], uv[3]);
                gl::Uniform2f(
                    self.lcd.u_src,
                    view.rect[2] / SCALE as f32,
                    view.rect[3] / SCALE as f32,
                );
                gl::Uniform1f(self.lcd.u_bright, screen_brightness(self.power));
                gl::Uniform1f(self.lcd.u_glow, screen_glow(self.power));
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, tex);
                gl::ActiveTexture(gl::TEXTURE1);
                gl::BindTexture(gl::TEXTURE_2D, self.mask);
                gl::ActiveTexture(gl::TEXTURE0);
            },
            Filter::Sharp => unsafe {
                gl::UseProgram(self.sharp.prog);
                gl::Uniform4f(self.sharp.u_rect, x, y, w, h);
                gl::Uniform4f(self.sharp.u_uv, uv[0], uv[1], uv[2], uv[3]);
                gl::Uniform2f(
                    self.sharp.u_scale,
                    view.rect[2] / (uv[2] * SRC_W as f32),
                    view.rect[3] / (uv[3] * SRC_H as f32),
                );
                gl::Uniform1f(self.sharp.u_bright, screen_brightness(self.power));
                gl::Uniform1f(self.sharp.u_glow, screen_glow(self.power));
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, tex);
                filter(gl::LINEAR);
            },
        }
        quad.draw();
        if view.filter == Filter::Sharp {
            unsafe { filter(gl::NEAREST) };
        }
    }
}

/// GLES2 has no sampler objects, so the sharp view sets the bound texture's own filter for
/// its draw and puts it back after.
unsafe fn filter(f: gl::types::GLenum) {
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, f as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, f as i32);
}

impl Drop for GamePass {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.game);
            gl::DeleteTextures(1, &self.mask);
            gl::DeleteProgram(self.lcd.prog);
            gl::DeleteProgram(self.sharp.prog);
        }
    }
}
