# Native display effects

Gameplay uses omega-zero's native libretro integration. LCD1x/LCD3x use the continuous modulation equations and default constants (16 and 4) of the upstream GLSL/Slang reference files, with nearest game sampling. LCD1x retains its quarter-cell offset on both axes. LCD3x retains its positive half-pi channel phase and negative two-thirds-pi RGB steps. The coefficients are cached at physical output pixel centres, following source resolution and crop/zoom, without snapping to a fixed 3x3 approximation.

Test-support renders the actual unmodified upstream GLSL sources and compares their RGB output with native rendering over multiple source sizes, output scales and crop windows. This establishes the shader math/sampling agreement, not visual identity to a particular hardware/firmware configuration.

Grid and Grid+PT use fixed three-physical-pixel cells, independent of system/scaling/crop. LCD/Grid darkness controls separator opacity: at 100% they are black, at 50% half the underlying colour passes, and at 0% the grid disappears. Grid+PT applies that attenuation to the transparency result and retains its black-separator fast path at 100%.

Upstream files preserve their author/licence notices. LCD1x is adapted from jdgleaver's GPL-2.0-or-later shader, LCD3x from Gigaherz's public-domain shader; Pixel Transparency is by Matt Akins. Validate GLES stages with scripts/check-handheld-shaders.py.
