# FrankieT19 fork

Based on omega-zero 1.2.0. This frontend hosts libretro cores directly, so there is no RetroArch process or RetroArch shader menu to open.

## Controls and settings

START on the selector opens Settings. **Display** chooses the system's shader, adjusts Pixel Transparency Shadow X / Shadow Y, and enables Low poly labels.

- L/R cycles the populated system shelves, **All Systems**, **Recent**, and **Favourites**. All Systems and Favourites sort alphabetically; Recent contains the last 10 launched games, newest first, with repeats moved to the front. Press Y on the hovered game to add/remove a favourite, including removal directly from Favourites. Empty Recent/Favourites remain accessible. This preserves each cart/disc model and emulator. There is no All Systems setting.
- Turning off in the menus restores the hovered game and current carousel. It does not launch a game. Turning off during play resumes that active game, with the original crash-loop protection. A card with only one game also waits on the selector unless that game was active.
- SELECT+Left cycles blue-light levels 0 through 9. SELECT+Right cycles Clean, LCD1x, LCD3x, Grid (LCD-grid-v2), Pixel Transparency, LCD1x + PT, LCD3x + PT, and Grid + PT for the current system. Names use the existing toast design and duration.
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
