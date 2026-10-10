/*
  <action-right>: the bottom bar's right action, as the game's R1 BUTTON / ELF LIST. <action-left>
  mirrored: left to right, a button pill with a hint, a big label, and the arrow in its notch at
  the screen's edge. It comes without text; give it some with attributes when it is used:

    <action-right></action-right>
    <action-right label="SETTINGS" hint="START"></action-right>

  label  the big word beside the arrow, drawn as a sprite: Pixel Code made bold the pixel-art
         way (drawn again a pixel to the right) and stretched to twice its height, each face pixel
         one art pixel wide and two tall, with a shadow down and to the right.
  hint   the smaller word in the button pill, in Pixel Code.

  Everything is pixel art at one art pixel to --u, text included. It is 21 art pixels tall and as
  wide as its text (with room kept for it when there is none). The notch rises 6 pixels above it,
  over whatever is there, as the game's does over the backdrop. Areas, each a div that ::part()
  reaches: action, button, hint, label, word, icon.

  Needs everything in foundation/ on the page (see README.md); its colours follow palette.js.
*/
(() => {
  // Sizes in art pixels: the bar's height, how far the notch rises above it, the notch's width.
  const BAR = 21;
  const RISE = 6;
  const NOTCH = 16;
  const TALL = BAR + RISE;
  // where the bracket over the notch starts down toward the bar's top rail
  const KNEE = 4;

  // A return arrow: a big head at the top pointing in, toward the label, on a thick band that runs
  // back, turns down and comes round underneath. Drawn as <action-left>'s and mirrored, so here it
  // points left and turns on the right.
  const ARROW = [
    '........#...',
    '........##..',
    '....#######.',
    '..##########',
    '.##########.',
    '####....##..',
    '###.....#...',
    '###.........',
    '####........',
    '.#######....',
    '..######....',
    '....####....',
  ];

  // The button pill, the bar's full height, drawn as <action-left>'s and turned around when it is
  // used: its square edge, the column that stretches (frame colour on top, a band, a frame line,
  // dark, frame colour at the bottom), then the rounded end, which ends up on the left.
  const PILL = [
    'AAAA....',
    'AbbbAA..',
    'AbbbbbA.',
    'AbbbbbA.',
    'AAAAAAAA',
    'AkkkkkkA',
    'AkkkkkkA',
    'AkkkkkkA',
    'AkkkkkkA',
    'AkkkkkkA',
    'AkkkkkkA',
    'AkkkkkkA',
    'AkkkkkkA',
    'AkkkkkkA',
    'AkkkkkkA',
    'AkkkkkkA',
    'AkkkkkkA',
    'AkkkkkA.',
    'AkkkkkA.',
    'AkkkAA..',
    'AAAA....',
  ];
  const PILL_END = 6;

  // The notch as rows of colour keys (see pixel.js): the bracket over it, the dark inside, the
  // arrow over its shadow. Drawn left to right as <action-left>'s and turned around at the end.
  function notch() {
    const grid = Array.from({ length: TALL }, () => Array(NOTCH).fill('.'));
    const run = (y, from, to, key) => {
      for (let x = Math.max(0, from); x < Math.min(NOTCH, to); x++) grid[y][x] = key;
    };
    // the bracket's frame line: along the top, down 45 degrees, then along the bar's rail
    const line = r => (r === 0 ? [0, KNEE + 1] : r < RISE ? [KNEE + r, KNEE + r + 2] : [KNEE + RISE, NOTCH]);
    for (let y = 2; y < TALL - 1; y++) {
      const stripeFrom = y <= RISE ? KNEE + y - 1 : y === RISE + 1 ? KNEE + RISE : NOTCH;
      run(y, 0, stripeFrom, 'k');
    }
    for (let r = 0; r <= RISE; r++) run(r, ...line(r), 'A');
    // the stripe under it: where the line was a row above and is not on this row
    for (let r = 1; r <= RISE + 1; r++) {
      const [from, to] = line(r - 1);
      for (let x = from; x < to; x++) if (grid[r][x] !== 'A') grid[r][x] = 's';
    }
    run(TALL - 1, 0, NOTCH, 'A');

    // the arrow over its own shadow, one pixel down and right, for some depth, in the middle of
    // the bar between its rail and its bottom line
    const arrow = (dx, dy, key) => ARROW.forEach((row, y) => [...row].forEach((ch, x) => {
      if (ch === '#') grid[RISE + 4 + y + dy][2 + x + dx] = key;
    }));
    arrow(1, 1, 's');
    arrow(0, 0, 'H');
    return OmegaPixel.flip(grid.map(row => row.join('')));
  }

  // The label as sprite rows. Pixel Code drawn one face pixel to a pixel; each letter drawn again
  // a pixel to its right for bold (Pixel Code's own bold is fattened by part of a pixel, off the
  // grid), so 6 wide and a pixel clear of the next; every row doubled, so it stands twice as tall;
  // and a shadow one pixel down and right of all of it, in the well's colour.
  function word(value) {
    const ink = OmegaPixel.letters(value, 7);
    const at = (x, y) => ink[y >> 1]?.[x] === '#' || ink[y >> 1]?.[x - 1] === '#';
    return Array.from({ length: ink.length * 2 + 1 }, (_, y) =>
      Array.from({ length: ink[0].length }, (_, x) => (at(x, y) ? 'L' : at(x - 1, y - 1) ? 'w' : '.')).join(''));
  }

  const template = document.createElement('template');
  template.innerHTML = `
    <style>
      /* text-rendering: Pixel Code drawn without the browser's hinting, which smears its edges by
         part of a pixel even where it sits on whole pixels */
      :host { display: block; height: calc(${BAR} * var(--u)); text-rendering: geometricPrecision; }
      :host([hidden]) { display: none; }

      .action { display: flex; height: 100%; }

      .icon { flex: none; position: relative; width: calc(${NOTCH} * var(--u)); }
      .icon .art {
        position: absolute; right: 0; bottom: 0;
        width: 100%; height: calc(${TALL} * var(--u));
        background: no-repeat 0 0 / 100% 100%;
        image-rendering: pixelated;
      }

      /* the bar's top rail (frame colour over a stripe) and bottom line run through the label */
      .label {
        flex: none;
        box-sizing: border-box;
        display: flex;
        padding: 0 calc(3 * var(--u)) 0 calc(4 * var(--u));
        background: var(--oz-dark);
        border-top: var(--u) solid var(--oz-frame);
        border-bottom: var(--u) solid var(--oz-frame);
        box-shadow: inset 0 var(--u) 0 var(--oz-stripe);
      }
      /* room kept for a label still to come */
      .label.empty { min-width: calc(46 * var(--u)); }
      /* The word is a sprite (see word()), its top on the bar's top row so its capitals fill rows
         4 to 17: two dark rows under the rail above them and, below, their shadow and one dark
         row before the bottom line. 4 pixels of dark each side of the letters; the right side's
         first is the shadow. */
      .word {
        flex: none;
        margin-top: calc(-1 * var(--u));
        background: no-repeat 0 0 / 100% 100%;
        image-rendering: pixelated;
      }

      .button {
        flex: none;
        display: flex;
        border-style: solid;
        border-color: transparent;
        border-width: 0 var(--u) 0 calc(${PILL_END} * var(--u));
        border-image-slice: 0 1 0 ${PILL_END} fill;
        border-image-repeat: stretch;
        image-rendering: pixelated;
      }
      /* Pixel Code one face pixel to an art pixel, its capitals on rows 9 to 15: four dark rows
         each side, between the pill's band and its bottom. Across, the word sits in the middle of
         the pill's whole dark inside, the rounded end's included: the box reaches the 5 dark
         pixels into that end, and is an odd width with a pixel more on its left, since every
         letter's last pixel is blank, so the ink has exactly as much room on either side. */
      .hint {
        box-sizing: border-box;
        min-width: calc(43 * var(--u));
        padding: 0 calc(3 * var(--u)) 0 calc(4 * var(--u));
        text-align: center;
        margin: calc(7 * var(--u)) 0 0 calc(-5 * var(--u));
        font: 400 calc(9 * var(--u)) / calc(10 * var(--u)) var(--oz-font);
        color: var(--oz-cream);
        white-space: nowrap;
      }
    </style>
    <div class="action" part="action">
      <div class="button" part="button">
        <div class="hint" part="hint"></div>
      </div>
      <div class="label" part="label"><div class="word" part="word" role="img"></div></div>
      <div class="icon" part="icon"><div class="art"></div></div>
    </div>`;

  customElements.define('action-right', class extends OmegaPixel.Element {
    static observedAttributes = ['label', 'hint'];

    constructor() {
      super();
      this.attachShadow({ mode: 'open' }).append(template.content.cloneNode(true));
    }

    paint() {
      const { sprite, flip } = OmegaPixel;
      const part = name => this.shadowRoot.querySelector(`.${name}`);
      part('icon .art').style.backgroundImage = sprite(notch(), this);
      part('button').style.borderImageSource = sprite(flip(PILL), this);
      this.paintLabel();
    }

    // The label's sprite, sized to it; nothing, and room kept, when there is no label.
    paintLabel() {
      if (!this.isConnected) return;
      const value = this.getAttribute('label') ?? '';
      const label = this.shadowRoot.querySelector('.label');
      const art = this.shadowRoot.querySelector('.word');
      label.classList.toggle('empty', !value);
      art.setAttribute('aria-label', value);
      if (!value || !OmegaPixel.fontReady(() => this.paintLabel())) {
        art.style.cssText = '';
        return;
      }
      const rows = word(value);
      art.style.width = `calc(${rows[0].length} * var(--u))`;
      art.style.height = `calc(${rows.length} * var(--u))`;
      art.style.backgroundImage = OmegaPixel.sprite(rows, this);
    }

    attributeChangedCallback(name, _old, value) {
      if (name === 'label') this.paintLabel();
      else this.shadowRoot.querySelector('.hint').textContent = value ?? '';
    }
  });
})();
