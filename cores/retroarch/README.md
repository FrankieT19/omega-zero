# RetroArch gameplay backend

The H700 package includes a pinned RetroArch build with `glcore`, Mali framebuffer EGL/ES3,
Slang, ALSA and RGUI. The carousel still uses omega-zero's GLES renderer. Gameplay runs in
a separate process; exiting replaces the companion with a new carousel process. They never
own EGL, audio or the game input concurrently.

The backend source is libretro/RetroArch at `ea79555bc8fcd4cb1a2fb8a48c1a6cdba819fb07`.
`build.sh` applies `patch.py` and builds on ARM64 Debian bullseye. The patch adds a private
stdin controller/input/rumble bridge and advertises the Mali context's ES3 Slang capability.
It does not replace RetroArch's Slang renderer, compiler, uniforms or pass processing.
The original source archive, patcher, build recipe and native smoke test ship under
`System/licenses/`. RetroArch is GPL-3.0-or-later, including its own bundled dependency
licences in the source archive.

`System/retroarch-shaders/lcd1x.slang` and `lcd3x.slang` are byte-for-byte copies from
libretro/slang-shaders at `e1d75632a205c70f14a4cc947c46d5abb7b3f7f1`. The Pixel Transparency
source is from the same commit. Author and licence notices remain in those files. LCD passes
use nearest filtering, viewport output, no float/sRGB framebuffer, and original source-size
uniforms. At 100% LCD darkness the upstream brightness parameters are unchanged (16 and 4).
A final neutral display-control pass adds saturation, brightness, blue-light reduction and
optional cropping. Only Grid / Grid + PT uses its fixed three-physical-pixel raster.

Display > Game backend switches RetroArch / Native. Display > Shader settings opens
RetroArch on an available game of the selected system, starting in its menu. SELECT+X or
MENU opens that menu during gameplay. Select Quick Menu > Shaders to change presets and
parameters. The native photo/save-state browser becomes RetroArch's save-state menu in
this backend. Link play remains available through the Native backend.

SELECT+Right cycles Clean, LCD1x, LCD3x, Grid, PT, LCD1x+PT, LCD3x+PT, Grid+PT.
SELECT+Left cycles blue-light levels; SELECT+Up/Down controls all 13 brightness steps;
SELECT+L1/R1 loads/saves; SELECT+B cycles Gambatte's GB palettes. L2/R2 rewind/fast-forward
(on PS1 these remain game buttons, and fast-forward uses SELECT+R2). Hold MENU to save and
return to the carousel. GB/GBC L1/R1 keep their Fit/Stretch shortcuts. Volume, rumble,
headphone volume, lid sleep, inactivity dim/sleep and power-off run in the companion.
Notifications use RetroArch's OSD, with a 1.5-second duration for the companion's messages.

Generated configs/presets live in `Config/retroarch/<SYSTEM>/`; the latest gameplay log is
`Config/retroarch/retroarch.log`. To preserve a preset tweaked in RetroArch, save it as
`Config/retroarch/<SYSTEM>/custom.slangp`. It is then used at launch; cycling presets
intentionally selects the generated presets instead. The generated files can be recreated.
When loading an unrelated preset without omega-zero's final display pass, that preset also
controls which software display adjustments are available. Hardware brightness still works.

Native save states remain untouched. RetroArch stores its own states under
`States/RetroArch/<SYSTEM>/<CORE>/`. Raw native states are not automatically imported.
Battery RAM is shared through `Saves/<SYSTEM>/<GAME>.sav`, with a one-time
`<GAME>.pre-retroarch.sav` backup. RetroArch's active `.srm` copy lives under
`Saves/RetroArch/<SYSTEM>/`; a newer copy surviving a power loss is retained. Normal exit,
lid-timeout shutdown and power-off finish RetroArch's battery/autostate writes before
returning to the carousel or shutting down. On a menu exit, the active-game marker is cleared;
power-off during gameplay keeps it for resume. An old native resume remains available if
you select Native, but it describes the earlier native session, not newer RetroArch progress.

The device build runs a real libretro probe through the packaged ARM64 RetroArch, checking
input, save/load, fast-forward, menu, pause/resume, battery/autostate writes and clean exit.
Shader stages are checked separately. Actual Mali shader output, frame pacing, ALSA and lid
hardware behavior need testing on the handheld; desktop checks cannot establish stock-firmware
visual identity on that display.
