/*
  <omega-list-row>: one Wi-Fi network in a list, an <omega-row> with the network's name on the
  left and, on the right, what it is to the handheld, a padlock if it needs a password and how
  strong it comes in.

    <omega-panel label="NETWORKS" active>
      <omega-list-row label="Home" value="CONNECTED" signal="3" lock lit></omega-list-row>
      <omega-list-row label="Office" value="SAVED" signal="1" lock></omega-list-row>
      <omega-list-row label="Café" signal="2"></omega-list-row>
    </omega-panel>

  label   the network's name, from the left, in its own case (names are the player's).
  value   what it is to the handheld, from the right: CONNECTED, JOINING, SAVED. Optional.
  signal  0 to 3: the Wi-Fi bars, that many of the three lit and the rest drawn faint. Without
          it there are no bars.
  lock    a padlock beside the bars, for a network with a password.
  lit     the row under the cursor, as <omega-row>'s.
  dim     a network that cannot be joined, as <omega-row>'s.

  The icons stand on the capitals' rows, the padlock 5 art pixels wide like a letter and the bars 7,
  3 pixels apart and a letter's width (6) clear of the value, so they read as a pair of their own;
  their room is kept when a row has none, so every row's value, padlock and bars line up down the
  list. The icons are drawn in the name's colour. Everything else is <omega-row>'s: 13 art pixels
  tall, the same band, words and colours. Areas, each a div that ::part() reaches: row, label,
  value, lock, signal.

  Needs everything in foundation/ on the page (see README.md) and components/omega-row.js before it.
*/
(() => {
  const OmegaRow = customElements.get('omega-row');

  // The top bar's Wi-Fi icon (WIFI in crates/omega-ui/src/status_bar.rs), its three parts numbered
  // in the order they light: 1 the dot, 2 the small arc, 3 the big one.
  const SIGNAL = [
    '..333..',
    '.3...3.',
    '3.....3',
    '..222..',
    '.2...2.',
    '.......',
    '...1...',
  ];

  // A padlock as tall as the capitals and as wide as a letter: the shackle, then the body with
  // its keyhole.
  const LOCK = [
    '.###.',
    '#...#',
    '#...#',
    '#####',
    '##.##',
    '##.##',
    '#####',
  ];

  const template = document.createElement('template');
  template.innerHTML = `
    <style>
      ${OmegaRow.STYLE}
      .lock { width: calc(5 * var(--u)); margin-left: calc(6 * var(--u)); }
      .signal { width: calc(7 * var(--u)); margin-left: calc(3 * var(--u)); }
      :host(:not([lock])) .lock, :host(:not([signal])) .signal { visibility: hidden; }
    </style>
    <div class="row" part="row">
      <div class="label" part="label"></div>
      <div class="value" part="value"></div>
      <div class="art lock" part="lock" role="img" aria-label="needs a password"></div>
      <div class="art signal" part="signal" role="img"></div>
    </div>`;

  customElements.define('omega-list-row', class extends OmegaRow {
    static observedAttributes = [...OmegaRow.observedAttributes, 'signal', 'lock'];
    static template = template;

    // The bars lit, 0 to 3; none without a signal.
    level() {
      const level = Number.parseInt(this.getAttribute('signal'), 10);
      return Number.isNaN(level) ? null : Math.min(3, Math.max(0, level));
    }

    // The padlock and the bars, in the name's colour, the bars not reached faint.
    paintArt(ink) {
      const { sprite } = OmegaPixel;
      const level = this.level() ?? 0;
      const bars = SIGNAL.map(row => row.replace(/\d/g, part => (part <= level ? ink.name : ink.off)));
      const signal = this.area('signal');
      signal.style.backgroundImage = sprite(bars, this);
      signal.setAttribute('aria-label', `signal ${level} of 3`);
      this.area('lock').style.backgroundImage = sprite(LOCK.map(row => row.replaceAll('#', ink.name)), this);
    }
  });
})();
