/*
  <omega-bar>: the bar along the bottom of a screen. An <action-left> and an <action-right> at its
  ends and, between them, the game's plate, which sits a little under the actions' top rails and
  runs on behind both buttons' rounded ends, so the bar's bottom line never breaks. The keys a
  screen has beyond the two actions are shown on the plate as hints.

    <omega-bar hint="Y FORGET">
      <action-left slot="left" label="BACK" hint="B"></action-left>
      <action-right slot="right" label="JOIN" hint="A"></action-right>
    </omega-bar>

  hint      a key and what it does, the first word the key: "Y FORGET", "START FETCH". Two are
            given with a | between them: "Y FORGET | X RENAME". Each is drawn as its key in a
            small dark cap, then the word, in Pixel Code one face pixel to an art pixel, and the
            hints together are centred on the plate, 8 pixels apart.
  floating  only the two actions, at the bottom corners, with no plate between them (and so no
            hints), for a screen that wants what is behind the bar to show between them.

  Slots: left and right, for the two actions.

  It fills its parent's width and is 21 art pixels tall, the actions' height; their notches rise 6
  above it, over whatever is there. The hints' capitals are on rows 9 to 15, the same rows as the
  words in the actions' buttons, and each key's cap is 11 rows tall around them: two dark rows above
  and below its letters, two dark pixels either side. Areas, each a div that ::part() reaches: bar,
  plate, hints, hint, key, word.

  Needs everything in foundation/ on the page (see README.md) and the two actions' components; its
  colours follow palette.js.
*/
(() => {
  // Sizes in art pixels: the bar's height, how far down the plate starts, how far it runs on
  // behind each button's rounded end, and how tall a key's cap is.
  const BAR = 21;
  const PLATE_TOP = 3;
  const UNDER = 7;
  const CAP = 11;
  // How far into the plate, under its top line, the hints start: the top line is the bar's row 3,
  // so they start on row 7 and their capitals, two rows down their line, are on rows 9 to 15, the
  // rows of the words in the actions' buttons.
  const HINTS_TOP = 3;
  // The gap between a key's cap and its word, and between two hints, in pixels of ink to ink.
  const GAP = 3;
  const BETWEEN = 8;

  // A key's cap: its left end, the column that stretches, its right end; dark, with its four
  // corner pixels left out.
  const KEY = ['.k.', ...Array(CAP - 2).fill('kkk'), '.k.'];

  // Pixel Code letters are 6 pixels apart, the last of the 6 blank.
  const ADVANCE = 6;

  const template = document.createElement('template');
  template.innerHTML = `
    <style>
      /* text-rendering: Pixel Code drawn without the browser's hinting, which smears its edges by
         part of a pixel even where it sits on whole pixels */
      :host { display: block; flex: none; height: calc(${BAR} * var(--u)); text-rendering: geometricPrecision; }
      :host([hidden]) { display: none; }

      .bar { display: flex; height: 100%; }
      ::slotted(*) { position: relative; z-index: 1; flex: none; }

      /* the plate: its top line, a lit row, the plate colour, a shadow row, and the bar's bottom line
         in the frame colour under all of it, level with the actions' own */
      .plate {
        flex: 1;
        min-width: 0;
        margin: calc(${PLATE_TOP} * var(--u)) calc(-${UNDER} * var(--u)) 0;
        background: var(--oz-plate);
        border-top: var(--u) solid var(--oz-edge);
        border-bottom: var(--u) solid var(--oz-frame);
        box-shadow: inset 0 var(--u) 0 var(--oz-band), inset 0 calc(-1 * var(--u)) 0 var(--oz-edge);
      }
      :host([floating]) .plate { visibility: hidden; }

      /* Centred on the plate, rounded down to a whole art pixel: --ink is the hints' width in art
         pixels from their first ink to their last, which paintHints() works out from the text. */
      .hints {
        display: flex;
        width: max-content;
        margin-top: calc(${HINTS_TOP} * var(--u));
        margin-left: max(0px, round(down, (100% - var(--ink, 0) * var(--u)) / 2, var(--u)));
        font: 400 calc(9 * var(--u)) / calc(10 * var(--u)) var(--oz-font);
        white-space: nowrap;
        color: var(--oz-cream);
      }
      .hint { display: flex; }
      .hint + .hint { margin-left: calc(${BETWEEN - 1} * var(--u)); }
      /* the key's letters start two dark pixels in, its end column and one more; on the right the
         letters' own blank pixel and the end column make the same two */
      .key {
        box-sizing: border-box;
        height: calc(${CAP} * var(--u));
        padding-left: var(--u);
        border-style: solid;
        border-color: transparent;
        border-width: 0 var(--u);
        border-image-source: var(--cap);
        border-image-slice: 0 1 fill;
        border-image-repeat: stretch;
        image-rendering: pixelated;
      }
      .word { margin-left: calc(${GAP} * var(--u)); }
      .word:empty { display: none; }
    </style>
    <div class="bar" part="bar">
      <slot name="left"></slot>
      <div class="plate" part="plate"><div class="hints" part="hints"></div></div>
      <slot name="right"></slot>
    </div>`;

  // The hints in `value`, each { key, word }.
  function parse(value) {
    return (value ?? '').split('|').map(hint => hint.trim()).filter(Boolean).map(hint => {
      const [key, ...word] = hint.split(/\s+/);
      return { key, word: word.join(' ') };
    });
  }

  customElements.define('omega-bar', class extends OmegaPixel.Element {
    static observedAttributes = ['hint'];

    constructor() {
      super();
      this.attachShadow({ mode: 'open' }).append(template.content.cloneNode(true));
    }

    paint() {
      this.shadowRoot.querySelector('.bar').style.setProperty('--cap', OmegaPixel.sprite(KEY, this));
    }

    // One key and word per hint, and the width they take, ink to ink, to centre them by.
    paintHints(value) {
      const hints = parse(value);
      const box = this.shadowRoot.querySelector('.hints');
      const length = text => [...text].length;
      box.replaceChildren(...hints.map(({ key, word }) => {
        const hint = document.createElement('div');
        hint.className = 'hint';
        hint.setAttribute('part', 'hint');
        for (const [name, text] of [['key', key], ['word', word]]) {
          const area = document.createElement('div');
          area.className = name;
          area.setAttribute('part', name);
          area.textContent = text;
          hint.append(area);
        }
        return hint;
      }));
      // a cap is its letters and three pixels more, a word its letters; the last word's blank
      // pixel is not ink
      const ink = hints.reduce((sum, { key, word }, i) =>
        sum + (i ? BETWEEN - 1 : 0) + length(key) * ADVANCE + 3 + (word ? GAP + length(word) * ADVANCE : 0), 0);
      const last = hints.at(-1);
      box.style.setProperty('--ink', hints.length ? ink - (last.word ? 1 : 0) : 0);
    }

    attributeChangedCallback(name, _old, value) {
      if (name === 'hint') this.paintHints(value);
    }
  });
})();
