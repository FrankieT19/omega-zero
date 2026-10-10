use crate::surface::GfxError;

const VERT_PREAMBLE: &str = "#version 330 core\n#define attribute in\n#define varying out\n";

const FRAG_PREAMBLE: &str = "#version 330 core\n#define varying in\n\
                             #define texture2D texture\nout vec4 FRAG_COLOR;\n";

const VERT_PREAMBLE_ES: &str = "";
const FRAG_PREAMBLE_ES: &str = "#define FRAG_COLOR gl_FragColor\n";

pub fn program(vert: &str, frag: &str) -> Result<gl::types::GLuint, GfxError> {
    let (vp, fp) = match crate::gl::es() {
        true => (VERT_PREAMBLE_ES, FRAG_PREAMBLE_ES),
        false => (VERT_PREAMBLE, FRAG_PREAMBLE),
    };
    crate::gl::program(&format!("{vp}{vert}"), &format!("{fp}{frag}"))
}

pub const RECT_VERT: &str = r#"
attribute vec2 a_pos;
uniform vec4 u_rect;
uniform vec2 u_target;
varying vec2 v_uv;
void main() {
    v_uv = a_pos;
    vec2 p = (u_rect.xy + a_pos * u_rect.zw) / u_target;
    gl_Position = vec4(p.x * 2.0 - 1.0, 1.0 - p.y * 2.0, 0.0, 1.0);
}
"#;

pub const SPRITE_VERT: &str = r#"
attribute vec2 a_pos;
uniform vec4 u_rect;
uniform vec2 u_target;
uniform vec2 u_turn;
varying vec2 v_uv;
void main() {
    v_uv = a_pos;
    vec2 mid = u_rect.zw * 0.5;
    vec2 local = a_pos * u_rect.zw - mid;
    vec2 turned = vec2(u_turn.x * local.x - u_turn.y * local.y,
                       u_turn.y * local.x + u_turn.x * local.y);
    vec2 p = (u_rect.xy + a_pos * u_rect.zw + (turned - local)) / u_target;
    gl_Position = vec4(p.x * 2.0 - 1.0, 1.0 - p.y * 2.0, 0.0, 1.0);
}
"#;

/// A sprite turned like `SPRITE_VERT`, then spun about its upright middle by `u_spin` (sin, cos,
/// depth) and seen from `u_focal` pixels away. Its own program, so the sprites that never turn
/// pay nothing for it. The spin is divided through on the vertex, so the clipper only ever sees
/// w = 1: some drivers drop a triangle that crosses the edge of the screen with any other w. For
/// the texture to keep true perspective, `v_uvq` carries uv/w and 1/w for `SPUN_FRAG` to divide.
pub const SPUN_VERT: &str = r#"
attribute vec2 a_pos;
uniform vec4 u_rect;
uniform vec2 u_target;
uniform vec2 u_turn;
uniform vec3 u_spin;
uniform float u_focal;
varying vec3 v_uvq;
void main() {
    vec2 mid = u_rect.zw * 0.5;
    vec2 local = a_pos * u_rect.zw - mid;
    vec2 turned = vec2(u_turn.x * local.x - u_turn.y * local.y,
                       u_turn.y * local.x + u_turn.x * local.y);
    float across = turned.x * u_spin.y - u_spin.z * u_spin.x;
    float w = (u_focal + turned.x * u_spin.x) / u_focal;
    v_uvq = vec3(a_pos, 1.0) / w;
    vec2 spun = vec2(across, turned.y) / w;
    vec2 p = (u_rect.xy + a_pos * u_rect.zw + (turned - local) + (spun - turned)) / u_target;
    gl_Position = vec4(p.x * 2.0 - 1.0, 1.0 - p.y * 2.0, 0.0, 1.0);
}
"#;

pub const GAME_FRAG: &str = r#"
precision mediump float;
uniform sampler2D u_game;
uniform sampler2D u_mask;
uniform vec2 u_src;
uniform vec4 u_uv;
uniform float u_bright;
varying vec2 v_uv;
void main() {
    vec2 uv = u_uv.xy + v_uv * u_uv.zw;
    vec3 rgb = texture2D(u_game, uv).rgb * texture2D(u_mask, v_uv * u_src).rgb;
    FRAG_COLOR = vec4(rgb * u_bright, 1.0);
}
"#;

/// The game at a scale that is not a whole number: each game pixel stays a sharp square, and
/// only the one panel pixel where two meet is blended, so nothing shimmers as it scrolls.
pub const GAME_SHARP_FRAG: &str = r#"
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
uniform sampler2D u_game;
uniform vec2 u_src;
uniform vec4 u_uv;
uniform vec2 u_scale;
uniform float u_bright;
varying vec2 v_uv;
void main() {
    vec2 texel = (u_uv.xy + v_uv * u_uv.zw) * u_src;
    vec2 from_centre = fract(texel) - 0.5;
    vec2 region = 0.5 - 0.5 / u_scale;
    vec2 f = (from_centre - clamp(from_centre, -region, region)) * u_scale + 0.5;
    vec3 rgb = texture2D(u_game, (floor(texel) + f) / u_src).rgb;
    FRAG_COLOR = vec4(rgb * u_bright, 1.0);
}
"#;

pub const SPRITE_FRAG: &str = r#"
precision mediump float;
uniform sampler2D u_tex;
uniform vec4 u_colour;
varying vec2 v_uv;
void main() {
    FRAG_COLOR = texture2D(u_tex, v_uv) * u_colour;
}
"#;

pub const SPUN_FRAG: &str = r#"
precision mediump float;
uniform sampler2D u_tex;
uniform vec4 u_colour;
varying vec3 v_uvq;
void main() {
    FRAG_COLOR = texture2D(u_tex, v_uvq.xy / v_uvq.z) * u_colour;
}
"#;

pub const BLIT_VERT: &str = r#"
attribute vec2 a_pos;
varying vec2 v_uv;
void main() {
    v_uv = a_pos;
    gl_Position = vec4(a_pos * 2.0 - 1.0, 0.0, 1.0);
}
"#;

/// The blit for a panel mounted on its side: `u_turn` takes each viewport point to the point
/// of the upright picture that belongs there.
pub const BLIT_VERT_TURNED: &str = r#"
attribute vec2 a_pos;
uniform mat2 u_turn;
varying vec2 v_uv;
void main() {
    v_uv = u_turn * (a_pos - 0.5) + 0.5;
    gl_Position = vec4(a_pos * 2.0 - 1.0, 0.0, 1.0);
}
"#;

pub const BLIT_FRAG: &str = r#"
precision mediump float;
uniform sampler2D u_tex;
uniform vec3 u_gain;
varying vec2 v_uv;
void main() {
    FRAG_COLOR = vec4(texture2D(u_tex, v_uv).rgb * u_gain, 1.0);
}
"#;
