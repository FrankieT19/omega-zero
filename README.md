<h1 align="center"><img src="assets/logo/omega-zero-logo@2x.png" width="420" alt="omega-zero"></h1>

<p align="center">A Game Boy-centric frontend for Anbernic H700 handhelds running
<a href="https://github.com/pvaibhav/BaseOS">BaseOS</a>. It plays GBA, GBC and GB games, and
only those, on a shelf of cartridges that go down into the handheld when you play them.</p>

<p align="center">
<a href="https://oauramos.github.io/omega-zero/"><b>Site</b></a> ·
<a href="https://github.com/oauramos/omega-zero/releases/latest"><b>Download</b></a> ·
<a href="https://github.com/oauramos/omega-zero/wiki"><b>Wiki</b></a> ·
<a href="CHANGELOG.md"><b>Changelog</b></a>
</p>

<img src="site/media/readme/supported-devices.png" width="760" alt="">

## Supported devices

One build runs on all of them: omega-zero reads the model from BaseOS at boot and lays itself out
for its panel.

| Device | Panel | Status |
|---|---|---|
| RG35XX SP | 640x480 | Main target, in daily use |
| RG SP | 720x480 | Supported, not yet retested on hardware |
| RG34XX SP, RG34XX | 720x480 | Untested |
| RG35XX Plus / 2024, H, Pro | 640x480 | Untested |
| RG40XX H, RG40XX V | 640x480 | Untested |
| RG CubeXX | 720x720 | Experimental |
| RG28XX | 480x640, turned | Experimental |

Own an untested one? A [device report](https://github.com/oauramos/omega-zero/wiki/Device-Reports)
is the most useful thing you can send.

<img src="site/media/readme/highlights.png" width="760" alt="">

## Highlights

- A shelf of carts in their own plastics with real label scans, modelled low-poly and drawn the
  way a PlayStation drew 3D. The one you are on floats and sways; A pulls it down out of the
  screen into the handheld, and the game comes on like a tube TV. At power-on, an opening in the
  style of Mega Man X5's boot terminal tells you the handheld's memory, chip and battery, then
  fades into the shelf.
- Two GBA cores, mGBA and gpSP, chosen per cart. GBA fills a 640x480 screen; Game Boy stays at 3x.
- Resume on boot, save states with a switcher, rewind, fast-forward, turbo buttons.
- Link play between two handhelds over Wi-Fi, for Pokémon trades, Advance Wars, Mario Kart and Tetris.
- Settings on START, in windows in the style of Mega Man Zero's menus that slide from screen to
  screen, over a background that keeps moving: CONFIGS (Fast Forward, Fast Forward Sound, Colour
  Correction, Rumble, Labels) and GENERAL (Date & Time, Network, Bluetooth, Theme, About).
- A tap of SELECT opens CONTROLS: what every button does, on the shelf and in a game, on one panel.
- Network: home Wi-Fi, network time, and SSH and SFTP, each ON or OFF. Y on one opens its panel:
  SSH's shows the login and root's own random password. Cart labels are fetched from
  ScreenScraper.
- Bluetooth, on handhelds that have it: pair a device and connect it, or forget it. Game sound and
  gamepads over Bluetooth are still to come.
- Theme: the background's pattern, its motion and its colour, which the menus and both bars take
  (Night is grey, Amber is Mega Man Zero's own), and the bottom bar's font, Bold or Thin.
- A pixel font, a top bar with the radios, clock and battery, and a bottom bar with the buttons.

<img src="site/media/readme/screens.png" width="760" alt="">

## Screens

<p align="center">
<a href="https://oauramos.github.io/omega-zero/"><img src="site/media/loops/lid.webp" width="640" alt="An RG35XX SP drawn in 3D, its lid open on the shelf of carts, Emerald in the middle"></a>
<br><sub>The RG35XX SP in 3D, drawn the way a PlayStation drew it, with omega-zero's own screens on it: the loops play on the <a href="https://oauramos.github.io/omega-zero/">site</a></sub>
</p>

<p align="center">
<img src="site/media/settings.png" width="380" alt="Settings in Night: CONFIGS (Fast Forward 6X, Fast Forward Sound, Colour Correction, Rumble, Labels) and GENERAL (Date and Time, Network, Bluetooth, Theme, About)">
<img src="site/media/amber.png" width="380" alt="The same Settings in Amber, Mega Man Zero's own orange and brown">
<br><sub>Settings, in Night and in Amber</sub>
</p>

<p align="center">
<img src="site/media/network.png" width="380" alt="Network: Wi-Fi, Network Time and SSH on, and the networks around with their padlocks, signal and CONNECTED">
<img src="site/media/bluetooth.png" width="380" alt="Bluetooth: the radio on, and a gamepad connected, headphones paired, a keyboard and a smart band">
<br><sub>Network and Bluetooth</sub>
</p>

<p align="center">
<img src="site/media/theme.png" width="380" alt="Theme: Background Plates, Motion Moving, Colour Night and Font Bold, over the plates pattern">
<img src="site/media/controls.png" width="380" alt="CONTROLS: what every button does on the shelf and in a game, on one panel">
<br><sub>Theme, and CONTROLS on a tap of SELECT</sub>
</p>

<img src="site/media/readme/quick-start.png" width="760" alt="">

## Quick start

1. Write the [BaseOS image](https://github.com/pvaibhav/BaseOS/releases/latest) for your model to a
   microSD card, put it in **SD1** and boot once.
2. Copy the contents of the [latest release](https://github.com/oauramos/omega-zero/releases/latest)
   to the root of that card.
3. Put your games in `Games/GBA`, `Games/GB` and `Games/GBC`.
4. Boot. On the shelf, START opens Settings and a tap of SELECT shows every button.

Two cards and updating are in the site's
[install guide](https://oauramos.github.io/omega-zero/#install); controls and everything else are
in the [wiki](https://github.com/oauramos/omega-zero/wiki).

<img src="site/media/readme/credits.png" width="760" alt="">

## Credits and licence

omega-zero is made by **Otávio ([@oauramos](https://github.com/oauramos))**.

- [BaseOS](https://github.com/pvaibhav/BaseOS) by pvaibhav.
- Emulation by [mGBA](https://mgba.io) (MPL-2.0) and [gpSP](https://github.com/libretro/gpsp)
  (GPL-2.0). Their licences, patches and gpSP's source ship in `System/licenses/`.
- Type is [Pixel Code](https://github.com/qwerasd205/PixelCode) by Qwerasd (SIL OFL 1.1); every
  icon is drawn as pixel art. The LCD mask is derived from GigaHerz's LCD3x.
- Cart labels come from [ScreenScraper](https://www.screenscraper.fr), fetched with your own
  account.

omega-zero is licensed under the GPL-3.0-or-later; see [LICENSE](LICENSE). It began in October
2026 as a modified version of a GPL-3.0 program. It is written with Claude, and reviewed and tested
by its maintainer. The site and these pictures are drawn with its
UI kit, [`assets/ui`](assets/ui).

Game Boy, Game Boy Color and Game Boy Advance are trademarks of Nintendo. Anbernic is a trademark
of its owner. This project is not affiliated with either.
