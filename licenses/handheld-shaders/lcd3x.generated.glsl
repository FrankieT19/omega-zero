// Generated from lcd3x.upstream.slang; source SHA256: 58cb684d42d3ab336aefedb7f961bf551428d21d7371d2fa9eae64cbdaafcdec
// glslang -V -Os -> SPIRV-Cross --es --version 100. Do not edit shader arithmetic.
precision mediump float;
precision highp int;

struct Push
{
    highp vec4 OriginalSize;
    highp float brighten_scanlines;
    highp float brighten_lcd;
};

uniform Push params;

uniform highp sampler2D Source;

varying highp vec2 vTexCoord;

void main()
{
    highp vec2 _41 = vTexCoord * (vec2(6.283185482025146484375) * params.OriginalSize.xy);
    FRAG_COLOR = vec4((((vec3(params.brighten_lcd) + sin(vec3(_41.x) + vec3(1.57079637050628662109375, -0.52359879016876220703125, -2.617993831634521484375))) / vec3(params.brighten_lcd + 1.0)) * ((params.brighten_scanlines + sin(_41.y)) / (params.brighten_scanlines + 1.0))) * texture2D(Source, vTexCoord).xyz, 1.0);
}

