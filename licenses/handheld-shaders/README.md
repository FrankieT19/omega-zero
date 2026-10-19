# Native display effects

Gameplay uses omega-zero's native libretro integration. LCD1x/LCD3x use the continuous modulation equations and default constants (16 and 4) of the upstream GLSL/Slang reference files, with nearest game sampling. LCD1x retains its quarter-cell offset on both axes. LCD3x retains its positive half-pi channel phase and negative two-thirds-pi RGB steps. The coefficients are cached at physical output pixel centres, following source resolution and crop/zoom, without snapping to a fixed 3x3 approximation.

Test-support renders the actual unmodified upstream GLSL sources and compares their RGB output with native rendering over multiple source sizes, output scales and crop windows. This establishes the shader math/sampling agreement, not visual identity to a particular hardware/firmware configuration.

Grid and Grid+PT use fixed three-physical-pixel cells, independent of system/scaling/crop. LCD/Grid darkness controls separator opacity: at 100% they are black, at 50% half the underlying colour passes, and at 0% the grid disappears. Grid+PT applies that attenuation to the transparency result and retains its black-separator fast path at 100%.

Upstream files preserve their author/licence notices. LCD1x is adapted from jdgleaver's GPL-2.0-or-later shader, LCD3x from Gigaherz's public-domain shader; Pixel Transparency is by Matt Akins. Validate GLES stages with scripts/check-handheld-shaders.py.


## Flat console CRT presets and scaling

SNES/PS1 offer Clean, CRT Scanlines, zfast CRT and CRT Pi (flat). All are native
single-pass GLES 2 shaders. There is no curvature, border, geometry distortion,
feedback, history buffer or extra full-screen scaling pass.

- CRT Scanlines uses the existing sharp bilinear scaler and an analytically
  pixel-averaged cosine beam (sinc attenuation) to limit scanline moire.
- zfast CRT adapts Greg Hogan/SoltanGris42's GPL-2.0-or-later `zfast_crt.glsl`.
  Its Quilez filtering, brightness-dependent beam and fine mask are retained.
- CRT Pi (flat) adapts davej's GPL-2.0-or-later `crt-pi.glsl`, with curvature
  disabled, three-sample scanlines, gamma, bloom and green/magenta aperture mask.
  Both unmodified upstream sources are included alongside the native adapters:
  https://github.com/libretro/glsl-shaders/tree/master/crt/shaders

These are native adaptations, not claims of byte-identical RetroArch output.
The console beam phase aligns one separator to an output pixel at integer scales
(including PS1's 2x), avoiding the two equal-brightness rows produced by symmetric
sampling at 2x. At 1x or downscaling the beam fades out to avoid darkening the
whole frame. The Display scanline darkness control blends out the effect.

Clean already uses sharp bilinear scaling. GB/GBC now use the same single-sample
hardware-filtered boundary blend for every effect at fractional scales; at integer
scales it returns nearest texels. Only game sampling changes, not LCD mask phase
or Grid's fixed physical 3x3 raster. PT's original and intermediate inputs use
their own texture sizes/scales. GBA keeps its upstream LCD sampling unchanged.
CRT presets perform their own lightweight interpolation; no additional AA pass
is stacked on zfast/CRT Pi. H700 frame rate and appearance require device testing.


Console CRT beam phase is computed from physical output-pixel centres and the
cropped source-row origin/step, rather than interpolated source coordinates.
This keeps repeated integer-scaled scanlines consistent across the viewport.
Console darkness adjusts the beam independently of the aperture mask. PS1
Integer and SNES Integer H 4:3 re-fit quantized crop/zoom windows at integer row
scales; Fit remains continuous and may equal Integer at an exact 2x height.
