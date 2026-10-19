//! Native handheld effects at viewport resolution, with the original frame available
//! to Pixel Transparency's backing tint and shadows. Targets are reused between frames.
use crate::power::{screen_brightness, screen_glow, screen_rect_in};
use crate::{pipeline::GameView, quad::Quad, surface::GfxError};

const VERT: &str = r#"
attribute vec2 a_pos;
uniform vec2 TextureSize, OrigTextureSize, OutputSize;
uniform vec4 u_rect, u_crop;
uniform vec2 u_target;
uniform float u_direct;
varying vec4 TEX0;
varying vec2 texel, orig_texel, orig_coord, source_coord;
varying float shadow_scale_factor;
void main() {
    TEX0=vec4(a_pos,0.0,1.0);
    texel=1.0/TextureSize; orig_texel=1.0/OrigTextureSize;
    orig_coord=a_pos;
    source_coord=u_crop.xy+a_pos*u_crop.zw;
    shadow_scale_factor=sqrt(OutputSize.x/640.0*OutputSize.y/480.0);
    vec2 p=(u_rect.xy+a_pos*u_rect.zw)/u_target;
    gl_Position=u_direct>0.5 ? vec4(p.x*2.0-1.0,1.0-p.y*2.0,0.0,1.0) : vec4(a_pos*2.0-1.0,0.0,1.0);
}
"#;
const COPY: &str = r#"
#ifdef GL_ES
precision highp float;
#endif
uniform sampler2D u_game, u_grid_mask;
uniform float u_bright, u_glow, u_saturation, u_grid_opacity;
varying vec2 v_uv;
void main() {
    vec3 rgb=texture2D(u_game,v_uv).rgb;
    if (u_grid_opacity>0.0) rgb*=vec3(1.0)-u_grid_opacity*(vec3(1.0)-texture2D(u_grid_mask,v_uv).rgb);
    rgb=mix(vec3(dot(rgb,vec3(0.299,0.587,0.114))),rgb,u_saturation);
    FRAG_COLOR=vec4(mix(rgb*u_bright,vec3(1.0),u_glow),1.0);
}
"#;

pub(crate) struct Presets {
    programs: [Option<u32>; 7],
    failed: [bool; 7],
    copy: u32,
    neutral: u32,
    targets: [(u32, u32); 2],
    size: (u32, u32),
    pub shadow: (f32, f32),
    pub saturation: f32,
    pub palette: u8,
    pub lcd_darkness: f32,
    pub antialias: bool,
    mask: crate::lcd_mask::Mask,
}
fn loc(p: u32, n: &str) -> i32 {
    crate::gl::uniform_location(p, n)
}
impl Presets {
    pub fn new() -> Result<Self, GfxError> {
        Ok(Self {
            programs: [None; 7],
            failed: [false; 7],
            copy: crate::shaders::program(crate::shaders::RECT_VERT, COPY)?,
            neutral: crate::gl::texture(
                1,
                1,
                gl::NEAREST,
                gl::CLAMP_TO_EDGE,
                gl::RGBA,
                Some(&[255; 4]),
            ),
            targets: [(0, 0); 2],
            size: (0, 0),
            shadow: (1.0, 1.0),
            saturation: 1.0,
            palette: 0,
            lcd_darkness: 1.0,
            antialias: false,
            mask: crate::lcd_mask::Mask::new(),
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
            8 => &[4],
            9 => &[5],
            10 => &[6],
            _ => &[3],
        };
        // Compile only the programs in the selected preset, once per GL context.
        // The selector and Clean mode do not pay for unused grid or shadow programs.
        let sources = [
            include_str!("../shaders/lcd1x.frag"),
            include_str!("../shaders/lcd3x.frag"),
            include_str!("../shaders/pixel-grid.frag"),
            include_str!("../shaders/pixel_transparency.frag"),
            include_str!("../shaders/crt-scanlines.frag"),
            include_str!("../shaders/zfast-crt.frag"),
            include_str!("../shaders/crt-pi.frag"),
        ];
        for &index in chain {
            if self.failed[index] {
                return false;
            }
            if self.programs[index].is_none() {
                match crate::shaders::program(
                    VERT,
                    &format!(
                        "{}\n{}\n{}",
                        crate::shaders::GAME_PALETTE,
                        include_str!("../shaders/sampling.glsl"),
                        sources[index]
                    ),
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
        let orig = (source.0 as f32 * view.uv[2], source.1 as f32 * view.uv[3]);
        let screen_grid = matches!(mode, 3 | 7);
        let raster = if screen_grid {
            (size.0 as f32 / 3.0, size.1 as f32 / 3.0)
        } else {
            orig
        };
        let origin = if screen_grid {
            [view.rect[0] / 3.0, view.rect[1] / 3.0]
        } else {
            [source.0 as f32 * view.uv[0], source.1 as f32 * view.uv[1]]
        };
        let kind = match mode {
            1 | 5 => 0,
            2 | 6 => 1,
            _ => 2,
        };
        let direct = mode <= 3 || mode >= 8;
        let key = crate::lcd_mask::Key {
            size,
            raster: [raster.0.to_bits(), raster.1.to_bits()],
            origin: origin.map(f32::to_bits),
            kind,
            source_grid: false,
        };
        unsafe {
            gl::ActiveTexture(gl::TEXTURE2);
        }
        let mask = if mode == 4 || mode >= 8 {
            self.neutral
        } else {
            self.mask.get(key, quad)
        };
        unsafe {
            let mut fbo = 0;
            let mut viewport = [0; 4];
            gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fbo);
            gl::GetIntegerv(gl::VIEWPORT, viewport.as_mut_ptr());
            let blend = gl::IsEnabled(gl::BLEND) != 0;
            gl::Disable(gl::BLEND);
            if !direct {
                self.resize(size);
            }
            let orig = (source.0 as f32 * view.uv[2], source.1 as f32 * view.uv[3]);
            let mut input = original;
            for (pass, &index) in chain.iter().enumerate() {
                let p = self.programs[index].expect("selected shader compiled");
                gl::UseProgram(p);
                if !direct {
                    gl::BindFramebuffer(gl::FRAMEBUFFER, self.targets[pass].0);
                    gl::Viewport(0, 0, size.0 as i32, size.1 as i32);
                }
                let (x, y, w, h) = screen_rect_in(power, view.rect);
                gl::Uniform4f(loc(p, "u_rect"), x, y, w, h);
                gl::Uniform2f(
                    loc(p, "u_target"),
                    crate::out_w() as f32,
                    crate::out_h() as f32,
                );
                gl::Uniform1f(
                    loc(p, "u_aa"),
                    if self.antialias || mode >= 8 {
                        1.0
                    } else {
                        0.0
                    },
                );
                let sample_size = if pass == 0 {
                    (source.0 as f32, source.1 as f32)
                } else {
                    (size.0 as f32, size.1 as f32)
                };
                let sample_scale = if pass == 0 {
                    (size.0 as f32 / orig.0, size.1 as f32 / orig.1)
                } else {
                    (1.0, 1.0)
                };
                gl::Uniform2f(loc(p, "u_sample_size"), sample_size.0, sample_size.1);
                gl::Uniform2f(loc(p, "u_sample_scale"), sample_scale.0, sample_scale.1);
                gl::Uniform2f(loc(p, "u_original_size"), source.0 as f32, source.1 as f32);
                gl::Uniform2f(
                    loc(p, "u_original_scale"),
                    size.0 as f32 / orig.0,
                    size.1 as f32 / orig.1,
                );
                gl::Uniform1f(loc(p, "u_direct"), if direct { 1.0 } else { 0.0 });
                gl::Uniform1f(loc(p, "u_post"), if direct { 1.0 } else { 0.0 });
                gl::Uniform1f(loc(p, "u_saturation"), self.saturation);
                gl::Uniform1f(loc(p, "u_bright"), screen_brightness(power));
                gl::Uniform1f(loc(p, "u_glow"), screen_glow(power));
                gl::Uniform1i(loc(p, "Mask"), 2);
                gl::ActiveTexture(gl::TEXTURE2);
                gl::BindTexture(gl::TEXTURE_2D, mask);
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
                gl::Uniform1f(
                    loc(p, "u_grid"),
                    if mode == 7 && self.lcd_darkness >= 1.0 {
                        1.0
                    } else {
                        0.0
                    },
                );
                let raster = if screen_grid {
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
                gl::Uniform1f(loc(p, "u_lcd_darkness"), self.lcd_darkness);
                gl::Uniform2f(loc(p, "FullSourceSize"), source.0 as f32, source.1 as f32);
                gl::Uniform1i(loc(p, "Texture"), 0);
                gl::Uniform1i(loc(p, "OrigTexture"), 1);
                gl::Uniform1i(loc(p, "FrameDirection"), 1);
                gl::Uniform1f(loc(p, "PT_SHADOW_OFFSET_X"), self.shadow.0);
                gl::Uniform1f(loc(p, "PT_SHADOW_OFFSET_Y"), self.shadow.1);
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, input);
                gl::ActiveTexture(gl::TEXTURE1);
                gl::BindTexture(gl::TEXTURE_2D, original);
                gl::ActiveTexture(gl::TEXTURE0);
                let filter = if self.antialias || mode >= 8 {
                    gl::LINEAR
                } else {
                    gl::NEAREST
                };
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filter as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filter as i32);
                quad.draw();
                if direct {
                    if blend {
                        gl::Enable(gl::BLEND);
                    }
                    return true;
                }
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
            // Partial Grid opacity is applied to the visible RGBA8 PT result, so it
            // passes exactly that fraction of the colour without altering PT inputs.
            gl::Uniform1i(loc(p, "u_grid_mask"), 2);
            gl::Uniform1f(
                loc(p, "u_grid_opacity"),
                if mode == 7 && self.lcd_darkness < 1.0 {
                    self.lcd_darkness
                } else {
                    0.0
                },
            );
            gl::ActiveTexture(gl::TEXTURE2);
            gl::BindTexture(gl::TEXTURE_2D, mask);
            gl::Uniform1i(loc(p, "u_game"), 0);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, input);
            quad.draw();
        }
        true
    }
    #[cfg(feature = "test-support")]
    pub fn reference(
        &self,
        mode: u8,
        original: u32,
        source: (u32, u32),
        view: GameView,
        quad: &Quad,
    ) {
        let vertex = r#"
attribute vec2 a_pos;
uniform vec4 u_rect, u_crop;
uniform vec2 u_target;
varying vec4 TEX0;
void main() {
    TEX0 = vec4(u_crop.xy + a_pos * u_crop.zw, 0.0, 1.0);
    vec2 p = (u_rect.xy + a_pos * u_rect.zw) / u_target;
    gl_Position = vec4(p.x*2.0-1.0, 1.0-p.y*2.0, 0.0, 1.0);
}
"#;
        let fragment = if mode == 1 {
            include_str!("../shaders/lcd1x.upstream.glsl")
        } else {
            include_str!("../shaders/lcd3x.upstream.glsl")
        };
        let p = crate::shaders::program(vertex, &format!("#define FRAGMENT\n{fragment}"))
            .expect("upstream LCD compiles");
        unsafe {
            gl::UseProgram(p);
            gl::Uniform4f(
                loc(p, "u_rect"),
                view.rect[0],
                view.rect[1],
                view.rect[2],
                view.rect[3],
            );
            gl::Uniform4f(
                loc(p, "u_crop"),
                view.uv[0],
                view.uv[1],
                view.uv[2],
                view.uv[3],
            );
            gl::Uniform2f(
                loc(p, "u_target"),
                crate::out_w() as f32,
                crate::out_h() as f32,
            );
            for n in ["OrigInputSize", "TextureSize", "InputSize"] {
                gl::Uniform2f(loc(p, n), source.0 as f32, source.1 as f32);
            }
            gl::Uniform2f(loc(p, "OutputSize"), view.rect[2], view.rect[3]);
            gl::Uniform1i(loc(p, "Texture"), 0);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, original);
            quad.draw();
            gl::DeleteProgram(p);
        }
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
            gl::DeleteTextures(1, &self.neutral);
        }
    }
}
