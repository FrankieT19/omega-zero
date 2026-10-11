# omega-zero UI

The kit omega-zero's menus are drawn from: pixel-art components rebuilt in HTML, the reference for
how each screen looks before and while it is drawn on the device. Open
[`storybook.html`](storybook.html) straight from disk to see every component, what it does and how
to use it, and screens put together from them. [`flow.html`](flow.html) walks every menu screen
and the slides between them.

```
assets/ui/
├── storybook.html         every component and screen, live
├── flow.html              every menu screen, and the slides between them
├── flow/                  what flow.html is drawn and driven with
│   ├── flow.css           the page, and the screen in art pixels
│   ├── screens.js         FlowScreens: what each screen does with a button
│   ├── player.js          the slides, the keys and the URL's hash
│   ├── art.js             FlowArt: placeholders for what Rust draws, the arrows and the …
│   └── stubs.js           stand-ins for omega-row, omega-list-row and omega-bar, if missing
├── foundation/            what every component draws with
│   ├── fonts.css          Pixel Code Regular, cut to Latin-1 and inlined
│   ├── tokens.css         the art pixel (--u) and the shelf's backdrop
│   ├── pixel.js           OmegaPixel: sprites, 9-slices, letters, the base element
│   ├── backdrop.js        OmegaBackdrop: the patterns in Settings > Theme > Background
│   └── palette.js         OmegaPalette: colours from Settings > Theme > Colour
└── components/
    ├── omega-modal.js     <omega-modal>     the menu window, a frame that stacks its content
    ├── omega-panel.js     <omega-panel>     a titled section for inside it
    ├── omega-row.js       <omega-row>       one setting in a panel, lit under the cursor
    ├── omega-list-row.js  <omega-list-row>  a Wi-Fi network or Bluetooth device in a list
    ├── omega-bar.js       <omega-bar>       the bottom bar: two actions, key hints between them
    ├── action-left.js     <action-left>     the bottom bar's left action
    ├── action-right.js    <action-right>    the same, mirrored
    ├── omega-button.js    <omega-button>    a page's link or button, drawn as the bar's pill
    ├── omega-screen.js    <omega-screen>    a screenshot in a handheld's bezel
    ├── omega-card.js      <omega-card>      a feature: an icon, a title and a few lines
    └── omega-code.js      <omega-code>      lines to read or type exactly, scrolled in a box
```

## Using it

A page loads the foundation, in this order, then the components it uses. Custom elements and
classic scripts, no build step, so it works from `file://`.

```html
<link rel="stylesheet" href="foundation/fonts.css">
<link rel="stylesheet" href="foundation/tokens.css">
<script src="foundation/pixel.js"></script>
<script src="foundation/backdrop.js"></script>
<script src="foundation/palette.js"></script>
<script src="components/omega-modal.js"></script>
<script src="components/omega-panel.js"></script>
<script src="components/omega-row.js"></script>   <!-- omega-list-row.js needs it first -->
<script src="components/omega-list-row.js"></script>
<script src="components/action-left.js"></script>
<script src="components/action-right.js"></script>
<script src="components/omega-bar.js"></script>

<div class="omega-backdrop" data-style="plates" data-moving></div>  <!-- Grid without data-style -->

<omega-modal>
  <omega-panel label="BUTTON SETTING" active>
    <omega-row label="JUMP" value="A" lit carets></omega-row>
  </omega-panel>
  <omega-panel label="DEVICES">
    <omega-list-row label="8BitDo SN30 Pro" value="CONNECTED" device="gamepad"></omega-list-row>
  </omega-panel>
</omega-modal>
<omega-bar hint="Y RESET">
  <action-left slot="left" label="CONTROLS" hint="SELECT"></action-left>
  <action-right slot="right" label="SETTINGS" hint="START"></action-right>
</omega-bar>

<script>OmegaPalette.apply('teal')</script>  <!-- Night unless told otherwise -->
```

## Pages around the menus

The last four components are for pages about omega-zero rather than its screens: the site
(`site/`) is drawn with them and the rest of the kit. Each holds the page's own element, so the
words, links, headings and alt text stay real HTML: `<omega-button>` a link or button in the
bar's pill, lit on hover and focus, with a ring for keyboard focus; `<omega-screen>` a screenshot
in a handheld's bezel, 2×2 art pixels kept 2×2 at its own size; `<omega-card>` a feature, its icon
an 11×11 sprite (`cart`, `chip`, `rewind`, `link`, `window`, `gamepad`, `wifi`, `bluetooth`,
`theme`) beside a slotted title; `<omega-code>` a `<pre>` in a dark box that scrolls sideways
instead of the page, with a label and a COPY key if asked. They mark how they draw what they hold
`!important`, so a page's own rules for links, headings and `<pre>` stay off it.

`site/kit` is a link to this folder (and `site/logo` to `assets/logo`), so `site/index.html`
opens from disk like the storybook; `.github/workflows/pages.yml` copies `site/` with the links
followed, and that copy is what GitHub Pages serves, the storybook at `kit/storybook.html`.

## Screens and slides

`flow.html` puts every menu screen together from the kit (the shelf, Settings and everything
under it, CONTROLS, the power menu and the screens after it) and moves between them with the slides the device draws: 280 ms,
smootherstep, every frame on whole art pixels. It is the approved reference for the Rust screens.
Open it from disk like the storybook. What Rust draws itself (the carts, the slot, the top bar,
the About sticker) is a flat placeholder.

Walk it with the keyboard or the buttons under the screen:

| Button | Keys |
|---|---|
| D-pad | arrows |
| A | `A` |
| B | `Z`, `B` or `Esc` |
| X | `X` or `S` |
| Y | `Y` |
| START | `Enter` |
| SELECT | `Space` or right `Shift` |
| POWER, held | `P` |

The sidebar jumps straight to a screen, sets Settings > Theme and the panel size, and has
**slow motion ×5** and **step one frame**, which holds a slide and moves it a frame at a time.
A key pressed during a slide finishes it, then does what it does.

Settings > Network never looks for networks on its own. AVAILABLE NETWORKS lists what is known
without a scan, the network joined (Home, CONNECTED, with no bars or padlock), and the note says
"Press X to look for networks.". X looks once: "Looking for networks…" for a moment, then every
network found with its bars and padlock, which stay until the next scan's answer or until
Network is left. The bar says `X SCAN` while Wi-Fi is on, and `X SCAN` `Y FORGET` on the network
joined. With Wi-Fi off the list is empty and the note says "Wi-Fi is off.".

Settings > Bluetooth is a modal like Network's: BLUETOOTH, its one row switching the radio ON
or OFF with Left and Right, and DEVICES, while it is on the devices paired, then those a look
finds, each with its name in its own case, what it is to the handheld (CONNECTED, CONNECTING,
PAIRING, PAIRED, or nothing for one only found) and its kind's icon
(`<omega-list-row device>`). A device found that gives no name is never listed. Opening the
screen looks for nothing: a note under the paired devices says "Press X to look for devices.",
and X looks for a few seconds, the note saying "Looking for devices…" while what it finds joins
the list, then stops by itself; a look that finds nothing says "No devices found.". What a look
found stays until the screen is left. A on a found device pairs with it and connects (PAIRING,
then CONNECTED), ending a look under way, on a paired one connects and on a connected one
disconnects; Y forgets a paired device. The bar says `X SCAN` while the radio is on, and
`X SCAN` `Y FORGET` while the cursor is on a paired device. With the radio off the note says
"Bluetooth is off.". Settings' BLUETOOTH row shows the connected device's name in capitals,
else ON or OFF.

Holding POWER opens POWER on any screen, a lone panel like THEME's with RESTART and POWER OFF,
deeper from the screen it was held on, which its B goes back to. A on either row goes deeper to
the screen shown until the handheld goes down: RESTART saying "Restarting…" or POWER OFF saying
"Turning off…", a note on its own as SSH's. Nothing can be pressed there, so the whole bar sinks
away as it slides in, `even(p·38)` down, as it does on the device when a cart goes in.

The URL's hash says what is shown, and the page keeps it up to date as you walk, so a link or a
headless Chrome screenshot lands on the same frame:

| Key | Values |
|---|---|
| `screen` | `shelf`, `settings`, `controls`, `network`, `ssh`, `keyboard`, `bluetooth`, `labels`, `datetime`, `background` (THEME), `about`, `power` (POWER held), `restarting`, `poweroff` |
| `from` | slide into `screen` from this one, deeper or back as the tree has it; `dir=in` or `dir=out` forces the way. For `power`, the screen POWER was held on (the shelf when there is none) |
| `t` | with `from`: hold the slide at this time, 0 to 1 |
| `size` | `640x480` (default), `720x480`, `720x720` |
| `colour` | `night` (default), `slate`, `navy`, `teal`, `green`, `plum`, `crimson`, `amber` |
| `style`, `moving` | the backdrop, as Settings > Theme sets it: `style=plates`, `moving=0` |
| `font` | `thin`: the bottom bar's labels in the thin font, as Settings > Theme > Font sets it |
| `wifi` | `off`: SSH shows what it says offline, and Network lists no networks |
| `scan` | Network's scan: `none` (default, as it opens), `running` (looking, standing still) or `done` (the networks found) |
| `bt` | Bluetooth: `on` (default) as the screen opens, before any look, with the two paired devices; `off`; `empty` (nothing paired, before any look); `looking` (a look under way, held, the keyboard found so far); `found` (after it, the band found too); or `pairing` (after it, the keyboard pairing, the cursor on it) |
| `row` | the cursor's row on `screen`, from 0 |
| `slow` | `1`: slides at a fifth of their speed |
| `shot` | `1`: the screen alone at the window's top left, for screenshots |

```
flow.html#screen=network&from=settings&t=0.5&colour=amber&shot=1
```

is Settings halfway through sliding out to the right for Network, in Amber, ready for a
screenshot the size of the panel.

## The rules

These are what make it look like one thing. Keep to them when changing or adding to the kit.

- **One art pixel is `--u`, 2px.** omega-zero draws pixel art 2×2, so a 640×480 panel is 320×240
  art pixels. Every size is a whole number of `--u`; nothing lands on a half pixel.
- **Every shape is a sprite**, written in its component as rows of colour keys, one character to an
  art pixel (see `CORNER` in `omega-modal.js`, `ARROW` in `action-left.js`), and painted by
  `OmegaPixel.sprite()` with `image-rendering: pixelated`. No `border-radius`, no smooth curves, no
  filters. A shape that fits any size is a 9-slice: draw its top-left quarter, `mirror()` the rest.
- **Text sits on whole pixels.** Pixel Code Regular at 18px is one face pixel to an art pixel; set
  it on a 20px line so its capitals land on rows 2 to 8. Pixel Code Bold is fattened off the grid:
  draw bold instead, the word again one pixel to the right. A word drawn bigger or stretched goes
  into a sprite through `OmegaPixel.letters()`, never through a CSS transform (a transformed line's
  baseline falls between pixels). Every component sets `text-rendering: geometricPrecision` on its
  host: Chrome's default hinting smears Pixel Code's edges by part of a pixel even where it sits on
  whole pixels, at 1× and on a 2× display alike.
- **Colours come from the backdrop colour.** Use only the `--oz-*` properties `palette.js` sets.
  Night (the default) gives plain greys, Amber gives Mega Man Zero's own colours, the rest turn
  that ramp to their hue. A component that paints sprites extends `OmegaPixel.Element` and draws in
  `paint()`, which runs again whenever the palette changes.
- **Pages put components on whole pixels too.** Centring something in a box with an odd number of
  pixels to spare puts it half a pixel off, and its text blurs (borders and sprites hide it, text
  does not). Round the offset, as `main` in `storybook.html` does.
- **Each area is its own div**, named with `part` so a page can reach it with `::part()`.
- **Space is even.** Text gets the same room above as below inside its box; a title starts from
  its tab's left edge.

## Adding a component

1. Write `components/<name>.js` as a custom element extending `OmegaPixel.Element`, its shapes as
   sprite rows at the top of the file and its sizes in art pixels.
2. Draw its sprites in `paint()`; attributes that change its text call the part of `paint()` that
   draws it.
3. Give it a card in `storybook.html`: what it does, a live example (empty and filled when it takes
   text), its attributes and parts, and the markup to use it.
4. Check it zoomed in at 1× and 2×: every edge on a whole art pixel, nothing blurred.
