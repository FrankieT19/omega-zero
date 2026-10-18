#ifdef GL_ES
precision highp float;
#endif
uniform float u_palette, u_orig_palette;
vec3 palette_rgb(vec3 rgb, float palette) {
    if (palette < 0.5) return rgb;
    float l=dot(rgb,vec3(0.299,0.587,0.114));
    float shade=floor(min(l,0.9999)*4.0);
    vec3 dark=vec3(0.0), mid1=vec3(85.0), mid2=vec3(170.0), light=vec3(255.0);
    if (palette < 1.5) { dark=vec3(42.0,51.0,37.0); mid1=vec3(83.0,95.0,73.0); mid2=vec3(134.0,146.0,124.0); light=vec3(167.0,177.0,154.0); }
    else if (palette < 2.5) { dark=vec3(15.0,56.0,15.0); mid1=vec3(48.0,98.0,48.0); mid2=vec3(139.0,172.0,15.0); light=vec3(155.0,188.0,15.0); }
    else if (palette < 3.5) { mid1=vec3(0.0,0.0,255.0); mid2=vec3(99.0,165.0,255.0); }
    return (shade<0.5?dark:shade<1.5?mid1:shade<2.5?mid2:light)/255.0;
}
