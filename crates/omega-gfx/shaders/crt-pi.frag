/* Adapted from crt-pi.glsl, Copyright (C) 2015-2016 davej.
   GPL-2.0-or-later. See crt-pi.upstream.glsl.
   Flat geometry (upstream CURVATURE disabled), gamma, multisampled scanlines,
   green/magenta mask and bloom. Native crop, pixel-aligned scanline phase and darkness control added.
*/
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
float line_weight(float d) { return max(1.0-d*d*6.0,0.12); }
void main() {
    vec2 p=source_coord*FullSourceSize;
    float centre=floor(p.y)+0.5;
    float dy=p.y-centre;
    float width=1.0/(max(u_sample_scale.y,1.0)*3.0);
    float beam=fract(crt_row()+0.5/max(u_sample_scale.y,1.0))-0.5;
    float scan=(line_weight(beam)+line_weight(beam-width)+line_weight(beam+width))/3.0;
    float sy=sign(dy);
    float q=dy*dy; q=q*q;
    vec2 tc=vec2(source_coord.x,(centre+q*8.0*sy)/FullSourceSize.y);
    vec3 colour=texture2D(Texture,tc).rgb;
    vec3 crt=pow(pow(colour,vec3(2.4))*max(0.0,mix(1.0,scan*1.5,u_lcd_darkness)),vec3(1.0/2.2));
    vec3 mask=fract(gl_FragCoord.x*0.5)<0.5?vec3(0.7,1.0,0.7):vec3(1.0,0.7,1.0);
    crt=mix(colour,crt,clamp(u_sample_scale.y-1.0,0.0,1.0));
    colour=crt*mask;
    FRAG_COLOR=vec4(finish_crt(colour),1.0);
}
