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
- [Wi-Fi, SSH and SFTP](#wi-fi-ssh-and-sftp)
- [Cart labels](#cart-labels)
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
- Home Wi-Fi from the quick menu, with an on-screen keyboard for the passphrase. Once joined,
  copy games over SFTP and let the clock set itself from the network.
- [Real cart labels](#cart-labels), scanned from real carts, fetched over Wi-Fi from
  ScreenScraper with your own account.
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
| MENU | Settings, including [Wi-Fi](#wi-fi-ssh-and-sftp) |

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
| `Config` | Settings, and `wifi.conf` with the Wi-Fi networks you have joined |
| `System` | slot itself, its cores and its licences. Replaced on every update |

slot can fetch [labels](#cart-labels) itself over Wi-Fi. They can also be made with the
upstream author's [slot Cart Studio](https://studio.slot-cfw.fyi). Per-cart shell colours go in
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

## Wi-Fi, SSH and SFTP

### Joining a network

1. On the shelf, press MENU, go down to **Wi-Fi** and press A.
2. Press A again to switch Wi-Fi on. The networks in reach appear under it after a few seconds.
3. Pick yours and press A. If it needs a passphrase, type it on the keyboard and press START.

| On the keyboard | Does |
|---|---|
| D-pad | Move between keys |
| A | Type the key, or press Shift, #+=, Space, Delete or Done |
| B | Delete the last character. On an empty field, go back to the list |
| X | Shift: capitals, and back |
| Y | Space |
| L1 / R1 | Symbols, and back |
| START | Done |
| MENU | Go back to the list |

slot remembers each network you join and joins it again at the next boot, as long as Wi-Fi was
left on. On the list, Y forgets a saved network.

The networks are kept on the card in `Config/wifi.conf`, in
[wpa_supplicant](https://w1.fi/wpa_supplicant/)'s format. You can add one from a computer:

```
network={
	ssid="My Network"
	psk="my passphrase"
}
```

The passphrase is stored as plain text, like BaseOS's SSH password, so keep that in mind
before lending the card out. Networks that need a username (enterprise Wi-Fi such as eduroam)
are not supported.

### Copying games over SSH and SFTP

Once joined, the Wi-Fi screen shows the handheld's address and name, such as
`root@rg35xxsp.local`. BaseOS runs an SSH server with SFTP on every boot.

- **User:** `root`. **Password:** `root`, unless you set `ssh_password=` in `baseos.conf` at
  the root of the card in SD1 (see [BaseOS's settings](https://github.com/pvaibhav/BaseOS#settings)).
- The card slot runs from is `/mnt/sdcard`, so games go in `/mnt/sdcard/Games/GBA` and so on.
- Use any SFTP client, such as [Cyberduck](https://cyberduck.io),
  [FileZilla](https://filezilla-project.org) or `scp`.

```sh
scp "Metroid Fusion.gba" root@rg35xxsp.local:/mnt/sdcard/Games/GBA/
```

The name is the model's own (`rg35xxsp`, `rgsp`, ...) unless you set `hostname=` in
`baseos.conf`. If `.local` names don't resolve on your network, use the address shown.

### Network time

**Network Time**, on the Wi-Fi screen, keeps the clock right whenever Wi-Fi is joined. The time
zone stays the one you chose under Date & Time.

### Wi-Fi and link play

A link uses the same radio, so it takes over from home Wi-Fi while it lasts and hands it back
when it ends.

## Cart labels

A cart with no label of its own carries its name. slot can fetch the real thing: the scan of the
label on a real cart, from [ScreenScraper](https://www.screenscraper.fr), a free database run by
its community. You need a ScreenScraper account (free) and Wi-Fi joined.

1. On the Wi-Fi screen, pick **Cart Labels**.
2. On **ScreenScraper**, press A and type your user name, then your password, on the keyboard.
3. On **Fetch Labels**, press A.

slot looks up each cart that has no label, by its ROM's checksum, so a renamed file is still
found. It takes the label from the region the ROM is from, (USA), (Europe) or (Japan), and from
another region when that one was never scanned. Each label goes on the shelf as it arrives, into
`Labels/GBA`, `Labels/GB` or `Labels/GBC` under the ROM's name. A label you put there yourself is
never replaced, and a cart ScreenScraper has no scan for, such as a ROM hack, keeps its name.

Your account is kept in `Config/screenscraper.conf`, as plain text: anyone with the card can read
it. Y on **ScreenScraper** signs out and deletes it. A free account can look up thousands of carts
a day, and the screen shows how many are left.

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
- **"The Wi-Fi radio did not start"** means BaseOS never brought up `wlan0`. Restart once;
  `baseos-boot.log` on the card says what the driver did. The RG35XX 2024 shares its BaseOS
  image with the Plus but has no radio.

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

Cart labels need ScreenScraper's key for the program, which is not in the source. Set
`SLOT_SS_DEVID` and `SLOT_SS_DEVPASSWORD` when building to put it in; releases get it from
the repository's secrets. A build without it says so on the Cart Labels screen.

The device build compiles the cores in an arm64 Linux container. gpSP's compiler needs more
memory than a 2 GB Docker VM has; with colima, `colima start --memory 4`.

`task screens` draws slot's main screens headless to PNGs, and `task screens:compare` fails
on any pixel that differs from a baseline: the way to prove a change leaves a layout alone.

## Roadmap

- Validate everything on the RG35XX SP: buttons, lid, rumble, LEDs, audio, link play, Wi-Fi.
- Bluetooth headphones and controllers.
- Cart labels on the device, once ScreenScraper has given slot its key.
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
