# Changelog

## Unreleased

- The carts are low-poly models drawn the way a PlayStation drew 3D: a face and a side for
  every cut of the outline, each lit on its own, corners that land on the 2-pixel grid as they
  turn, textures mapped without perspective, and colour in 15 bits through the PlayStation's
  dither. The one you are on sways twenty degrees either way, seen from closer than before, so
  its side and the step under a GBA cart's grip show. The cart going into the slot and the core
  picker's lid are the same model. A cart is one draw instead of up to seventeen.
- The volume, brightness, blue light and rewind show as a gauge drawn like the menus: the
  level's name in a tab, then a cell for each step, lit as far as the level reaches, and its
  icon, on the 2-pixel grid in the colours of Settings > Theme, with no dark band behind it. It
  stands at the top of the screen, on the shelf just under the top bar, and fades out in four
  steps. What just happened shows the same way, a small panel titled SAVE STATE, LINK or COLOUR
  over "State saved", "Link ended", "Correction on" and the rest.
- Fast forward's badge in the corner is a key cap holding ▶▶ and the speed, ×2 to ×6, lit
  while it is latched. A link's badge shows a link, lit while hosting and dark when joined, and
  broken once the other handheld is lost.
- A bar along the bottom of the shelf: SELECT CONTROLS on the left, START SETTINGS on the
  right. The slot sits a bar higher to make room for it.
- Game Boy carts stand at 80% of their size on the shelf and grow to full size as they go into
  the slot.
- The boot logo fades into the shelf. The shelf's first frames show the same logo, where the
  handheld drew it, and it fades out over half a second to the shelf already moving under it.
  Any button ends the fade at once. A boot back into a game, or to the clock, cuts as before.
- START on the shelf opens Settings, a window in the style of Mega Man Zero's menus, in place of
  the old quick menu. CONFIGS holds Fast Forward (2X, 3X, 4X or 6X), Fast Forward Sound, Colour
  Correction, Rumble and Labels; GENERAL holds Date & Time, Network, Theme and About.
- A row that Left and Right change shows ◀ ▶ while it is lit. A row that opens a screen ends
  in ›.
- A tap of SELECT on the shelf opens CONTROLS: what every button does, on the shelf and in a
  game, on one panel. SELECT held with another button works as before.
- The Wi-Fi screen is now Network, under Settings. Its NETWORK panel switches Wi-Fi, Network
  Time and SSH. AVAILABLE NETWORKS shows each network's signal, a padlock on those with a
  passphrase, and CONNECTED, JOINING or SAVED. Y forgets a saved network.
- Network looks for networks only when X is pressed, once each time, and the bar says X SCAN.
  Until then it lists the network joined and the saved ones, with a note to press X. It no
  longer scans every 12 seconds while it is open, which cost battery and, on the RG35XX SP,
  came before reboots that sometimes hung.
- SSH steps through ON, OFF and INFO. ON starts the SSH server and OFF stops it, leaving a
  login already made to carry on; the card remembers which. A on INFO shows how to log in from a
  computer: the address, `<name>.local`, user `root`, the password and port 22. SFTP uses the
  same login.
- The first time omega-zero runs on a card whose `baseos.conf` sets no `ssh_password`, it gives
  root a password of six random digits, at once and in `baseos.conf`, so BaseOS keeps it. A
  password you set there yourself is kept.
- Cart Labels is no longer on the Wi-Fi screen: it is Labels, in Settings.
- Theme, Date & Time and Labels are panels over the background, and the passphrase
  keyboard is a window. About is the same sticker as before.
- In the menus the bottom bar says B RETURN and A CONFIRM. A screen's own keys sit in the middle
  of it: Y FORGET on Network, START FETCH and Y SIGN OUT on Labels. Between its two ends there is
  nothing but those keys, over the background.
- Every move between screens slides, over a background that keeps moving. Going into a screen,
  the one you leave goes out to the right and the new one comes in from the left; coming back is
  the mirror. Leaving the shelf, the top bar goes up, the carts sideways and the slot down.
  Settings, from the shelf, slides the other way round: the carts go out to the left as it comes
  in from the right, and it goes back out to the right. Inside Settings every screen slides the
  usual way.
- The menus, both bars and the slot take their colours from Settings > Theme > Colour.
  Night is grey; Amber is Mega Man Zero's own colours.
- Theme is what Background was: the background's pattern, its motion and its colour. It also
  picks the font of the bottom bar's big words, Bold or Thin.
- Restart no longer freezes the console after Settings > Network has been open. The RTL8821CS's
  Wi-Fi driver hung the kernel's shutdown when it had just gone into its power saving, as it does
  after scanning for networks, so the console stayed frozen until it was reset. omega-zero now
  lets go of the driver itself before a restart or power off: wpa_supplicant and the DHCP client
  stop, the interfaces go down, and the device is unbound from its driver.
- The top bar lights Wi-Fi and ssh within a second of a network being joined, and puts them out
  as quickly when it is lost. After boot they stayed dark until a menu was opened.
- Bluetooth, under Network in Settings, on handhelds that have it. Its screen switches it on and
  off and, while it is on, lists the devices paired, then those a look finds. A pairs with a new
  device and connects it, connects a paired one, or disconnects one connected; Y forgets a
  paired one. Settings names the device connected. The card remembers whether Bluetooth is on,
  bringing it up at boot, and the top bar lights its icon while it is. Pairing asks nothing on
  screen, which most gamepads and headphones take. Game sound over Bluetooth headphones and
  Bluetooth gamepads as controllers are still to come.
- The Bluetooth screen looks for devices only when asked: it opens on the paired devices and
  "Press X to look for devices.", and X SCAN looks for 12 seconds, saying "Looking for devices…"
  while the devices it finds join the list, which keeps them until the screen is left. Devices
  are listed by name: one found that gives none, most often a beacon nobody can pair with, is
  left out rather than shown by its address.
- Holding POWER opens POWER, a panel like the other menus over the background, with RESTART and
  POWER OFF and B RETURN and A CONFIRM on the bar; B goes back to where you were, a game
  included. After a choice, in place of the black screen, a RESTART or POWER OFF panel says
  "Restarting…" or "Turning off…" until the handheld goes down, and the bar sinks away.

### For developers

- `assets/ui/flow.html` walks every new screen with the slides the device draws, and is the
  reference they are built to. Keys stand in for the buttons, slow motion runs the slides five
  times slower, and the URL's hash picks the screen, colour, panel size and any moment of a
  slide: `flow.html#screen=network&from=settings&t=0.5&colour=amber&shot=1`.
- The storybook has three new components: `<omega-row>`, `<omega-list-row>` and `<omega-bar>`.
- And four for pages, which the site is now drawn with: `<omega-button>`, `<omega-screen>`, `<omega-card>`, `<omega-code>`.
- The kit is in Rust too, in `crates/omega-ui/src/kit/` (palette, pixel, modal, panel, action,
  row, list_row, bar, text, layout). `tests/suite/kit/` holds it to Chrome's screenshots of the
  HTML kit pixel for pixel; `make-refs.sh` takes them again.
- Each menu screen's input and drawing is a file of its own in `crates/omega-zero/src/screens/`,
  its faces are made in `frontend/sync_*.rs`, and `nav.rs` slides between them.
- `card/System/bluetooth.sh` drives the radio for omega-zero (up, down, status, scan on or off,
  pair, connect, disconnect, forget) and prints each device as a tab-separated line;
  `card/test-bluetooth.sh` tests it against fakes of BlueZ's tools.
- The daemons wifi.sh and bluetooth.sh leave running, and a dropbear omega-zero starts again,
  no longer hold the framebuffer open or share omega-zero's session and signals.
- Now that root's password is random, `task deploy:wifi` and `task log:wifi` log in with
  `CONSOLE_PASSWORD` or, without it, the first line of a git-ignored `.console-password` at the
  top of the repo.

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
