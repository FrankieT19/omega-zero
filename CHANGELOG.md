# Changelog

## Unreleased

- A bar along the bottom of the shelf: SELECT QUICKMENU on the left, START SETTINGS on the
  right. The slot sits a bar higher to make room for it.
- Game Boy carts stand at 80% of their size on the shelf and grow to full size as they go into
  the slot.
- START on the shelf opens Settings, a window in the style of Mega Man Zero's menus, in place of
  the old quick menu. CONFIGS holds Fast Forward (2X, 3X, 4X or 6X), Fast Forward Sound, Colour
  Correction, Rumble and Labels; GENERAL holds Date & Time, Network, Background and About.
- A row that Left and Right change shows ◀ ▶ while it is lit. A row that opens a screen ends
  in ›.
- A tap of SELECT on the shelf opens QUICKMENU: what every button does, on the shelf and in a
  game, on one panel. SELECT held with another button works as before.
- The Wi-Fi screen is now Network, under Settings. Its NETWORK panel switches Wi-Fi and Network
  Time and opens SSH. AVAILABLE NETWORKS shows each network's signal, a padlock on those with a
  passphrase, and CONNECTED, JOINING or SAVED. Y forgets a saved network.
- SSH shows how to log in from a computer: the address, `<name>.local`, user `root`, the
  password and port 22. SFTP uses the same login. The password is BaseOS's: `root`, or what
  `ssh_password=` says in `baseos.conf` on the card.
- Cart Labels is no longer on the Wi-Fi screen: it is Labels, in Settings.
- Background, Date & Time and Labels are panels over the background, and the passphrase
  keyboard is a window. About is the same sticker as before.
- In the menus the bottom bar says B RETURN and A CONFIRM. A screen's own keys sit in the middle
  of it: Y FORGET on Network, START FETCH and Y SIGN OUT on Labels.
- Every move between screens slides, over a background that keeps moving. Going into a screen,
  the one you leave goes out to the right and the new one comes in from the left; coming back is
  the mirror. Leaving the shelf, the top bar goes up, the carts right and the slot down.
- The menus, both bars and the slot take their colours from Settings > Background > Colour.
  Night is grey; Amber is Mega Man Zero's own colours.

### For developers

- `assets/ui/flow.html` walks every new screen with the slides the device draws, and is the
  reference they are built to. Keys stand in for the buttons, slow motion runs the slides five
  times slower, and the URL's hash picks the screen, colour, panel size and any moment of a
  slide: `flow.html#screen=network&from=settings&t=0.5&colour=amber&shot=1`.
- The storybook has three new components: `<omega-row>`, `<omega-list-row>` and `<omega-bar>`.
- The kit is in Rust too, in `crates/omega-ui/src/kit/` (palette, pixel, modal, panel, action,
  row, list_row, bar, text, layout). `tests/suite/kit/` holds it to Chrome's screenshots of the
  HTML kit pixel for pixel; `make-refs.sh` takes them again.
- Each menu screen's input and drawing is a file of its own in `crates/omega-zero/src/screens/`,
  its faces are made in `frontend/sync_*.rs`, and `nav.rs` slides between them.

## omega-zero 0.0.2

- A cart leaning half off the screen is drawn whole on every GPU: some drivers dropped it for a
  frame. Turning carts go through a shader of their own and everything else through the one it
  always had, so the shelf costs no more to draw than before carts turned.

### For developers

- `task deploy:wifi` builds omega-zero, copies it onto a console on the home network over SSH
  and reboots into it; `task log:wifi` prints its log. `PROFILE=device-dev` builds in seconds.
- Each crate's tests build as one binary, and the suite runs in about 42 s instead of 130 s.

## omega-zero 0.0.1

The first release, for the RG35XX SP and the other H700 handhelds BaseOS runs on.

### Every panel, natively

- omega-zero lays itself out for the panel it finds, from the model BaseOS names: 640x480 on
  the RG35XX SP instead of the RG SP's 720x480 picture shrunk to fit.
- On a 640x480 panel, GBA games fill the width (640x427) through a sharp scaler, over the
  console's body carrying the GAME BOY ADVANCE logo. Game Boy games stay at 3x through the LCD
  mask.
- Your own overlays: a `.png` the size of the screen in `Overlays/GBA`, `Overlays/GB` or
  `Overlays/GBC`, laid over the game. One that would cover the game is ignored.
- The boot logo is only ever replaced with one the size of the device's own. Logos ship for
  640x480, 720x480 and 720x720 panels.
- Rumble works on devices whose motor is the PMIC's `moto` pin rather than a force feedback
  node.
- Hosting a link works on devices whose Wi-Fi driver gives no second interface.
- A panel mounted on its side is turned in software (the RG28XX).
- `System/probe.sh` writes a hardware report, also from an empty `RUN_PROBE` file at boot, for
  device reports.

### Wi-Fi, SSH and labels

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

### A new look

- Every word on screen is in Pixel Code, a pixel font set on its own grid so it stays sharp.
- A top bar over the shelf: Wi-Fi, Bluetooth and ssh on the left, lit while they are up; the
  clock in the middle; the battery on the right.
- The shelf's background is yours to pick under START > Background: a grid, raised plates
  (riveted or not), dotted lines, a dithered grid, bits or a plain colour, drifting or still, in
  eight colours, every pattern in the same 2x2 pixels as the words over it. A wallpaper on the
  card still works.
- The slot below the shelf is bigger: a cart slides into it at full size instead of shrinking.
- The quick menu scrolls when its rows outgrow the screen.
- On the shelf START opens the quick menu and X the core picker; MENU opens nothing there.
- The carts on the shelf are solid. The one you are on floats and sways ten degrees either way,
  showing its edge, and every cart leans into a scroll. It squares up as it slides into the
  slot or opens for the core picker.
- The boot logo is the OMEGA ZERO title in pixel art, edged in white, over the shelf's grid.
  Its source is `assets/logo/omega-zero-logo.html`; `task bootlogos` renders it for every
  panel, and `STYLE=smooth` renders the drawn-straight version instead.
