/*
  What each of flow.html's screens does with a button: where its cursor goes, what its values
  are, and which screen a row opens. player.js does the moving between them, and opens the power
  menu on any screen when POWER is held; this file only answers each press with nothing,
  { go: <screen> } to go deeper or { back: true }. A screen whose bar hint follows its cursor
  (Bluetooth's Y FORGET on a paired device) says it with hint().

  The settings that show on more than one screen (Wi-Fi, SSH, Bluetooth and its devices, the date,
  the backdrop, the bar's font) live once, in FlowScreens.model, and every screen showing one
  follows it.
*/
window.FlowScreens = (() => {
  // ---- the settings, shared ----

  // The Bluetooth devices the flow starts with, by address: a gamepad connected, headphones
  // paired, a keyboard only found, and a device found that gives no name. A device is found,
  // pairing, paired, connecting or connected; its kind is the icon its row shows.
  const DEVICES = [
    { mac: 'E4:17:D8:2C:41:07', name: '8BitDo SN30 Pro', kind: 'gamepad', state: 'connected' },
    { mac: '38:18:4C:B2:9A:10', name: 'WH-1000XM4', kind: 'audio', state: 'paired' },
    { mac: 'DC:2C:26:F1:05:3B', name: 'Keychron K3', kind: 'keyboard', state: 'found' },
    { mac: '4C:4C:6B:1A:80:D3', name: null, kind: 'other', state: 'found' },
  ];

  const model = {
    wifi: true,
    ssh: true,            // the SSH server running
    network: 'Home',      // the network joined
    joining: 'Home',      // the one the keyboard asks the passphrase of
    bluetooth: true,      // the Bluetooth radio on
    devices: DEVICES,     // what it knows of and finds, top to bottom, never changed in place
    date: { year: 2026, month: 10, day: 10, hour: 16, minute: 0, zone: 0 },
    style: 'grid',
    moving: true,
    colour: 'night',
    font: 'bold',         // the bottom bar's labels, bold or thin
  };
  const listeners = new Set();
  function set(key, value) {
    if (model[key] === value) return;
    model[key] = value;
    for (const listener of listeners) listener(key, value);
  }
  const onChange = listener => listeners.add(listener);

  // How a row with data-set shows a setting, and reads it back: undefined for a value that is no
  // setting, as SSH's INFO, which only leads to its screen.
  const CODECS = {
    wifi: { show: on => (on ? 'ON' : 'OFF'), read: value => value === 'ON' },
    ssh: { show: on => (on ? 'ON' : 'OFF'), read: value => (value === 'INFO' ? undefined : value === 'ON') },
    bluetooth: { show: on => (on ? 'ON' : 'OFF'), read: value => value === 'ON' },
    moving: { show: on => (on ? 'MOVING' : 'STILL'), read: value => value === 'MOVING' },
    style: { show: name => name.toUpperCase(), read: value => value.toLowerCase() },
    colour: { show: name => name.toUpperCase(), read: value => value.toLowerCase() },
    font: { show: name => name.toUpperCase(), read: value => value.toLowerCase() },
  };

  const MONTHS = ['JAN', 'FEB', 'MAR', 'APR', 'MAY', 'JUN', 'JUL', 'AUG', 'SEP', 'OCT', 'NOV', 'DEC'];
  const two = n => String(n).padStart(2, '0');
  const daysIn = (year, month) => new Date(year, month, 0).getDate();

  // ---- a menu: rows, top to bottom across its panels, one under the cursor ----

  class Menu {
    constructor(page) {
      this.page = page;
      this.rows = [...page.querySelectorAll('omega-row, omega-list-row')];
      this.panels = [...page.querySelectorAll('omega-panel')];
      this.cursor = 0;
      this.light();
    }

    get row() { return this.rows[this.cursor]; }

    // the cursor's row lit, and the panel holding it active
    light() {
      this.rows.forEach((row, i) => row.toggleAttribute('lit', i === this.cursor));
      for (const panel of this.panels) panel.toggleAttribute('active', panel.contains(this.row));
      this.reveal();
    }

    // In a list taller than its panel, scroll it a whole row at a time to keep the cursor in it.
    reveal() {
      const list = this.row.closest('.list');
      if (!list) return;
      const top = this.row.offsetTop;
      const bottom = top + this.row.offsetHeight;
      if (top < list.scrollTop) list.scrollTop = top;
      else if (bottom > list.scrollTop + list.clientHeight) list.scrollTop = bottom - list.clientHeight;
    }

    move(by) {
      this.settle();
      this.cursor = Math.max(0, Math.min(this.rows.length - 1, this.cursor + by));
      this.light();
    }

    // the cursor's row showing its setting again, not a value that only leads somewhere
    settle() {
      const key = this.row.dataset.set;
      if (key) this.row.setAttribute('value', CODECS[key].show(model[key]));
    }

    // ◀ ▶ step a row's value through its data-values, round and round
    change(by) {
      const values = this.row.dataset.values?.split('|');
      if (!values) return;
      const at = values.indexOf(this.row.getAttribute('value'));
      const value = values[(at + by + values.length) % values.length];
      this.row.setAttribute('value', value);
      const key = this.row.dataset.set;
      const setting = key && CODECS[key].read(value);
      if (key && setting !== undefined) set(key, setting);
    }

    // the cursor onto the row that opens `screen`, as it was when that screen was opened: on
    // the value that opens it, for a row that opens it on one alone
    focus(screen) {
      const at = this.rows.findIndex(row => row.dataset.opens === screen &&
        (row.localName !== 'omega-list-row' || row.getAttribute('label') === model.joining));
      if (at < 0) return;
      this.cursor = at;
      if (this.row.dataset.opensOn) this.row.setAttribute('value', this.row.dataset.opensOn);
      this.light();
    }

    press(key) {
      switch (key) {
        case 'up': return this.move(-1);
        case 'down': return this.move(1);
        case 'left': return this.change(-1);
        case 'right': return this.change(1);
        case 'a': return this.open();
        case 'b': return { back: true };
      }
    }

    open() {
      const screen = this.row.dataset.opens;
      if (!screen || this.row.hasAttribute('dim')) return;
      const on = this.row.dataset.opensOn;
      if (on && this.row.getAttribute('value') !== on) return;
      if (this.row.localName === 'omega-list-row') set('joining', this.row.getAttribute('label'));
      return { go: screen };
    }
  }

  // Settings: START goes back to the shelf too.
  class Settings extends Menu {
    press(key) {
      return key === 'start' ? { back: true } : super.press(key);
    }
  }

  // A screen with nothing to pick: B goes back.
  class Still {
    constructor(page) { this.page = page; }
    press(key) { if (key === 'b') return { back: true }; }
  }

  // SSH's login, opened from its row on INFO: B goes back to a row showing SSH's state again.
  class Login extends Still {
    constructor(page, menu) { super(page); this.menu = menu; }
    press(key) {
      if (key === 'b') this.menu.settle();
      return super.press(key);
    }
  }

  // The screen after the power menu's choice, shown until the handheld goes down: nothing can be
  // pressed.
  class Going {
    constructor(page) { this.page = page; }
    press() {}
  }

  // The shelf, as far as these screens go: START to Settings, SELECT to Controls.
  class Shelf extends Still {
    press(key) {
      if (key === 'start') return { go: 'settings' };
      if (key === 'select') return { go: 'controls' };
    }
  }

  // ---- Date & Time: a line of fields, ◀ ▶ between them, ▲ ▼ to change one ----

  const FIELDS = ['year', 'month', 'day', 'hour', 'minute', 'zone'];
  class Picker {
    constructor(page) {
      this.page = page;
      this.line = page.querySelector('.picker');
      this.cursor = 0;
      this.draw();
    }

    // 2026 - 10 - 10   16 : 00   UTC+00:00, each field its own box so the cursor can underline it
    draw() {
      const { year, month, day, hour, minute, zone } = model.date;
      const text = {
        year: String(year), month: two(month), day: two(day), hour: two(hour), minute: two(minute),
        zone: `UTC${zone < 0 ? '-' : '+'}${two(Math.abs(zone))}:00`,
      };
      const parts = [['year'], ' - ', ['month'], ' - ', ['day'], '   ', ['hour'], ' : ', ['minute'], '   ', ['zone']];
      this.line.replaceChildren(...parts.map(part => {
        const span = document.createElement('span');
        if (typeof part === 'string') span.textContent = part;
        else {
          span.className = 'field';
          span.textContent = text[part[0]];
          span.classList.toggle('lit', FIELDS[this.cursor] === part[0]);
        }
        return span;
      }));
      this.line.style.setProperty('--chars', this.line.textContent.length);
    }

    step(by) {
      const date = { ...model.date };
      const field = FIELDS[this.cursor];
      const wrap = (value, lo, hi) => lo + ((((value - lo) % (hi - lo + 1)) + (hi - lo + 1)) % (hi - lo + 1));
      if (field === 'year') date.year = wrap(date.year + by, 2000, 2099);
      if (field === 'month') date.month = wrap(date.month + by, 1, 12);
      if (field === 'day') date.day = wrap(date.day + by, 1, daysIn(date.year, date.month));
      if (field === 'hour') date.hour = wrap(date.hour + by, 0, 23);
      if (field === 'minute') date.minute = wrap(date.minute + by, 0, 59);
      if (field === 'zone') date.zone = wrap(date.zone + by, -12, 14);
      date.day = Math.min(date.day, daysIn(date.year, date.month));
      set('date', date);
    }

    focus() {}

    press(key) {
      switch (key) {
        case 'left': this.cursor = Math.max(0, this.cursor - 1); return this.draw();
        case 'right': this.cursor = Math.min(FIELDS.length - 1, this.cursor + 1); return this.draw();
        case 'up': return this.step(1);
        case 'down': return this.step(-1);
        case 'b': return { back: true };
      }
    }
  }

  // ---- the keyboard: a field, 10 by 4 keys and a row of wide ones ----

  // Sizes in art pixels (see flow.css): a key, the room between keys.
  const KEY_W = 23;
  const KEY_H = 19;
  const GAP = 3;
  const LETTERS = ['1234567890', 'qwertyuiop', 'asdfghjkl-', 'zxcvbnm_.@'];
  const SYMBOLS = ['!?#$%&*()=', '+/\\|[]{}<>', '\'":;,^~`£¢', '§°¡¿«»¤¥©®'];
  // the bottom row, each key as many columns wide as it says
  const WIDE = [['SHIFT', 2], ['#+=', 1], ['SPACE', 3], ['DELETE', 2], ['DONE', 2]];

  class Keyboard {
    constructor(page) {
      this.page = page;
      this.panel = page.querySelector('omega-panel');
      this.grid = page.querySelector('.keys');
      this.typed = page.querySelector('.typed');
      this.text = '';
      this.shift = false;
      this.symbols = false;
      this.at = { row: 0, col: 0 };
      this.draw();
    }

    // every key's label, row by row: 4 rows of ten single keys, then the wide ones
    layout() {
      const rows = (this.symbols ? SYMBOLS : LETTERS).map(row => [...row].map(ch => [this.shift ? ch.toUpperCase() : ch, 1]));
      return [...rows, WIDE.map(([label, span]) => [label === '#+=' && this.symbols ? 'abc' : label, span])];
    }

    // The key under the cursor: on the bottom row, the wide key over the remembered column.
    key() {
      const row = this.layout()[this.at.row];
      let col = 0;
      for (let i = 0; i < row.length; i++) {
        col += row[i][1];
        if (this.at.col < col) return { row: this.at.row, index: i, label: row[i][0] };
      }
      return { row: this.at.row, index: row.length - 1, label: row.at(-1)[0] };
    }

    // Each key placed in whole art pixels, its label centred by its ink: n letters are 6n - 1
    // pixels of ink, and a key's width is odd, so the room either side is the same.
    draw() {
      const u = n => `calc(${n} * var(--u))`;
      const lit = this.key();
      this.grid.replaceChildren(...this.layout().flatMap((row, y) => {
        let col = 0;
        return row.map(([label, span], i) => {
          const key = document.createElement('div');
          const width = span * KEY_W + (span - 1) * GAP;
          key.className = 'key';
          key.classList.toggle('lit', y === lit.row && i === lit.index);
          key.textContent = label;
          key.style.left = u(col * (KEY_W + GAP));
          key.style.top = u(y * (KEY_H + GAP));
          key.style.width = u(width);
          key.style.paddingLeft = u((width - 2 - (6 * label.length - 1)) / 2);
          col += span;
          return key;
        });
      }));
      // the field shows the end of what is typed when it is longer than the field
      this.typed.textContent = this.text.slice(-40);
    }

    move(dx, dy) {
      const rows = this.layout();
      if (dy) this.at.row = Math.max(0, Math.min(rows.length - 1, this.at.row + dy));
      if (dx) {
        // step a whole key: from the key's first column, past its width
        const row = rows[this.at.row];
        const { index } = this.key();
        const next = Math.max(0, Math.min(row.length - 1, index + dx));
        this.at.col = row.slice(0, next).reduce((sum, [, span]) => sum + span, 0);
      }
      this.draw();
    }

    type(label) {
      if (label === 'SHIFT') this.shift = !this.shift;
      else if (label === '#+=' || label === 'abc') this.symbols = !this.symbols;
      else if (label === 'SPACE') this.text += ' ';
      else if (label === 'DELETE') this.text = this.text.slice(0, -1);
      else if (label === 'DONE') return this.done();
      else this.text += label;
      this.draw();
    }

    // joined: back to the networks, which now show this one connected
    done() {
      set('network', model.joining);
      return { back: true };
    }

    focus() {}

    press(key) {
      switch (key) {
        case 'up': return this.move(0, -1);
        case 'down': return this.move(0, 1);
        case 'left': return this.move(-1, 0);
        case 'right': return this.move(1, 0);
        case 'a': return this.type(this.key().label);
        case 'start': return this.done();
        // B deletes, as the hint line says, and goes back once there is nothing left to delete
        case 'b':
          if (!this.text) return { back: true };
          this.text = this.text.slice(0, -1);
          return this.draw();
      }
    }
  }

  // ---- Bluetooth: the radio's row, then every device it knows or finds while it is on ----

  // How long the flow takes to pair with a device, and to connect to a paired one, in ms.
  const PAIRING_MS = 1500;
  const CONNECTING_MS = 1000;

  // What a device's row says it is to the handheld; nothing for one only found.
  const DEVICE_VALUE = {
    connected: 'CONNECTED', connecting: 'CONNECTING', pairing: 'PAIRING', paired: 'PAIRED', found: '',
  };
  // A device the handheld is paired with, which Y forgets.
  const isPaired = device => ['connected', 'connecting', 'paired'].includes(device.state);
  // A device's name as its row shows it: its own, or its address when it gives none.
  const nameOf = device => device.name ?? device.mac;

  // `mac`'s device in `state`, the rest as they are.
  function setDevice(mac, state) {
    set('devices', model.devices.map(device => (device.mac === mac ? { ...device, state } : device)));
  }

  // A note's text, any … in it drawn as flow/art.js's glyph (the web font has none); hidden when
  // there is nothing to say.
  function note(p, text) {
    p.replaceChildren(...text.split('…').flatMap((part, i) => {
      if (!i) return [part];
      const glyph = document.createElement('i');
      glyph.className = 'g ellipsis';
      return [glyph, part];
    }));
    p.hidden = !text;
  }

  class Bluetooth extends Menu {
    constructor(page) {
      super(page);
      this.radio = this.rows[0];
      this.list = page.querySelector('.devices');
      this.note = this.list.querySelector('.note');
      this.draw();
    }

    // The device under the cursor, when it is on one.
    device() {
      return model.devices.find(device => device.mac === this.row.dataset.device);
    }

    // The devices' rows drawn again from the model, the cursor staying on the row it was on (on
    // the one now in its place when that one is gone), and the note under them: the radio is
    // off, or nothing is found yet.
    draw() {
      const lit = this.row;
      const kept = new Map([...this.list.querySelectorAll('omega-list-row')].map(row => [row.dataset.device, row]));
      const rows = model.devices.map(device => {
        const row = kept.get(device.mac) ?? document.createElement('omega-list-row');
        kept.delete(device.mac);
        row.dataset.device = device.mac;
        row.setAttribute('label', nameOf(device));
        row.setAttribute('device', device.kind);
        if (DEVICE_VALUE[device.state]) row.setAttribute('value', DEVICE_VALUE[device.state]);
        else row.removeAttribute('value');
        row.hidden = !model.bluetooth;
        return row;
      });
      for (const row of kept.values()) row.remove();
      if (rows.some((row, i) => this.list.children[i] !== row)) this.note.before(...rows);
      note(this.note, !model.bluetooth ? 'Bluetooth is off.' : rows.length ? '' : 'Looking for devices…');
      this.rows = [this.radio, ...(model.bluetooth ? rows : [])];
      const at = this.rows.indexOf(lit);
      this.cursor = at >= 0 ? at : Math.min(this.cursor, this.rows.length - 1);
      this.light();
    }

    // The cursor on `mac`'s device, or on the radio's row.
    home(mac) {
      this.cursor = Math.max(0, this.rows.findIndex(row => row.dataset.device === mac));
      this.light();
    }

    // Y FORGET in the bar while the cursor is on a paired device.
    hint() {
      const device = this.device();
      return device && isPaired(device) ? 'Y FORGET' : null;
    }

    press(key) {
      const device = this.device();
      if (device && key === 'a') return this.pick(device);
      if (device && key === 'y' && isPaired(device)) return this.forget(device);
      return super.press(key);
    }

    // A on a device: one only found pairs and then connects, a paired one connects, a connected
    // one disconnects. One already on its way is left to get there.
    pick(device) {
      if (device.state === 'found') this.towards(device, 'pairing', PAIRING_MS);
      else if (device.state === 'paired') this.towards(device, 'connecting', CONNECTING_MS);
      else if (device.state === 'connected') setDevice(device.mac, 'paired');
    }

    // `device` `state` for `ms`, then connected, unless something else has happened to it since.
    towards(device, state, ms) {
      setDevice(device.mac, state);
      setTimeout(() => {
        if (model.devices.find(d => d.mac === device.mac)?.state === state) setDevice(device.mac, 'connected');
      }, ms);
    }

    // Y on a paired device: forgotten, it leaves the list, and the cursor takes the row after it.
    forget(device) {
      set('devices', model.devices.filter(d => d.mac !== device.mac));
    }
  }

  // Bluetooth as the URL's bt= has it: on with the devices the flow starts with, off, on with the
  // keyboard pairing and the cursor on it, or on with nothing found yet.
  const BT = ['on', 'off', 'pairing', 'empty'];
  let bluetoothMenu = null;
  function bluetoothAs(state) {
    const pairing = DEVICES.find(device => device.kind === 'keyboard');
    set('bluetooth', state !== 'off');
    set('devices', state === 'empty' ? [] : DEVICES.map(device =>
      (state === 'pairing' && device === pairing ? { ...device, state: 'pairing' } : device)));
    bluetoothMenu?.home(state === 'pairing' ? pairing.mac : null);
  }
  // Which of those Bluetooth is in now, as near as it comes.
  function bluetoothState() {
    if (!model.bluetooth) return 'off';
    if (!model.devices.length) return 'empty';
    return model.devices.some(device => device.state === 'pairing') ? 'pairing' : 'on';
  }

  // ---- every screen, and keeping them in step with the settings ----

  function build(screen) {
    const page = name => screen.querySelector(`[data-screen="${name}"]`);
    const network = new Menu(page('network'));
    const screens = {
      shelf: new Shelf(page('shelf')),
      settings: new Settings(page('settings')),
      controls: new Still(page('controls')),
      network,
      ssh: new Login(page('ssh'), network),
      keyboard: new Keyboard(page('keyboard')),
      bluetooth: new Bluetooth(page('bluetooth')),
      labels: new Menu(page('labels')),
      datetime: new Picker(page('datetime')),
      background: new Menu(page('background')),
      about: new Still(page('about')),
      power: new Menu(page('power')),
      restarting: new Going(page('restarting')),
      poweroff: new Going(page('poweroff')),
    };

    // what shows a setting, drawn again from the model
    function show() {
      const { date, wifi, ssh } = model;
      for (const row of screen.querySelectorAll('[data-set]')) {
        row.setAttribute('value', CODECS[row.dataset.set].show(model[row.dataset.set]));
      }
      screen.querySelector('[data-show="date"]').setAttribute('value',
        `${MONTHS[date.month - 1]} ${date.day} ${two(date.hour)}:${two(date.minute)}`);
      screen.querySelector('[data-show="network"]').setAttribute('value', wifi ? model.network.toUpperCase() : 'OFF');
      for (const row of page('network').querySelectorAll('omega-list-row')) {
        row.toggleAttribute('dim', !wifi);
        if (wifi && row.getAttribute('label') === model.network) row.setAttribute('value', 'CONNECTED');
        else row.removeAttribute('value');
      }
      page('ssh').querySelector('.ssh').classList.toggle('offline', !ssh || !wifi);
      page('ssh').querySelector('.when-offline .note').textContent =
        ssh ? 'Join a Wi-Fi network first.' : 'Turn SSH on first.';
      page('keyboard').querySelector('omega-panel').setAttribute('label', `PASSPHRASE FOR ${model.joining.toUpperCase()}`);
      // Settings' BLUETOOTH: the device connected, else whether the radio is on
      const connected = model.devices.find(device => device.state === 'connected');
      screen.querySelector('[data-show="bluetooth"]').setAttribute('value',
        !model.bluetooth ? 'OFF' : connected ? nameOf(connected).toUpperCase() : 'ON');
      screens.bluetooth.draw();
      screens.datetime.draw();
    }
    show();
    onChange(show);
    bluetoothMenu = screens.bluetooth;
    return screens;
  }

  // Each screen's parent, the one B goes back to. The power menu's is the screen POWER was held
  // on, which player.js puts here as it opens it: the shelf until then.
  const PARENT = {
    settings: 'shelf', controls: 'shelf',
    network: 'settings', bluetooth: 'settings', labels: 'settings', datetime: 'settings',
    background: 'settings', about: 'settings',
    ssh: 'network', keyboard: 'network',
    power: 'shelf', restarting: 'power', poweroff: 'power',
  };

  // The screens after the power menu's choice, which have no bar: nothing can be pressed on them.
  const GOING = ['restarting', 'poweroff'];

  return { model, set, onChange, build, PARENT, GOING, BT, bluetoothAs, bluetoothState };
})();
