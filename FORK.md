# FrankieT19 fork

Based on omega-zero 1.2.0. This frontend hosts libretro cores directly, so there is no RetroArch process or RetroArch shader menu to open.

## Controls and settings

START on the selector opens Settings. Open **Display & Startup**:

- System selects GBA, GB, GBC, SNES or PS1. Shader, Mask and Pixel blend are saved separately for each system.
- Shader cycles Clean, LCD1x, LCD3x, Grid, Pixel transparency, LCD1x + PT, LCD3x + PT, Grid + PT. These are lightweight native approximations, not imported RetroArch presets. Pixel transparency blends neighbouring horizontal pixels to soften checkerboard dithering.
- Mask adjusts the LCD/grid effect strength. Pixel blend adjusts the transparency blend amount.
- Boot last game defaults on. It remembers the last successfully launched game after ejecting to the selector. Disable it to start on the selector. Deleted games fall back to the selector; startup-crash protection still prevents a crash loop.
- All systems combines the five supported systems in one alphabetically sorted carousel, preserving each cart/disc model and emulator. Turn it off to restore the system shelves.
- Low poly labels pixelates scans in four-texel blocks with 5-bit colour channels. Workers bake this into cartridge textures, including discs, when loaded or changed; it adds no per-frame pass and does not write the styled image to your scans. Existing upstream label shrinking is unchanged.

SELECT+Left cycles blue-light levels 0 through 9. SELECT+Right cycles the current system's shader. The shader name uses the existing toast design and 1.5-second lifetime. Up/Down brightness controls are unchanged. SELECT chords use the existing 600 ms chord window.

The frontend terminal intro and logo fade are disabled. The BaseOS/bootloader hardware logo and first-run clock setup remain.

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
