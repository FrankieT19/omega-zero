// Adapted from lcd3x.upstream.glsl; see that file for authors and license.
#ifdef GL_ES
precision highp float;
#endif
uniform vec2 RasterSize;
uniform sampler2D Texture;
uniform sampler2D OrigTexture;
uniform vec4 u_crop;
uniform vec4 u_orig_crop;
vec4 input_sample(vec2 uv) { return vec4(palette_rgb(texture2D(Texture, u_crop.xy + clamp(uv,0.0,1.0)*u_crop.zw).rgb,u_palette),1.0); }
vec4 original_sample(vec2 uv) { return vec4(palette_rgb(texture2D(OrigTexture, u_orig_crop.xy + clamp(uv,0.0,1.0)*u_orig_crop.zw).rgb,u_orig_palette),1.0); }




#ifdef GL_ES
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
#define COMPAT_PRECISION highp
#else
#define COMPAT_PRECISION
#endif

uniform COMPAT_PRECISION int FrameDirection;
uniform COMPAT_PRECISION int FrameCount;
uniform COMPAT_PRECISION vec2 OrigInputSize;
uniform COMPAT_PRECISION vec2 TextureSize;
uniform COMPAT_PRECISION vec2 InputSize;
uniform COMPAT_PRECISION vec2 OutputSize;

varying vec4 TEX0;

// compatibility #defines
#define Source Texture
#define vTexCoord TEX0.xy

#define outsize vec4(OutputSize, 1.0 / OutputSize)

#ifdef PARAMETER_UNIFORM
// All parameter floats need to have COMPAT_PRECISION in front of them
uniform COMPAT_PRECISION float brighten_scanlines;
uniform COMPAT_PRECISION float brighten_lcd;
#else
#define brighten_scanlines 16.0
#define brighten_lcd 4.0
#endif

const vec3 offsets = vec3(3.141592654) * vec3(1.0/2.0,1.0/2.0 - 2.0/3.0,1.0/2.0-4.0/3.0);

void main()
{
    vec2 omega = vec2(3.141592654) * vec2(2.0) * RasterSize;
    vec3 res = input_sample( vTexCoord).xyz;

    vec2 angle = vTexCoord * TextureSize / InputSize * omega;
	
    float yfactor = (brighten_scanlines + sin(angle.y)) / (brighten_scanlines + 1.0);
    vec3 xfactors = (brighten_lcd + sin(angle.x + offsets)) / (brighten_lcd + 1.0);
   
    vec3 color = yfactor * xfactors * res;
   
   FRAG_COLOR = vec4(color.x, color.y, color.z, 1.0);
} 
