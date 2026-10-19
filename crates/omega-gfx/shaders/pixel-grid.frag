// Physical screen grid; darkness is its opacity.
#ifdef GL_ES
precision highp float;
#endif
uniform sampler2D Texture, Mask;
uniform vec4 u_crop;
uniform vec2 FullSourceSize;
uniform float u_lcd_darkness, u_post, u_saturation, u_bright, u_glow;
varying vec4 TEX0;
void main() {
    vec2 p=floor((u_crop.xy+TEX0.xy*u_crop.zw)*FullSourceSize);
    vec2 uv=u_crop.xy+TEX0.xy*u_crop.zw;
    vec3 rgb=palette_rgb(game_sample(Texture,u_aa>0.5?uv:(p+0.5)/FullSourceSize).rgb,u_palette);
    vec3 mask=texture2D(Mask,TEX0.xy).rgb;
    rgb*=max(vec3(0.0),vec3(1.0)-u_lcd_darkness*(vec3(1.0)-mask));
    if (u_post>0.5) {
        rgb=mix(vec3(dot(rgb,vec3(0.299,0.587,0.114))),rgb,u_saturation);
        rgb=mix(rgb*u_bright,vec3(1.0),u_glow);
    }
    FRAG_COLOR=vec4(rgb,1.0);
}
