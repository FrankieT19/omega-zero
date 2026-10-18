// Dark final third of each source-pixel cell: exact 2x2 lit + black separators at 3x.
// GB/GBC Stretch overrides RasterSize to preserve a physical three-pixel panel grid.
#ifdef GL_ES
precision highp float;
#endif
uniform sampler2D Texture;
uniform vec2 RasterSize, OutputSize;
uniform vec4 u_crop;
varying vec4 TEX0;
void main() {
    vec2 cell = mod(gl_FragCoord.xy, OutputSize/RasterSize)/(OutputSize/RasterSize);
    if (cell.x >= 0.6666667 || cell.y >= 0.6666667) { FRAG_COLOR=vec4(0.0,0.0,0.0,1.0); return; }
    FRAG_COLOR=vec4(palette_rgb(texture2D(Texture,u_crop.xy+TEX0.xy*u_crop.zw).rgb,u_palette),1.0);
}
