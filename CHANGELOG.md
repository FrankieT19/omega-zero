# Changelog

## 1.6.0 (unreleased)

- Wi-Fi in the quick menu: switch it on, pick a network and type its passphrase on an
  on-screen keyboard. Networks are remembered in `Config/wifi.conf` and joined again at boot.
- Once joined, the Wi-Fi screen shows the address and name to reach the handheld over SSH and
  SFTP.
- Network Time keeps the clock right through BaseOS whenever Wi-Fi is joined.
- A link takes the radio over from home Wi-Fi and gives it back when it ends.
- Handhelds without Wi-Fi don't show the row.
- Cart Labels, on the Wi-Fi screen: sign in with your ScreenScraper account and fetch the
  scanned label of every cart that has none, from the ROM's own region first. Labels go on the
  shelf as they arrive. Yours are never replaced.
- The on-screen keyboard's last key says Done.
- On a 640x480 panel, the band under a GBA game looks like the console's lens: its dark grey, a
  black rim at the screen's edge, and the GAME BOY ADVANCE logo in white edged in black.

## 1.5.0 (unreleased)

The first release of this fork, for the RG35XX SP and the other H700 handhelds BaseOS runs on.

- slot lays itself out for the panel it finds, from the model BaseOS names: 640x480 on the
  RG35XX SP instead of the RG SP's 720x480 picture shrunk to fit. The RG SP is unchanged.
- On a 640x480 panel, GBA games fill the width (640x427) through a sharp scaler, with the
  console's body below carrying GAME BOY ADVANCE. Game Boy games stay at 3x through the LCD
  mask.
- Your own overlays: a `.png` the size of the screen in `Overlays/GBA`, `Overlays/GB` or
  `Overlays/GBC`, laid over the game. One that would cover the game is ignored.
- The boot logo is only ever replaced with one the size of the device's own. Logos ship for
  640x480, 720x480 and 720x720 panels.
- Rumble works on devices whose motor is the PMIC's `moto` pin rather than a force feedback
  node.
- Hosting a link works on devices whose Wi-Fi driver gives no second interface.
- The About label fits a 640-wide screen.
- `System/probe.sh` writes a hardware report, also from an empty `RUN_PROBE` file at boot, for
  device reports.
- A panel mounted on its side is turned in software (the RG28XX).

## [1.4.0](https://github.com/BrandonKowalski/slot/releases/tag/v1.4.0)

- The volume bar shows a headphones icon while they are plugged in.
- Speaker and headphones keep their own volume and mute, and switch the moment you plug / unplug.
- Settings moved from `System/` to a new `Config/` folder, so replacing `System/` on update no longer resets them. slot moves them on first boot.
- The quick menu and power menu wrap from the last item to the first, and back.

## [1.3.0](https://github.com/BrandonKowalski/slot/releases/tag/v1.3.0)

- The SP boots to a slot logo instead of BaseOS's. slot installs `System/bootlogo.bmp` on first run and keeps the original as `bootlogo.baseos.bmp`.

## [1.2.2](https://github.com/BrandonKowalski/slot/releases/tag/v1.2.2)

- Fix for hitch in games caused by autosave. The autosave now writes to the card in the background.

## [1.2.1](https://github.com/BrandonKowalski/slot/releases/tag/v1.2.1)

- Shell colors from Cart Studio now apply to games with accented names.
- Clear GBA carts no longer show their circuit board above the label.

## [1.2.0](https://github.com/BrandonKowalski/slot/releases/tag/v1.2.0)

- Hold X for turbo A and Y for turbo B.
- Carts on the shelf are larger, and shrink into the slot as they go in.
- Carts are drawn at their real proportions, so full label scans fit without cropping.
- Jumping a letter with Up or Down shows the letter in the slot.
- Saves now stick for games whose save is smaller than the core first reports, such as Golden Sun on mGBA.
- The seated cart no longer shows through the slot as a game starts.
- SELECT reaches the game as soon as it can no longer be the start of a shortcut.

## [1.1.0](https://github.com/BrandonKowalski/slot/releases/tag/v1.1.0)

- Fast-forward runs several game frames per refresh, so it is properly fast, and rewind no longer records while fast forwarding.
- The About label credits BaseOS.

## [1.0.0](https://github.com/BrandonKowalski/slot/releases/tag/v1.0.0)

- Game Boy and Game Boy Color, each on its own shelf. L1 and R1 switch shelves, and carts are drawn at their own size in their own plastics.
- Two cores, mGBA and gpSP. Press START on a cart to open it and swap the chip.
- Link play between two RG SPs over WiFi: the link cable and the Wireless Adapter for Pokémon trades and Advance Wars, mGBA's link mode for Mario Kart, and Game Boy link for Tetris.
- A quick menu on the shelf for fast forward speed and sound, rumble, colour correction, the date and time, and About.
- Game Boy games stretch to fill the screen with L1 and go back with R1.
- The real BIOS boot animation plays when the card has one.
- Per-cart shell colors, set in `cart_shell.ini`.
- Up and Down jump through the shelf a letter at a time.
- Your own folder layout on the card is kept, and slot runs on upstream BaseOS.

## [0.1.1](https://github.com/BrandonKowalski/slot/releases/tag/v0.1.1)

- The game holds still behind the power menu.
- Rewind records at every speed and replays exactly what happened.

## [0.1.0](https://github.com/BrandonKowalski/slot/releases/tag/v0.1.0)

- First release: a GBA-only shelf of cartridges with an insert scrape and clunk.
- The game in the slot resumes on the next boot. Closing the lid sleeps and saves.
- Save states with a switcher, rewind, fast-forward, brightness, blue light and volume.
- A power menu on a held POWER, and an About label.
