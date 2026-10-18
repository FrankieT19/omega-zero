# FrankieT19 fork

Based on omega-zero 1.2.0. This frontend hosts libretro cores directly, so there is no RetroArch process or RetroArch shader menu to open.

## Controls and settings

START on the selector opens Settings. **Display** chooses the system's shader, adjusts Pixel Transparency Shadow X / Shadow Y, and enables Low poly labels.

- L/R cycles the populated system shelves, **All Systems**, **Recent**, and **Favourites**. All Systems and Favourites sort alphabetically; Recent contains the last 10 launched games, newest first, with repeats moved to the front. Press Y on the hovered game to add/remove a favourite, including removal directly from Favourites. Empty Recent/Favourites remain accessible. This preserves each cart/disc model and emulator. There is no All Systems setting.

- Turning off in the menus restores the hovered game and current carousel. It does not launch a game. Turning off during play resumes that active game, with the original crash-loop protection. A card with only one game also waits on the selector unless that game was active.

- SELECT+Left cycles blue-light levels 0 through 9. SELECT+Right cycles Clean, LCD1x, LCD3x, Grid (black 3x separators), Pixel Transparency, LCD1x + PT, LCD3x + PT, and Grid + PT for the current system. Names use the existing toast design and duration.

- Effects now use the upstream libretro GLSL shaders with RetroArch's pragma/preset defaults, at the actual game viewport dimensions. LCD+PT uses two passes, with the original frame supplied to PT. The former Mask and Pixel blend attenuation controls have been removed. Shadow X and Shadow Y use the real PT shader parameters, saved independently per system, from -30 to +30 in 0.5-pixel steps. This fork defaults both to 1 pixel as requested (upstream defaults to 3); choose 3.0 for upstream offsets. Offscreen targets are reused; no per-frame texture allocation. Sources and adaptation notes are in `crates/omega-gfx/shaders/`.

- The frontend hosts cores directly; there is no RetroArch process or RetroArch settings menu. These presets run in omega-zero's renderer.

- Brightness retains hardware levels 1–9, with four extra steps below level 1: 25%, 50%, 75%, and 87.5% black. These are implemented by multiplying the final display output, mathematically equivalent to an opaque-black overlay at those opacities, across both games and menus. Dimming is saved and restored. The panel is never set to hardware 0.

- Low poly labels pixelates scans in four-texel blocks with 5-bit colour channels. Workers bake this into cartridge textures, including discs, when loaded or changed; it adds no per-frame pass and does not write the styled image to your scans.

### Scaling

Display settings saves **Scaling** independently for GB, GBC, SNES and PS1: **Integer**, **Fit**, or **Stretch**. Fit fills the available height while keeping the handheld aspect ratio (160:144) or the consoles' 4:3 display aspect ratio. Stretch fills the screen. Integer uses the largest whole scale that fits the native frame, centring it; frames larger than the panel fall back to proportional downscaling. GB/GBC L/R cycles these modes during play. Console shoulder buttons remain with their games; change their scaling in Display settings. GBA retains its original layout, which is an exact 3× at 720×480.

### Replaceable splash

`System/bootlogo.png` is the white Anbernic wordmark on black. Replace that PNG with your own image; arbitrary dimensions are fitted proportionally with black letterboxing. PNG transparency is composited onto black. The frontend converts it to the panel-sized BMP required by the bootloader, retaining the original boot logo backup. The new logo takes effect at the next power-on after omega-zero has run once with the replacement. Invalid PNGs fall back to the bundled panel BMP. The terminal intro stays disabled; first-run clock setup remains.

The wordmark is derived from the official Anbernic website logo, rendered white on black. Anbernic is a trademark of its owner.

## Slot Studio shells

Copy Studio's `cart_shell.txt` to `Labels/cart_shell.txt` on the card containing the games. The Windows drive letter is whichever drive that card uses; no drive letter is hard-coded. Keys are ROM filenames without extensions. The existing `outline RRGGBB finish` format is unchanged, for example:

```ini

Pokemon - Emerald Version (USA, Europe) = auto 00aa66 clear

Tetris = notched 888888 solid

```

`Config/cart_shell.ini` < `Config/cart_shell.txt` < `Labels/cart_shell.ini` < `Labels/cart_shell.txt`. User labels override defaults. `Game = auto` removes a shell override. Legacy files remain readable and are never renamed or overwritten. UTF-8 BOM and Windows line endings are accepted.

## Build

Use the upstream pinned Rust toolchain and build workflow. Device compile check:

```sh

cargo check -p omega-zero --no-default-features --features device --target aarch64-unknown-linux-gnu

```

Full rendering tests require macOS, as in upstream CI. Hardware validation is still required for H700 performance and visual tuning. This source change alone is not an installable firmware release.


## v3 display changes

- Grid is now a physical panel mask: each 3x3 cell has a 2x2 lit area and a one-pixel black right/bottom separator. At 720x480, GBA's 240x160 source pixels align exactly with these cells. No fractional smearing, gamma integration or RGB subpixel pattern is used. Grid+PT is a single fused effect pass, skips PT work on black separators, and retains the real upstream PT algorithm/defaults on lit pixels. This intentionally replaces the former lcd-grid-v2 appearance; LCD1x/LCD3x retain their reference formulas.
- Stretched GB/GBC uses a panel raster of viewport/3 for LCD masks, separately from the actual source dimensions needed by PT. Stretching content therefore keeps display cells three physical pixels wide/high. The physical Grid mask uses this spacing in every scaling mode.
- Display > GB/GBC > Saturation: 0–200%, in 10% steps; 100% preserves existing colours. Saved separately for each system; applies to Clean and every shader, without affecting menus.
- GB already uses mGBA's game-specific GBC hardware palettes, equivalent to Gambatte's GBC game entries. For example, SML2 uses the same three sets of four colours (background cyan, orange player, blue enemies). The fallback Dark Green palette only applies without a game-specific entry. Colour Correction and LCD/PT can alter their displayed appearance.
- SELECT+B cycles Game colours, Pocket, DMG green, Blue, Grayscale on GB/GBC, with the normal toast. Game colours preserves the core's original palette; the other options convert source luminance to four shades before applying LCD/PT. This also allows four-shade styling of native GBC games; it does not replace the emulator or change a game's internal palette data. Each system remembers its choice.
- All decorative system overlays, wordmarks, rims and grey console borders are removed. Unused display space is plain black.
- SELECT+X during play opens PICTURE, pausing gameplay while leaving the frame visible for adjustment. Zoom: 100–200% in 5% steps. Crop each edge independently by 0–20% in 1% steps, then apply centred zoom. Reset restores 100%/no crop. B, START or SELECT+X closes it. Values are saved per platform/ROM in Config/picture.ini and reapplied on launch, including after a core changes resolution. These are manual controls for blank pixels supplied by games/cores; there is no automatic overscan detection.

Reference palette tables: [mGBA pinned source](https://github.com/libretro/mgba/blob/e31759b24e7a4e3899285ff720d7b573ac328ae7/src/gb/overrides.c), [Gambatte GBC entries](https://github.com/libretro/gambatte-libretro/blob/master/libgambatte/libretro/gbcpalettes.h).

Device compilation and GLES2 compile/link validation are checked locally. CI renders pixel-level grid/alignment, saturation, palette, crop/orientation and fixed-raster regressions on both supported landscape resolutions. H700 frame rate and the exact appearance on the RG SP panel still require hardware verification.

## v4: native GB palettes and display controls

- GB/GBC now default to Gambatte, pinned to d9d6cd06382d1ced30de34d56d3609452323dab1. The ARM64 build runs a real-core GB palette smoke test before packaging. mGBA remains selectable with X on GB/GBC carts for existing save states and cable link play. State files remain separated by core; existing battery saves are backed up as `<game>.pre-gambatte.sav` before first loading through Gambatte. Battery saves are shared; save states do not transfer between emulators.
- SELECT+B applies only to original GB games running Gambatte. It cycles game-specific GBC colours, Pocket, DMG, Blue, grayscale, Light, and the remaining original GBC presets (Brown, Dark Blue, Dark Brown, Dark Green, Green, Inverted, Orange, Pastel Mix, Red, Yellow). These are Gambatte's native background/sprite palettes; the v3 luminance remap is no longer used. Native GBC games always keep their full colours.
- SELECT+X > Scaling selects per-game Integer / Fit / Stretch without leaving play. On SNES/PS1, Fit is 4:3 and fills the available height on landscape panels; Stretch fills all four screen edges. Integer uses native whole-pixel scaling, so SNES's usual 256x224 frame has its native 8:7 pixel aspect in this mode. Existing v3 SNES defaults migrate once to Fit. Per-game scaling is saved with crop/zoom; legacy five-value picture files still load.
- Saturation 0–200% is saved independently for all five systems. LCD darkness 0–200% controls the LCD1x/LCD3x mask, including LCD+PT: 0% removes mask darkening, 100% is the original shader formula, and 200% deepens the mask. Both are in Display settings; shadows, grid spacing and PT defaults are otherwise preserved.
- The brightness indicator has 13 cells, one per hardware/dimming step, with the darkest level still showing one cell.
- Carousel models are drawn from farthest to nearest, so a focused SNES cart covers its GB/GBC neighbours on either side.

Gambatte palette definitions: https://github.com/libretro/gambatte-libretro/blob/d9d6cd06382d1ced30de34d56d3609452323dab1/libgambatte/libretro/libretro_core_options.h
