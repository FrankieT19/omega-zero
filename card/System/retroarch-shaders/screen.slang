#version 450
// omega-zero's display controls. LCD1x/LCD3x and PT run in their original passes.
#pragma parameter OMEGA_GRID "Fixed screen grid" 0.0 0.0 1.0 1.0
#pragma parameter OMEGA_SATURATION "Saturation" 1.0 0.0 2.0 0.1
#pragma parameter OMEGA_LEFT "Crop left" 0.0 0.0 0.2 0.01
#pragma parameter OMEGA_RIGHT "Crop right" 0.0 0.0 0.2 0.01
#pragma parameter OMEGA_TOP "Crop top" 0.0 0.0 0.2 0.01
#pragma parameter OMEGA_BOTTOM "Crop bottom" 0.0 0.0 0.2 0.01
#pragma parameter OMEGA_ZOOM "Zoom" 1.0 1.0 2.0 0.01
#pragma parameter OMEGA_ORIGIN_X "Screen grid origin X" 0.0 0.0 4096.0 1.0
#pragma parameter OMEGA_ORIGIN_Y "Screen grid origin Y" 0.0 0.0 4096.0 1.0
layout(push_constant) uniform Push {
    vec4 SourceSize;
    vec4 OriginalSize;
    vec4 OutputSize;
    float OMEGA_GRID;
    float OMEGA_SATURATION;
    float OMEGA_LEFT;
    float OMEGA_RIGHT;
    float OMEGA_TOP;
    float OMEGA_BOTTOM;
    float OMEGA_ZOOM;
    float OMEGA_ORIGIN_X;
    float OMEGA_ORIGIN_Y;
} params;
layout(std140, set=0, binding=0) uniform UBO { mat4 MVP; } global;
#pragma stage vertex
layout(location=0) in vec4 Position;
layout(location=1) in vec2 TexCoord;
layout(location=0) out vec2 uv;
void main() { gl_Position=global.MVP*Position; uv=TexCoord; }
#pragma stage fragment
layout(location=0) in vec2 uv;
layout(location=0) out vec4 FragColor;
layout(set=0, binding=2) uniform sampler2D Source;
void main() {
    // OutputSize is the actual viewport, not the emulated device's resolution.
    // In a one-to-one final pass each pixel is one physical panel pixel.
    vec2 cell=mod(floor(uv*params.OutputSize.xy)+vec2(params.OMEGA_ORIGIN_X,params.OMEGA_ORIGIN_Y),3.0);
    if (params.OMEGA_GRID>0.5 && (cell.x>1.5 || cell.y>1.5)) { FragColor=vec4(0.0,0.0,0.0,1.0); return; }
    vec2 lo=vec2(params.OMEGA_LEFT,params.OMEGA_TOP);
    vec2 hi=vec2(params.OMEGA_RIGHT,params.OMEGA_BOTTOM);
    vec2 source=lo+(1.0-lo-hi)*((uv-0.5)/params.OMEGA_ZOOM+0.5);
    vec3 rgb=texture(Source,source).rgb;
    rgb=mix(vec3(dot(rgb,vec3(0.299,0.587,0.114))),rgb,params.OMEGA_SATURATION);
    // Brightness/blue-light controls are applied after all gameplay, menu and OSD rendering.
    FragColor=vec4(rgb,1.0);
}
