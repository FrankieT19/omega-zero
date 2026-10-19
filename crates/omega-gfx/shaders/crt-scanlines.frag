// Flat scanlines with sharp bilinear scaling. No mask, curvature or extra pass.
// Scanline attenuation is integrated across the physical pixel to limit moire.
#ifdef GL_ES
precision highp float;
#endif
uniform sampler2D Texture;
uniform vec2 FullSourceSize, OutputSize;
uniform float u_lcd_darkness, u_saturation, u_bright, u_glow;
varying vec2 source_coord;
uniform vec2 u_crt_origin, u_crt_rows;
float crt_row() { return u_crt_rows.x+(floor(u_crt_origin.y-gl_FragCoord.y-u_crt_origin.x)+0.5)*u_crt_rows.y; }
vec3 finish_crt(vec3 rgb) {
    rgb=mix(vec3(dot(rgb,vec3(0.299,0.587,0.114))),rgb,u_saturation);
    return mix(rgb*u_bright,vec3(1.0),u_glow);
}
void main() {
    vec3 rgb=game_sample(Texture,source_coord).rgb;
    float phase=6.28318530718*(crt_row()-0.5+0.5/max(u_sample_scale.y,1.0));
    float half_width=3.14159265359/max(u_sample_scale.y,1.0);
    float attenuation=sin(half_width)/half_width;
    float scan=0.8+0.2*cos(phase)*attenuation;
    rgb*=max(0.0,1.0-u_lcd_darkness*(1.0-scan));
    FRAG_COLOR=vec4(finish_crt(rgb),1.0);
}
