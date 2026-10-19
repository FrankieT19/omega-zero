# Omega Zero FrankieT's personal edition

A personal fork of omega-zero 1.2.0 for Anbernic H700 handhelds running BaseOS. This frontend hosts libretro cores directly, using its native GLES renderer. It does not launch RetroArch. See [the README](README.md) for the feature overview and upstream credits.

## Startup and carousels

The frontend intro is skipped. Powering off in the menus remembers the hovered game and current carousel; powering off during gameplay resumes that game, with the existing crash-loop protection. A single-game card still waits at the selector unless that game was active at shutdown.

L/R cycles populated system shelves, All Systems, Recent and Favourites. Recent holds the last 10 launched games, newest first without duplicates. All Systems and Favourites sort alphabetically. Y adds or removes the hovered game as a favourite. Empty Recent/Favourites remain accessible.

`System/bootlogo.png` is the replaceable Anbernic wordmark on black. Replace that PNG with your own image; arbitrary dimensions are fitted proportionally with black letterboxing and transparency is composited onto black. After running once with the replacement, the next power-on uses the converted bootloader logo. Invalid PNGs fall back to the bundled panel BMP. The original boot logo backup is retained.

## Display, shaders and palettes

START on the selector opens Settings. Display chooses the system and its shared shader, scaling, saturation and darkness settings. Pixel Transparency Shadow X/Y are saved per system in half-pixel steps, defaulting to 1 pixel each (upstream uses 3).

- SELECT+Right cycles shaders, using the normal notification look and duration. Handhelds offer Clean, LCD1x, LCD3x, Grid, Pixel Transparency, LCD1x + PT, LCD3x + PT and Grid + PT.
- SNES/PS1 instead offer Clean, CRT Scanlines, zfast CRT and CRT Pi (flat). There is no curvature or decorative border. Each CRT uses a single native GLES pass. They are adaptations of the upstream formulas with output-aligned beam phase and darkness control; source/licences are in `crates/omega-gfx/shaders/` and the packaged `System/licenses/handheld-shaders/`.
- LCD masks follow the current game pixels and crop, while Grid is fixed to the physical screen: one black row and column per 3x3 cell, including in Fit/Stretch and Grid+PT. LCD/Grid darkness controls separator opacity (100% black, 50% half the underlying colour, 0% removed); consoles show this control as Scanline darkness. Saturation is independent for all systems.
- Clean uses sharp bilinear scaling. GB/GBC apply the same boundary blend to fractional scaling with effects, preserving integer pixel interiors. Console CRTs have their own lightweight interpolation. GBA LCD sampling retains the upstream reference behavior. LCD+PT reuses viewport targets rather than allocating them each frame.
- Gambatte is the default for GB/GBC; mGBA remains selectable. SELECT+B cycles the supported GBC hardware/game palettes and classic Pocket, DMG, Light and grayscale choices for GB games only. GBC games retain full colour.
- SELECT+Left cycles blue-light levels 0-9.

## Per-game picture and shader override

SELECT+X during play pauses gameplay and opens Picture. B, START or SELECT+X closes it. Zoom is 100-200% in 5% steps; crop each edge by 0-20% in 1% steps. Crop/zoom are saved per system/ROM in `Config/picture.ini` and reapplied after resolution changes. Reset restores the picture controls; it does not clear a shader override.

Enable the Shader override checkbox to capture the current shader for this game, then change its Shader row. Outside the popup, SELECT+Right also changes the override while enabled. Disable the checkbox to restore the shared system default immediately. With the checkbox off, the Shader row and hotkey edit the shared preset. Display always edits the shared default. Overrides are stored by system and ROM filename stem in `Config/shaders.ini`.

## Scaling

- **GBA:** retains its original layout, exactly 3x on 720x480.
- **GB/GBC:** Integer, Integer H stretch, Fit, Stretch. L/R cycles them during play. Integer H stretch keeps integer height and fills the width (720x432 on 720x480, with 24 pixels of black above/below). Fit keeps 160:144 aspect while filling the available height. Stretch fills the panel.
- **SNES:** Integer, **Integer H 4:3**, Fit 4:3, Stretch. Integer retains the raw frame's square pixels. Integer H 4:3 selects the largest whole-number source-row scale that fits and adjusts width to the TV's 4:3 aspect: a 224-line frame becomes about 597x448 on both 720x480 and 640x480. Hi-res 512-pixel-wide frames retain the same display shape; 239-line frames become about 637x478. This preference is global, including when changed in Picture; old per-game SNES scaling overrides are ignored.
- **PS1:** Integer, Fit 4:3, Stretch. Integer uses whole-number height scaling with 4:3 display correction rather than treating the framebuffer as square pixels. For example, 320x240, 512x240 and 640x240 all display at 640x480 on either 480-line landscape panel. This avoids a wide, short rectangle when games switch horizontal resolution. PS1 scaling can still be saved per game through Picture.

Integer height modes fall back to Fit 4:3 if even 1x source height cannot fit. Width is rounded to an output pixel; only height is constrained to an integer multiplier. Crop/zoom adjustments can deliberately change the effective pixel scale. Decorative overlays, wordmarks and grey console borders are removed; unused screen space is black.

Core geometry references: [PCSX ReARMed](https://docs.libretro.com/library/pcsx_rearmed/) (4:3 display aspect), [Snes9x](https://docs.libretro.com/library/snes9x/) (224/239-line base height and selectable display aspect).

## Brightness, Sleep and input

Brightness has hardware levels 1-9 and four lower software steps equivalent to black overlays at 25%, 50%, 75% and 87.5%. The final output is dimmed across games and menus, without setting hardware brightness to 0. The indicator represents all 13 steps.

Sleep settings separately configure menu/game Dim after and Sleep after: Off, 15s, 30s, 1/2/5/10/15/30/60 minutes. Defaults are menu dim 1 minute/sleep 5 minutes and game dim 5 minutes/sleep 15 minutes. In-game overlays use menu timers; any input, including held buttons, resets inactivity. Dimming fades to 25% of the chosen brightness and activity fades back. Sleep fades to black, then uses the existing save/pause, radio cool-down and backlight-off path used by lid close. Power wakes during the three-minute doze window; after power-down the normal boot/resume rules apply.

Carousel animation handles long frames; core exit clears held movement, chords, fast-forward and rewind state. Repeated Down reports for an already-held key are discarded. Input timing uses the presentation's monotonic clock. Device navigation saves use an ordered, coalescing writer, with durable fences for settings and shutdown. These changes address software causes of stalls/skipping; repeated GBA/CTR exits still need physical-device verification.

## Slot Studio shells and labels

Copy Studio's `cart_shell.txt` to `Labels/cart_shell.txt` on the card containing the games. The Windows drive letter is whichever drive the card uses. Keys are ROM filenames without extensions; the existing `outline RRGGBB finish` format remains, for example:

```ini
Pokemon - Emerald Version (USA, Europe) = auto 00aa66 clear
Tetris = notched 888888 solid
```

Precedence: `Config/cart_shell.ini` < `Config/cart_shell.txt` < `Labels/cart_shell.ini` < `Labels/cart_shell.txt`. User labels override defaults; `Game = auto` removes a shell override. UTF-8 BOM and Windows line endings are accepted.

Low poly labels pixelates scans in four-texel blocks with 5-bit colour channels. Workers bake styled cartridge/disc textures when loaded or changed, without changing the original scans or adding a per-frame pass. The selected cart draws above its neighbours. The Wi-Fi icon is nine art pixels wide, seven tall and aligned with the UI's integer pixel grid.

## Install and build

Custom builds are artifacts in [this fork's build workflow](https://github.com/FrankieT19/omega-zero/actions/workflows/release.yml), not the upstream release downloads. This is a frontend update for an existing BaseOS card, not a flashable OS image. Back up Saves, States and Config; replace the package's System files, preserving your custom boot PNG and card data. Native save states remain usable; states from the withdrawn RetroArch backend are not automatically imported. v10 and the pre-RetroArch backup remain available separately.

Use the pinned Rust toolchain and upstream build workflow. Local device check:

```sh
cargo check -p omega-zero --no-default-features --features device --target aarch64-unknown-linux-gnu
```

CI runs host/device lint, formatting, card scripts and workspace/render suites at 720x480 and 640x480. LCD tests compare against the included unmodified upstream GLSL references. Physical H700 performance, screen appearance and power behavior still require device testing.
