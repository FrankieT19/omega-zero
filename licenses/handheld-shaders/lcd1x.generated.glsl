// Generated from lcd1x.upstream.slang; source SHA256: 63c8dd3726c853feb12c8593cd192dfa3fb881449fd5dc672852f8024d511c82
// glslang -V -Os -> SPIRV-Cross --es --version 100. Do not edit shader arithmetic.
precision mediump float;
precision highp int;

struct Push
{
    highp float BRIGHTEN_SCANLINES;
    highp float BRIGHTEN_LCD;
    highp vec4 OriginalSize;
};

uniform Push registers;

uniform highp sampler2D Source;

varying highp vec2 vTexCoord;

void main()
{
    highp vec2 _30 = ((vTexCoord * registers.OriginalSize.xy) - vec2(0.25)) * 6.283185482025146484375;
    FRAG_COLOR = vec4(texture2D(Source, vTexCoord).xyz * (((registers.BRIGHTEN_SCANLINES + sin(_30.y)) / (registers.BRIGHTEN_SCANLINES + 1.0)) * ((registers.BRIGHTEN_LCD + sin(_30.x)) / (registers.BRIGHTEN_LCD + 1.0))), 1.0);
}

