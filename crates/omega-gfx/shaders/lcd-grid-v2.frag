// Adapted from lcd-grid-v2.upstream.glsl; see that file for authors and license.
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
uniform COMPAT_PRECISION vec2 OutputSize;
uniform COMPAT_PRECISION vec2 TextureSize;
uniform COMPAT_PRECISION vec2 InputSize;
uniform COMPAT_PRECISION vec2 OrigInputSize;

varying vec4 TEX0;

// compatibility #defines
#define Source Texture
#define vTexCoord TEX0.xy

#define SourceSize vec4(TextureSize, 1.0 / TextureSize) //either TextureSize or InputSize
#define outsize vec4(OutputSize, 1.0 / OutputSize)

#ifdef PARAMETER_UNIFORM
// All parameter floats need to have COMPAT_PRECISION in front of them
uniform COMPAT_PRECISION float RSUBPIX_R;
uniform COMPAT_PRECISION float RSUBPIX_G;
uniform COMPAT_PRECISION float RSUBPIX_B;
uniform COMPAT_PRECISION float GSUBPIX_R;
uniform COMPAT_PRECISION float GSUBPIX_G;
uniform COMPAT_PRECISION float GSUBPIX_B;
uniform COMPAT_PRECISION float BSUBPIX_R;
uniform COMPAT_PRECISION float BSUBPIX_G;
uniform COMPAT_PRECISION float BSUBPIX_B;
uniform COMPAT_PRECISION float gain;
uniform COMPAT_PRECISION float gamma;
uniform COMPAT_PRECISION float outgamma;
uniform COMPAT_PRECISION float blacklevel;
uniform COMPAT_PRECISION float ambient;
uniform COMPAT_PRECISION float BGR;
#else
#define RSUBPIX_R 1.0
#define RSUBPIX_G 0.0
#define RSUBPIX_B 0.0
#define GSUBPIX_R 0.0
#define GSUBPIX_G 1.0
#define GSUBPIX_B 0.0
#define BSUBPIX_R 0.0
#define BSUBPIX_G 0.0
#define BSUBPIX_B 1.0
#define gain 1.0
#define gamma 3.0
#define outgamma 2.2
#define blacklevel 0.05
#define ambient 0.0
#define BGR 0.0
#endif

#define OriginalTexelSize (InputSize.xy / (TextureSize.xy * OrigInputSize.xy))
#define fetch_offset(coord, offset) (pow(vec3(gain) * input_sample( (vec2((coord) + (offset)) + 0.5) * OriginalTexelSize).rgb + vec3(blacklevel), vec3(gamma)) + vec3(ambient))

// integral of (1 - x^2 - x^4 + x^6)^2
float cx(int i) { if(i==0)return 1.0; if(i==1)return -2.0/3.0; if(i==2)return -1.0/5.0; if(i==3)return 4.0/7.0; if(i==4)return -1.0/9.0; if(i==5)return -2.0/11.0; return 1.0/13.0; }
// integral of (1 - 2x^4 + x^6)^2
float cy(int i) { if(i==0)return 1.0; if(i==1)return 0.0; if(i==2)return -4.0/5.0; if(i==3)return 2.0/7.0; if(i==4)return 4.0/9.0; if(i==5)return -4.0/11.0; return 1.0/13.0; }

float intsmear_func(float z, bool vertical)
{
    float z2 = z*z;
    float zn = z;
    float ret = 0.0;
    for (int i = 0; i < 7; i++) {
        ret += zn*(vertical ? cy(i) : cx(i));
        zn *= z2;
    }
    return ret;
}

float intsmear(float x, float dx, float d, bool vertical)
{
    float zl = clamp((x-dx*0.5)/d,-1.0,1.0);
    float zh = clamp((x+dx*0.5)/d,-1.0,1.0);
    return d * ( intsmear_func(zh,vertical) - intsmear_func(zl,vertical) )/dx;
}

void main()
{
    vec2 texelSize = OriginalTexelSize;
    /* float2 range = IN.video_size / (IN.output_size * IN.texture_size); */
    vec2 range = InputSize.xy / (OutputSize.xy * TextureSize.xy);//outsize.zw;

    vec3 cred   = pow(vec3(RSUBPIX_R, RSUBPIX_G, RSUBPIX_B), vec3(outgamma));
    vec3 cgreen = pow(vec3(GSUBPIX_R, GSUBPIX_G, GSUBPIX_B), vec3(outgamma));
    vec3 cblue  = pow(vec3(BSUBPIX_R, BSUBPIX_G, BSUBPIX_B), vec3(outgamma));

    ivec2 tli = ivec2(floor(vTexCoord/texelSize-vec2(0.4999)));

    vec3 lcol, rcol;
    float subpix = (vTexCoord.x/texelSize.x - 0.4999 - float(tli.x))*3.0;
    float rsubpix = range.x/texelSize.x * 3.0;

    lcol = vec3(intsmear(subpix+1.0, rsubpix, 1.5, false),
                intsmear(subpix    , rsubpix, 1.5, false),
                intsmear(subpix-1.0, rsubpix, 1.5, false));
    rcol = vec3(intsmear(subpix-2.0, rsubpix, 1.5, false),
                intsmear(subpix-3.0, rsubpix, 1.5, false),
                intsmear(subpix-4.0, rsubpix, 1.5, false));

    if (BGR > 0.5) {
        lcol.rgb = lcol.bgr;
        rcol.rgb = rcol.bgr;
    }

    float tcol, bcol;
    subpix = vTexCoord.y/texelSize.y - 0.4999 - float(tli.y);
    rsubpix = range.y/texelSize.y;
    tcol = intsmear(subpix    ,rsubpix, 0.63, true);
    bcol = intsmear(subpix-1.0,rsubpix, 0.63, true);

    vec3 topLeftColor     = fetch_offset(tli, ivec2(0,0)) * lcol * vec3(tcol);
    vec3 bottomRightColor = fetch_offset(tli, ivec2(1,1)) * rcol * vec3(bcol);
    vec3 bottomLeftColor  = fetch_offset(tli, ivec2(0,1)) * lcol * vec3(bcol);
    vec3 topRightColor    = fetch_offset(tli, ivec2(1,0)) * rcol * vec3(tcol);

    vec3 averageColor = topLeftColor + bottomRightColor + bottomLeftColor + topRightColor;

    averageColor = mat3(cred, cgreen, cblue) * averageColor;

    FRAG_COLOR = vec4(pow(averageColor, vec3(1.0/outgamma)),0.0);
} 
