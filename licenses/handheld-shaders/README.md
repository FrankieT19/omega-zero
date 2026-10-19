# Native display effects

Gameplay runs directly through omega-zero's original libretro integration. No RetroArch process or Slang translation is used.

LCD masks use the original omega-zero sine coefficient approach, with brightness constants 16 and 4. LCD1x uses a monochrome cell; LCD3x uses the original RGB phase offsets. At 3x the LCD3x coefficients match `lcd3x_mask()` exactly. At other scales, the same nearest-sampled 3x3 mask repeats within each visible source pixel, including crop/zoom offsets. Game sampling is nearest. LCD darkness is adjustable and zero removes the modulation.

Grid and Grid+PT always use one black row and column in each 3x3 physical display cell, anchored to the screen origin. System, scaling, source resolution and crop/zoom do not change this raster. Pixel Transparency keeps its native GLES passes and adjustable shadow offsets. Combined LCD+PT presets apply the LCD mask before PT; Grid+PT reapplies the black separators after PT.

The upstream GLSL and Slang reference files retain their attribution and licences; they are reference material rather than a gameplay backend. Validate the GLES stages with `scripts/check-handheld-shaders.py`.
