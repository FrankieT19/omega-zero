# slot.

A bespoke, Game Boy-centric frontend for Anbernic H700 handhelds running
[BaseOS](https://github.com/pvaibhav/BaseOS). It plays GBA, GBC and GB games, and only those,
on a shelf of cartridges you slide into a slot.

This is a fork of [BrandonKowalski/slot](https://github.com/BrandonKowalski/slot), which was
made for the Anbernic RG SP. The fork brings slot to the **RG35XX SP** and the rest of the
H700 family, laid out natively for each panel instead of shrinking the RG SP's picture.

Release notes are in the [changelog](CHANGELOG.md).

## Contents

- [Supported devices](#supported-devices)
- [Features](#features)
- [Installing](#installing)
- [Updating](#updating)
- [Controls](#controls)
- [The card](#the-card)
- [Overlays](#overlays)
- [Link play](#link-play)
- [Helping with a device](#helping-with-a-device)
- [Troubleshooting](#troubleshooting)
- [Building from source](#building-from-source)
- [Roadmap](#roadmap)
- [Credits and licences](#credits-and-licences)

## Supported devices

One build runs on all of them. slot reads the model from BaseOS at boot and picks the layout
for its panel.

| Device | Panel | Status |
|---|---|---|
| RG35XX SP | 640x480 | Primary target of this fork. Native layout done, hardware validation in progress. |
| RG SP | 720x480 | The device slot was first written for. Unchanged, pixel for pixel. |
| RG34XX SP, RG34XX | 720x480 | Same layout as the RG SP. Untested. |
| RG35XX Plus / 2024, H, Pro | 640x480 | Same layout as the RG35XX SP, without the lid. Untested. |
| RG40XX H, RG40XX V | 640x480 | Same layout as the RG35XX SP. Untested. |
| RG CubeXX | 720x720 | Experimental: the square layout is not finished. |
| RG28XX | 480x640, turned | Experimental: the panel is mounted on its side and is turned in software. Untested. |

"Untested" means nobody has reported back yet. If you own one, a
[device report](#helping-with-a-device) is the most useful thing you can send.

## Features

- A shelf of carts drawn at their real proportions in their own plastics, with your labels on
  them. GBA, Game Boy and Game Boy Color each get a shelf.
- Two GBA cores, mGBA and gpSP, swapped per cart by opening the cart and changing the chip.
- The game you left in the slot resumes on the next boot. Closing the lid saves and sleeps.
- Save states with a switcher, rewind, fast-forward with turbo speeds, turbo A and B.
- Link play between two devices over Wi-Fi: the link cable and the Wireless Adapter for Pokémon
  trades and Advance Wars, mGBA's link for Mario Kart, Game Boy link for Tetris.
- Brightness, blue light, colour correction, rumble, and separate volumes for the speaker and
  headphones.
- On a 640x480 panel, GBA games fill the width (640x427) through a sharp scaler, with the
  console's body below them carrying GAME BOY ADVANCE, as under a real GBA's screen. Game Boy
  games stay at exactly 3x through an LCD mask.
- Your own [overlays](#overlays), and the real BIOS boot animation if your card has the BIOS.

## Installing

slot runs on BaseOS, a minimal Linux for H700 handhelds. You need the BaseOS image for your
exact model.

### On one card

1. Download the BaseOS image for your device from the
   [BaseOS releases](https://github.com/pvaibhav/BaseOS/releases/latest) (for example
   `baseos-rg35xxsp-*.img.zip`) and write it to a microSD card with a tool such as
   [Raspberry Pi Imager](https://www.raspberrypi.com/software/).
2. Put the card in the slot marked **SD1** (TF1) and power on. BaseOS expands its storage, then
   says no frontend was found. Power off.
3. Download the latest [slot release](https://github.com/oauramos/slot-rg35xxsp/releases/latest),
   unzip it, and copy the contents of the `slot-vX.Y.Z/` folder to the root of the card.
4. Put your games in `Games/GBA`, `Games/GB` and `Games/GBC`. Labels are optional; see
   [the card](#the-card).
5. Put the card back in SD1 and boot.

### On two cards

1. Write BaseOS to the card in **SD1** as above.
2. Format a second card as FAT32 or exFAT with an MBR partition table, and copy the contents of
   the release's `slot-vX.Y.Z/` folder to it.
3. Add your games to the second card and put it in **SD2**. BaseOS runs slot from SD2 whenever
   one is there.

## Updating

Replace the `System/` folder on the card slot runs from with the one from the new release.
Your games, saves, save states, labels, wallpapers, overlays and settings all live outside
`System/`.

> **Updating from a version before 1.4.0?** Those kept settings in `System/`. Copy the new
> `System/` over the old one instead of deleting it the first time, and slot moves your
> settings to `Config/`.

## Controls

### Everywhere

| Buttons | Does |
|---|---|
| SELECT + Up / Down | Brightness |
| SELECT + Left / Right | Blue light |
| VOL+ / VOL- | Volume |
| VOL+ and VOL- together | Mute |
| Hold POWER | Power menu |

### On the shelf

| Buttons | Does |
|---|---|
| Left / Right | Move along the shelf |
| Up / Down | Jump to the next letter |
| L1 / R1 | Switch between GBA, Game Boy and Game Boy Color |
| Tap A | Insert the cart and resume its last save state |
| Hold A | Insert the cart and start the game fresh |
| START | Open the cart and choose its core (GBA only) |
| MENU | Settings |

### In a game

| Buttons | Does |
|---|---|
| Hold MENU | Save a state, eject the cart, back to the shelf |
| Double tap MENU | Save state switcher |
| SELECT + MENU | Link screen |
| SELECT + R1 | Save a state |
| SELECT + L1 | Load the most recent save state |
| Hold L2 | Rewind |
| Hold R2 | Fast-forward. Double tap R2 to lock it on, again to unlock |
| Hold X / Y | Turbo A / turbo B |
| L1 / R1 | Stretch to fill the screen / original size (Game Boy and Game Boy Color) |

### In the save state switcher

| Buttons | Does |
|---|---|
| Left / Right | Flick through the states |
| A | Load the one on screen |
| Y | Delete the one on screen |
| X | Undo the last save or load, up to 30 seconds after |
| B | Back to the game |

Closing the lid saves a state and turns the screen off; opening it carries on. Left shut for
three minutes, slot powers off and resumes from that state on the next boot.

## The card

slot creates these folders on first boot. Nothing loose at the top of `Games/`, `Saves/`,
`Labels/` or `States/` reaches the shelf.

| Folder | Holds |
|---|---|
| `Games/GBA`, `Games/GB`, `Games/GBC` | Your legally obtained ROMs |
| `Labels/GBA`, `Labels/GB`, `Labels/GBC` | Cart labels: a `.png` named after the ROM, for example `Labels/GBA/Metroid Fusion.png` for `Games/GBA/Metroid Fusion.gba` |
| `Saves/...`, `States/...` | Battery saves and save states, per platform |
| `BIOS` | `gba_bios.bin`, `gbc_bios.bin`, `gb_bios.bin` for the boot animations (optional) |
| `Wallpapers` | `.png` wallpapers for the shelf |
| `Overlays/GBA`, `Overlays/GB`, `Overlays/GBC` | Your [overlays](#overlays) |
| `Config` | Settings |
| `System` | slot itself, its cores and its licences. Replaced on every update |

Labels can be made with the upstream author's
[slot Cart Studio](https://studio.slot-cfw.fyi). Per-cart shell colours go in
`Config/cart_shell.ini`.

## Overlays

An overlay is a `.png` the size of your screen, laid over the game and its bezel. Name it after
the screen size and put it in the folder for its platform:

- `Overlays/GBA/640x480.png` on a 640x480 device such as the RG35XX SP
- `Overlays/GB/720x480.png` on a 720x480 device such as the RG SP

It must be transparent where the game's picture is. slot checks, and ignores an overlay that
would cover more than 1% of the game, so one made for another layout never hides it (the log
says so). On a 640x480 panel, the game's picture is:

| Platform | Picture |
|---|---|
| GBA | 640x427 at the top of the screen |
| Game Boy and Game Boy Color | 480x432 in the middle (3x), or the whole screen when stretched |

Overlays made for RetroArch on 640x480 handhelds, such as
[Perfect Overlays](https://github.com/ourigen/perfect_overlays), use the same Game Boy picture
and work for it. slot ships no overlays of its own.

## Link play

Two devices running slot can link over Wi-Fi with no router: one hosts and the other joins.

1. Open the cart and pick **gpSP** for Pokémon (START on the shelf).
2. In the game, press SELECT + MENU and choose the link.
3. One player picks **HOST**, the other **JOIN**.

Pokémon Emerald, FireRed and LeafGreen link, and so do both Advance Wars. mGBA's link
mode plays Mario Kart: Super Circuit, and Game Boy link plays Tetris.

## Helping with a device

slot ships a probe that writes down how your handheld is wired: its buttons, panel, battery,
lid, LEDs, audio, Wi-Fi and Bluetooth.

1. Create an empty file named `RUN_PROBE` at the root of the card slot runs from.
2. Boot. slot writes `probe-<device>-<date>.txt` next to it and deletes `RUN_PROBE`.
3. Open a [device report](https://github.com/oauramos/slot-rg35xxsp/issues/new?template=device-report.yml)
   with the file attached, and say what worked.

Over SSH (BaseOS's default login is `root` / `root`), `sh /mnt/sdcard/System/probe.sh` does the
same.

## Troubleshooting

- **slot.log** at the root of the card holds the last session's log, and `slot.log.1` the one
  before. The first lines name the device, panel and canvas slot picked.
- **Crashes** leave `crash/cores.log` and a core dump on the card.
- **The boot logo** is only ever replaced with one the size of your device's own. The original
  is kept on the boot partition as `bootlogo.baseos.bmp`.
- **Button presses** can be traced to `input-trace.log` by running slot with
  `SLOT_TRACE_INPUT=1`.

## Building from source

You need Rust (the version is pinned in `rust-toolchain.toml`), [Task](https://taskfile.dev)
and, to build for the device on a machine that is not Linux on arm64, Docker.

```sh
task run                      # build the cores and run slot on the desktop
SLOT_CANVAS=640x480 task run  # see what a 640x480 device draws
task check                    # format, lint, tests on every canvas, card scripts
task dist:device              # the aarch64 card tree, in dist-device/
task deploy:device            # push to a device over adb and restart it
```

`task screens` draws slot's main screens headless to PNGs, and `task screens:compare` fails
on any pixel that differs from a baseline: the way to prove a change leaves a layout alone.

## Roadmap

- Validate everything on the RG35XX SP: buttons, lid, rumble, LEDs, audio, link play.
- Wi-Fi on your home network, SSH/SFTP to copy games, and the clock set over the network.
- Bluetooth headphones and controllers.
- The other H700 devices: analogue sticks, RGB LEDs, the square CubeXX layout, the RG28XX's
  turned panel.
- Full documentation in `docs/`, published as a wiki.

## Credits and licences

- slot was created by [Brandon T. Kowalski](https://github.com/BrandonKowalski). This fork
  keeps his design and adds device support.
- [BaseOS](https://github.com/pvaibhav/BaseOS) by pvaibhav.
- Emulation by [mGBA](https://mgba.io) (MPL-2.0) and [gpSP](https://github.com/libretro/gpsp)
  (GPL-2.0). Their licences, patches and gpSP's source ship in `System/licenses/`.
- Type is [Open Sans](https://github.com/googlefonts/opensans) and the
  [Nerd Fonts](https://www.nerdfonts.com) symbols.
- The LCD mask is derived from GigaHerz's LCD3x.

slot is licensed under the GPL-3.0-or-later; see [LICENSE](LICENSE).

Game Boy, Game Boy Color and Game Boy Advance are trademarks of Nintendo. Anbernic is a
trademark of its owner. This project is not affiliated with either.

### AI disclosure

The upstream Rust frontend was put together by Claude and reviewed by its author. This fork's
device support was written with Claude, and is reviewed and tested by its maintainer.
