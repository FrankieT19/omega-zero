// Physical panel grid: 2x2 lit pixels and a black row/column in every 3x3 cell.
#ifdef GL_ES
precision highp float;
#endif
uniform sampler2D Texture;
uniform vec4 u_crop;
varying vec4 TEX0;
void main() {
    vec2 cell = mod(floor(gl_FragCoord.xy), 3.0);
    if (cell.x > 1.5 || cell.y > 1.5) { FRAG_COLOR=vec4(0.0,0.0,0.0,1.0); return; }
    FRAG_COLOR=vec4(palette_rgb(texture2D(Texture,u_crop.xy+TEX0.xy*u_crop.zw).rgb,u_palette),1.0);
}
