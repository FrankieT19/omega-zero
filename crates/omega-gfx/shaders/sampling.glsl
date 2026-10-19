#ifdef GL_ES
precision highp float;
#endif
uniform float u_aa;
uniform vec2 u_sample_size, u_sample_scale, u_original_size, u_original_scale;
vec2 sharp_coord(vec2 uv, vec2 size, vec2 scale) {
    vec2 p=uv*size;
    scale=max(scale,vec2(1.0));
    vec2 d=fract(p)-0.5;
    vec2 r=0.5-0.5/scale;
    return (floor(p)+(d-clamp(d,-r,r))*scale+0.5)/size;
}
vec4 game_sample(sampler2D image, vec2 uv) {
    if (u_aa>0.5) uv=sharp_coord(uv,u_sample_size,u_sample_scale);
    return texture2D(image,uv);
}
vec4 original_sample_aa(sampler2D image, vec2 uv) {
    if (u_aa>0.5) uv=sharp_coord(uv,u_original_size,u_original_scale);
    return texture2D(image,uv);
}
