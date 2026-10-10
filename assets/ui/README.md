# omega-zero UI

The kit omega-zero's menus are drawn from: pixel-art components rebuilt in HTML, the reference for
how each screen looks before and while it is drawn on the device. Open
[`storybook.html`](storybook.html) straight from disk to see every component, what it does and how
to use it, and screens put together from them.

```
assets/ui/
├── storybook.html       every component and screen, live
├── foundation/          what every component draws with
│   ├── fonts.css        Pixel Code Regular, cut to Latin-1 and inlined
│   ├── tokens.css       the art pixel (--u) and the shelf's backdrop
│   ├── pixel.js         OmegaPixel: sprites, 9-slices, letters, the base element
│   └── palette.js       OmegaPalette: colours from Settings > Background > Colour
└── components/
    ├── omega-modal.js   <omega-modal>   the menu window, a frame that stacks its content
    ├── omega-panel.js   <omega-panel>   a titled section for inside it
    ├── action-left.js   <action-left>   the bottom bar's left action
    └── action-right.js  <action-right>  the same, mirrored
```

## Using it

A page loads the foundation, in this order, then the components it uses. Custom elements and
classic scripts, no build step, so it works from `file://`.

```html
<link rel="stylesheet" href="foundation/fonts.css">
<link rel="stylesheet" href="foundation/tokens.css">
<script src="foundation/pixel.js"></script>
<script src="foundation/palette.js"></script>
<script src="components/omega-modal.js"></script>

<omega-modal>
  <omega-panel label="BUTTON SETTING" active></omega-panel>
</omega-modal>

<script>OmegaPalette.apply('teal')</script>  <!-- Night unless told otherwise -->
```

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
  baseline falls between pixels).
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
