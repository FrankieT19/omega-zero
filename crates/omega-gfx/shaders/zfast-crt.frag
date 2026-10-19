/* Adapted from zfast_crt.glsl, Copyright (C) 2017 Greg Hogan
   (SoltanGris42). GPL-2.0-or-later. See zfast-crt.upstream.glsl.
   Native geometry/crop, pixel-aligned scanline phase and darkness control; no curvature.
   Filtering, scanlines, brightness response and fine mask use upstream defaults.
*/
#ifdef GL_ES
precision highp float;
#endif
uniform sampler2D Texture;
uniform vec2 FullSourceSize, OutputSize;
uniform float u_lcd_darkness, u_saturation, u_bright, u_glow;
varying vec2 source_coord;
vec3 finish_crt(vec3 rgb) {
    rgb=mix(vec3(dot(rgb,vec3(0.299,0.587,0.114))),rgb,u_saturation);
    return mix(rgb*u_bright,vec3(1.0),u_glow);
}
void main() {
    vec2 p=source_coord*FullSourceSize;
    vec2 i=floor(p)+0.5;
    vec2 f=p-i;
    vec2 tc=(i+4.0*f*f*f)/FullSourceSize;
    tc.x=mix(tc.x,source_coord.x,0.45);
    vec3 colour=texture2D(Texture,tc).rgb;
    float beam=fract(p.y+0.5/max(u_sample_scale.y,1.0))-0.5;
    float y=beam*beam, yy=y*y;
    float mask=1.0-float(fract(floor(gl_FragCoord.x)*-0.5)<0.5)*0.25;
    float low=1.25-5.0*(y-2.05*yy);
    float high=1.0-10.0*(yy-2.8*yy*y);
    float weight=mix(low*mask,high,dot(colour,vec3(0.3333*0.8)));
    weight=mix(1.0,weight,clamp(u_sample_scale.y-1.0,0.0,1.0));
    colour*=max(0.0,1.0-u_lcd_darkness*(1.0-weight));
    FRAG_COLOR=vec4(finish_crt(colour),1.0);
}
