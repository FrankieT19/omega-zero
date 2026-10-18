//! RetroArch handheld presets at viewport resolution, with the original frame available
//! to Pixel Transparency's backing tint and shadows. Targets are reused between frames.
use crate::power::{screen_brightness, screen_glow, screen_rect_in};
use crate::{pipeline::GameView, quad::Quad, surface::GfxError};

const VERT: &str = r#"
attribute vec2 a_pos;
uniform vec2 TextureSize, OrigTextureSize, OutputSize;
varying vec4 TEX0;
varying vec2 texel, orig_texel, orig_coord;
varying float shadow_scale_factor;
void main() {
    TEX0=vec4(a_pos,0.0,1.0);
    texel=1.0/TextureSize; orig_texel=1.0/OrigTextureSize;
    orig_coord=a_pos;
    shadow_scale_factor=sqrt(OutputSize.x/640.0*OutputSize.y/480.0);
    gl_Position=vec4(a_pos*2.0-1.0,0.0,1.0);
}
"#;
const COPY: &str = r#"
#ifdef GL_ES
precision mediump float;
#endif
uniform sampler2D u_game;
uniform float u_bright, u_glow, u_saturation;
varying vec2 v_uv;
void main() {
    vec3 rgb=texture2D(u_game,v_uv).rgb;
    rgb=mix(vec3(dot(rgb,vec3(0.299,0.587,0.114))),rgb,u_saturation);
    FRAG_COLOR=vec4(mix(rgb*u_bright,vec3(1.0),u_glow),1.0);
}
"#;

pub(crate) struct Presets {
    programs: [Option<u32>; 4],
    failed: [bool; 4],
    copy: u32,
    targets: [(u32, u32); 2],
    size: (u32, u32),
    pub shadow: (f32, f32),
    pub panel_raster: bool,
    pub saturation: f32,
    pub palette: u8,
}
fn loc(p: u32, n: &str) -> i32 {
    crate::gl::uniform_location(p, n)
}
impl Presets {
    pub fn new() -> Result<Self, GfxError> {
        Ok(Self {
            programs: [None; 4],
            failed: [false; 4],
            copy: crate::shaders::program(crate::shaders::RECT_VERT, COPY)?,
            targets: [(0, 0); 2],
            size: (0, 0),
            shadow: (1.0, 1.0),
            panel_raster: false,
            saturation: 1.0,
            palette: 0,
        })
    }
    fn resize(&mut self, size: (u32, u32)) {
        if self.size == size {
            return;
        }
        self.clear();
        self.size = size;
        unsafe {
            for (fbo, tex) in &mut self.targets {
                *tex = crate::gl::texture(
                    size.0,
                    size.1,
                    gl::NEAREST,
                    gl::CLAMP_TO_EDGE,
                    gl::RGBA,
                    None,
                );
                gl::GenFramebuffers(1, fbo);
                gl::BindFramebuffer(gl::FRAMEBUFFER, *fbo);
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gl::TEXTURE_2D,
                    *tex,
                    0,
                );
            }
        }
    }
    pub fn draw(
        &mut self,
        mode: u8,
        original: u32,
        source: (u32, u32),
        view: GameView,
        power: f32,
        quad: &Quad,
    ) -> bool {
        let chain: &[usize] = match mode {
            1 => &[0],
            2 => &[1],
            3 => &[2],
            4 => &[3],
            5 => &[0, 3],
            6 => &[1, 3],
            _ => &[3],
        };
        // Compile only the programs in the selected preset, once per GL context.
        // The selector and Clean mode do not pay for unused grid or shadow programs.
        let sources = [
            include_str!("../shaders/lcd1x.frag"),
            include_str!("../shaders/lcd3x.frag"),
            include_str!("../shaders/pixel-grid.frag"),
            include_str!("../shaders/pixel_transparency.frag"),
        ];
        for &index in chain {
            if self.failed[index] {
                return false;
            }
            if self.programs[index].is_none() {
                match crate::shaders::program(
                    VERT,
                    &format!("{}\n{}", crate::shaders::GAME_PALETTE, sources[index]),
                ) {
                    Ok(p) => self.programs[index] = Some(p),
                    Err(e) => {
                        eprintln!("omega-zero: shader: {e}");
                        self.failed[index] = true;
                        return false;
                    }
                }
            }
        }
        let size = (
            view.rect[2].round().max(1.0) as u32,
            view.rect[3].round().max(1.0) as u32,
        );
        unsafe {
            let mut fbo = 0;
            let mut viewport = [0; 4];
            gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fbo);
            gl::GetIntegerv(gl::VIEWPORT, viewport.as_mut_ptr());
            let blend = gl::IsEnabled(gl::BLEND) != 0;
            gl::Disable(gl::BLEND);
            self.resize(size);
            let orig = (source.0 as f32 * view.uv[2], source.1 as f32 * view.uv[3]);
            let mut input = original;
            for (pass, &index) in chain.iter().enumerate() {
                let p = self.programs[index].expect("selected shader compiled");
                gl::UseProgram(p);
                gl::BindFramebuffer(gl::FRAMEBUFFER, self.targets[pass].0);
                gl::Viewport(0, 0, size.0 as i32, size.1 as i32);
                let crop = if pass == 0 {
                    view.uv
                } else {
                    [0.0, 0.0, 1.0, 1.0]
                };
                let input_size = if pass == 0 {
                    orig
                } else {
                    (size.0 as f32, size.1 as f32)
                };
                for name in ["InputSize", "TextureSize"] {
                    gl::Uniform2f(loc(p, name), input_size.0, input_size.1);
                }
                for name in ["OrigInputSize", "OrigTextureSize"] {
                    gl::Uniform2f(loc(p, name), orig.0, orig.1);
                }
                gl::Uniform1f(loc(p, "u_grid"), if mode == 7 { 1.0 } else { 0.0 });
                let raster = if self.panel_raster {
                    (size.0 as f32 / 3.0, size.1 as f32 / 3.0)
                } else {
                    orig
                };
                gl::Uniform2f(loc(p, "RasterSize"), raster.0, raster.1);
                gl::Uniform2f(loc(p, "OutputSize"), size.0 as f32, size.1 as f32);
                gl::Uniform4f(loc(p, "u_crop"), crop[0], crop[1], crop[2], crop[3]);
                gl::Uniform4f(
                    loc(p, "u_orig_crop"),
                    view.uv[0],
                    view.uv[1],
                    view.uv[2],
                    view.uv[3],
                );
                gl::Uniform1f(
                    loc(p, "u_palette"),
                    if pass == 0 { self.palette as f32 } else { 0.0 },
                );
                gl::Uniform1f(loc(p, "u_orig_palette"), self.palette as f32);
                gl::Uniform1i(loc(p, "Texture"), 0);
                gl::Uniform1i(loc(p, "OrigTexture"), 1);
                gl::Uniform1i(loc(p, "FrameDirection"), 1);
                gl::Uniform1f(loc(p, "PT_SHADOW_OFFSET_X"), self.shadow.0);
                gl::Uniform1f(loc(p, "PT_SHADOW_OFFSET_Y"), self.shadow.1);
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, input);
                gl::ActiveTexture(gl::TEXTURE1);
                gl::BindTexture(gl::TEXTURE_2D, original);
                quad.draw();
                input = self.targets[pass].1;
            }
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo as u32);
            gl::Viewport(viewport[0], viewport[1], viewport[2], viewport[3]);
            if blend {
                gl::Enable(gl::BLEND);
            }
            let p = self.copy;
            gl::UseProgram(p);
            let (x, y, w, h) = screen_rect_in(power, view.rect);
            gl::Uniform4f(loc(p, "u_rect"), x, y, w, h);
            gl::Uniform2f(
                loc(p, "u_target"),
                crate::out_w() as f32,
                crate::out_h() as f32,
            );
            gl::Uniform1f(loc(p, "u_saturation"), self.saturation);
            gl::Uniform1f(loc(p, "u_bright"), screen_brightness(power));
            gl::Uniform1f(loc(p, "u_glow"), screen_glow(power));
            gl::Uniform1i(loc(p, "u_game"), 0);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, input);
            quad.draw();
        }
        true
    }
    fn clear(&mut self) {
        unsafe {
            for (f, t) in &mut self.targets {
                gl::DeleteFramebuffers(1, f);
                gl::DeleteTextures(1, t);
                *f = 0;
                *t = 0;
            }
        }
    }
}
impl Drop for Presets {
    fn drop(&mut self) {
        self.clear();
        unsafe {
            for p in self.programs.iter().flatten() {
                gl::DeleteProgram(*p);
            }
            gl::DeleteProgram(self.copy);
        }
    }
}
