// Adapted from lcd1x.upstream.glsl; see that file for authors and license.
#ifdef GL_ES
precision highp float;
#endif
uniform sampler2D Texture;
uniform sampler2D OrigTexture;
uniform vec4 u_crop;
uniform vec4 u_orig_crop;
vec4 input_sample(vec2 uv) { return texture2D(Texture, u_crop.xy + clamp(uv,0.0,1.0)*u_crop.zw); }
vec4 original_sample(vec2 uv) { return texture2D(OrigTexture, u_orig_crop.xy + clamp(uv,0.0,1.0)*u_orig_crop.zw); }




#ifdef GL_ES
precision highp float;
precision highp int;
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

#ifdef PARAMETER_UNIFORM
// All parameter floats need to have COMPAT_PRECISION in front of them
uniform COMPAT_PRECISION float BRIGHTEN_SCANLINES;
uniform COMPAT_PRECISION float BRIGHTEN_LCD;
#else
#define BRIGHTEN_SCANLINES 16.0
#define BRIGHTEN_LCD 4.0
#endif

// Magic Numbers
#define PI 3.141592654

/*
   FRAGMENT SHADER
*/
void main()
{
   // Generate LCD grid effect
   // > Note the 0.25 pixel offset -> required to ensure that
   //   scanlines occur *between* pixels
   COMPAT_PRECISION vec2 angle =
      2.0 * PI * ((TEX0.xy * OrigInputSize * TextureSize / InputSize) - 0.25);

   COMPAT_PRECISION float yfactor = (BRIGHTEN_SCANLINES + sin(angle.y)) / (BRIGHTEN_SCANLINES + 1.0);
   COMPAT_PRECISION float xfactor = (BRIGHTEN_LCD + sin(angle.x)) / (BRIGHTEN_LCD + 1.0);

   // Get colour sample
   COMPAT_PRECISION vec3 colour = input_sample( TEX0.xy).rgb;

   // Apply LCD grid effect
   colour.rgb = yfactor * xfactor * colour.rgb;

   FRAG_COLOR = vec4(colour.rgb, 1.0);
} 
