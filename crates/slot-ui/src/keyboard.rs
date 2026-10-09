//! An on-screen keyboard for typing a passphrase with the d-pad: four rows of characters
//! and a row of keys that change what they type, on three layers between them covering
//! every printable ASCII character.

use crate::draw::{out_w, Draw, TexId};
use crate::plate::{centred_hints, UndoFace, HINT_H, LEGEND_GAP};
use crate::power_menu::MENU_INK;
use crate::slot_chrome::{edge, housing, opening, recess};
use crate::stage::stage_y;
use crate::text;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Layer {
    Lower,
    Upper,
    Symbols,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Key {
    Char(char),
    Shift,
    Symbols,
    Space,
    Delete,
    Done,
}

/// The keys that change what the others type, under the four rows of characters.
pub const BOTTOM: [Key; 5] = [Key::Shift, Key::Symbols, Key::Space, Key::Delete, Key::Done];
/// Their widths, in shares of the row once the gaps between them are taken out.
const BOTTOM_SHARE: [f32; 5] = [0.15, 0.15, 0.40, 0.15, 0.15];

const LOWER: [&str; 4] = ["1234567890", "qwertyuiop", "asdfghjkl-", "zxcvbnm_.@"];
const UPPER: [&str; 4] = ["1234567890", "QWERTYUIOP", "ASDFGHJKL-", "ZXCVBNM_.@"];
const SYMBOLS: [&str; 4] = ["!@#$%^&*()", "`~-_=+[]{}", "\\|;:'\",.<>", "/?!@#$%&*+"];

pub const KEY_COLS: usize = 10;
pub const KEY_ROWS: usize = 5;
/// The longest passphrase wpa_supplicant takes.
pub const MAX_TEXT: usize = 63;

pub const KEY_W: f32 = 56.0;
pub const KEY_H: f32 = 48.0;
const KEY_GAP: f32 = 6.0;
const ROW_GAP: f32 = 8.0;
fn keys_top() -> f32 {
    150.0 + stage_y()
}
fn prompt_y() -> f32 {
    18.0 + stage_y()
}
fn field_y() -> f32 {
    74.0 + stage_y()
}
pub const FIELD_H: f32 = 52.0;
fn legend_y() -> f32 {
    427.0 + stage_y()
}

pub fn keys_w() -> f32 {
    KEY_W * KEY_COLS as f32 + KEY_GAP * (KEY_COLS - 1) as f32
}

fn keys_x() -> f32 {
    ((out_w() as f32 - keys_w()) / 2.0).round()
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Keyboard {
    pub text: String,
    pub layer: Layer,
    pub row: usize,
    pub col: usize,
}

impl Default for Keyboard {
    fn default() -> Self {
        Keyboard {
            text: String::new(),
            layer: Layer::Lower,
            row: 1,
            col: 0,
        }
    }
}

impl Keyboard {
    pub fn key_at(&self, row: usize, col: usize) -> Key {
        key_on(self.layer, row, col)
    }

    pub fn key(&self) -> Key {
        self.key_at(self.row, self.col)
    }

    pub fn left(&mut self) {
        let n = row_len(self.row);
        self.col = (self.col + n - 1) % n;
    }

    pub fn right(&mut self) {
        self.col = (self.col + 1) % row_len(self.row);
    }

    pub fn up(&mut self) {
        self.go_to((self.row + KEY_ROWS - 1) % KEY_ROWS);
    }

    pub fn down(&mut self) {
        self.go_to((self.row + 1) % KEY_ROWS);
    }

    /// Moves to another row, onto the key nearest straight above or below.
    fn go_to(&mut self, row: usize) {
        let [x, _, w, _] = key_rect(self.row, self.col);
        let centre = x + w / 2.0;
        self.col = (0..row_len(row))
            .min_by(|&a, &b| {
                let d = |c: usize| {
                    let [x, _, w, _] = key_rect(row, c);
                    (x + w / 2.0 - centre).abs()
                };
                d(a).total_cmp(&d(b))
            })
            .unwrap_or(0);
        self.row = row;
    }

    /// Presses the key under the cursor. Returns true for the key that finishes typing.
    pub fn press(&mut self) -> bool {
        match self.key() {
            Key::Char(c) => self.type_char(c),
            Key::Shift => self.shift(),
            Key::Symbols => self.symbols(),
            Key::Space => self.type_char(' '),
            Key::Delete => self.delete(),
            Key::Done => return true,
        }
        false
    }

    pub fn type_char(&mut self, c: char) {
        if self.text.len() < MAX_TEXT {
            self.text.push(c);
        }
    }

    pub fn delete(&mut self) {
        self.text.pop();
    }

    pub fn shift(&mut self) {
        self.layer = match self.layer {
            Layer::Lower => Layer::Upper,
            Layer::Upper | Layer::Symbols => Layer::Lower,
        };
    }

    pub fn symbols(&mut self) {
        self.layer = match self.layer {
            Layer::Symbols => Layer::Lower,
            Layer::Lower | Layer::Upper => Layer::Symbols,
        };
    }
}

pub fn row_len(row: usize) -> usize {
    if row + 1 == KEY_ROWS {
        BOTTOM.len()
    } else {
        KEY_COLS
    }
}

pub fn key_on(layer: Layer, row: usize, col: usize) -> Key {
    if row + 1 == KEY_ROWS {
        return BOTTOM[col.min(BOTTOM.len() - 1)];
    }
    let rows = match layer {
        Layer::Lower => &LOWER,
        Layer::Upper => &UPPER,
        Layer::Symbols => &SYMBOLS,
    };
    Key::Char(rows[row].chars().nth(col).unwrap_or(' '))
}

/// Where a key is drawn: x, y, w, h.
pub fn key_rect(row: usize, col: usize) -> [f32; 4] {
    let y = keys_top() + (KEY_H + ROW_GAP) * row as f32;
    if row + 1 < KEY_ROWS {
        let x = keys_x() + (KEY_W + KEY_GAP) * col as f32;
        return [x, y, KEY_W, KEY_H];
    }
    let gaps = KEY_GAP * (BOTTOM.len() - 1) as f32;
    let free = keys_w() - gaps;
    let mut x = keys_x();
    for share in &BOTTOM_SHARE[..col] {
        x += (free * share).round() + KEY_GAP;
    }
    let w = if col + 1 == BOTTOM.len() {
        keys_x() + keys_w() - x
    } else {
        (free * BOTTOM_SHARE[col]).round()
    };
    [x, y, w, KEY_H]
}

/// What a key in the bottom row says, by its place in [`BOTTOM`] and the layer showing.
pub fn bottom_label(key: Key, layer: Layer) -> &'static str {
    match key {
        Key::Shift => "Shift",
        Key::Symbols if layer == Layer::Symbols => "abc",
        Key::Symbols => "#+=",
        Key::Space => "Space",
        Key::Delete => "Delete",
        Key::Done => "Join",
        Key::Char(_) => "",
    }
}

const GLYPH_PX: f32 = 26.0;
const BOTTOM_PX: f32 = 18.0;
const FIELD_PX: f32 = 26.0;
const DIM_INK: [u8; 3] = [0x9a, 0x9a, 0xa4];
pub const SHIFT_INK: [u8; 3] = [0xf0, 0xb4, 0x3c];

/// A character on a key, in its own case, centred in a key-sized face.
pub fn key_glyph_face(c: char) -> UndoFace {
    centred_face(
        &c.to_string(),
        KEY_W as u32,
        KEY_H as u32,
        GLYPH_PX,
        0.0,
        MENU_INK,
    )
}

/// A bottom-row key's word, set like slot's other labels.
pub fn bottom_key_face(label: &str, w: u32, ink: [u8; 3]) -> UndoFace {
    let Some(font) = text::label_font() else {
        return blank(w, KEY_H as u32);
    };
    let layout = text::fit(font, label, w as f32 - 8.0, 1, BOTTOM_PX, 12.0);
    let mut rgba = vec![0u8; (w * KEY_H as u32 * 4) as usize];
    text::draw_centred(&mut rgba, w, KEY_H as u32, &layout, ink);
    UndoFace {
        rgba,
        w,
        h: KEY_H as u32,
    }
}

/// What has been typed so far, its tail if it is too long for the field, with a caret.
pub fn field_face(typed: &str) -> UndoFace {
    let w = keys_w() as u32;
    let h = FIELD_H as u32;
    let Some(font) = text::label_font() else {
        return blank(w, h);
    };
    let room = w as f32 - 32.0;
    let mut shown: String = format!("{typed}|");
    while text::line_width(font, &shown, FIELD_PX, 0.0) > room && shown.chars().count() > 2 {
        shown.remove(0);
    }
    let layout = text::Layout {
        lines: vec![shown],
        px: FIELD_PX,
        tracking: 0.0,
    };
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    let lead = text::line_width(font, &layout.lines[0], FIELD_PX, 0.0);
    let cov = text::coverage(w, h, &layout);
    // Left-aligned: coverage centres the line, so slide it to the field's left edge.
    let shift = ((w as f32 - lead) / 2.0 - 16.0).round().max(0.0) as usize;
    for y in 0..h as usize {
        for x in 0..w as usize {
            let a = cov[y * w as usize + x];
            if a == 0 || x < shift {
                continue;
            }
            let at = (y * w as usize + x - shift) * 4;
            rgba[at..at + 4].copy_from_slice(&[MENU_INK[0], MENU_INK[1], MENU_INK[2], a]);
        }
    }
    UndoFace { rgba, w, h }
}

/// A line above the field saying what is being typed, in its own case.
pub fn prompt_face(prompt: &str) -> UndoFace {
    let w = keys_w() as u32;
    let Some(font) = text::label_font() else {
        return blank(w, 40);
    };
    let mut shown = prompt.to_string();
    let room = w as f32;
    while text::line_width(font, &shown, 22.0, 0.0) > room && shown.chars().count() > 2 {
        shown.pop();
        shown.pop();
        shown.push('…');
    }
    centred_face(&shown, w, 40, 22.0, 0.0, DIM_INK)
}

fn centred_face(line: &str, w: u32, h: u32, px: f32, tracking: f32, ink: [u8; 3]) -> UndoFace {
    let layout = text::Layout {
        lines: vec![line.to_string()],
        px,
        tracking,
    };
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    text::draw_centred(&mut rgba, w, h, &layout, ink);
    UndoFace { rgba, w, h }
}

fn blank(w: u32, h: u32) -> UndoFace {
    UndoFace {
        rgba: vec![0; (w * h * 4) as usize],
        w,
        h,
    }
}

pub type Face = (TexId, u32, u32);

/// Faces made once: every character any layer types, and each bottom key's word.
pub struct KeyboardFaces {
    /// Printable ASCII from the space up, indexed by `c - ' '`.
    pub glyphs: Vec<Face>,
    /// Shift (and lit while shifted), #+=, abc, Space, Delete, Join.
    pub bottom: Vec<(&'static str, Face)>,
    pub shift_on: Face,
    pub legend: Vec<(TexId, u32)>,
}

impl KeyboardFaces {
    fn glyph(&self, c: char) -> Option<Face> {
        let i = (c as usize).checked_sub(' ' as usize)?;
        self.glyphs.get(i).copied()
    }

    fn bottom(&self, label: &str) -> Option<Face> {
        self.bottom
            .iter()
            .find(|(l, _)| *l == label)
            .map(|(_, f)| *f)
    }
}

pub struct KeyboardScreen<'a> {
    pub keys: &'a Keyboard,
    pub faces: Option<&'a KeyboardFaces>,
    pub prompt: Option<Face>,
    pub field: Option<Face>,
    /// The field is shown refusing what was typed.
    pub refused: bool,
}

impl KeyboardScreen<'_> {
    pub fn draw(&self, out: &mut Vec<Draw>) {
        out.push(Draw::Rect {
            x: 0.0,
            y: 0.0,
            w: out_w() as f32,
            h: crate::draw::out_h() as f32,
            colour: opening(),
        });
        if let Some(prompt) = self.prompt {
            push(out, prompt, keys_x(), prompt_y());
        }
        out.push(Draw::Rect {
            x: keys_x(),
            y: field_y(),
            w: keys_w(),
            h: FIELD_H,
            colour: if self.refused { refused() } else { recess() },
        });
        if let Some(field) = self.field {
            push(out, field, keys_x(), field_y());
        }
        for row in 0..KEY_ROWS {
            for col in 0..row_len(row) {
                self.draw_key(row, col, out);
            }
        }
        let Some(faces) = self.faces else {
            return;
        };
        for (tex, w, x) in centred_hints(&faces.legend, LEGEND_GAP) {
            push(out, (tex, w, HINT_H), x, legend_y());
        }
    }

    fn draw_key(&self, row: usize, col: usize, out: &mut Vec<Draw>) {
        let [x, y, w, h] = key_rect(row, col);
        let lit = (row, col) == (self.keys.row, self.keys.col);
        out.push(Draw::Rect {
            x,
            y,
            w,
            h,
            colour: if lit { edge() } else { housing() },
        });
        let Some(faces) = self.faces else {
            return;
        };
        let face = match self.keys.key_at(row, col) {
            Key::Char(c) => faces.glyph(c),
            Key::Shift if self.keys.layer == Layer::Upper => Some(faces.shift_on),
            key => faces.bottom(bottom_label(key, self.keys.layer)),
        };
        if let Some((tex, fw, fh)) = face {
            let at_x = (x + (w - fw as f32) / 2.0).round();
            let at_y = (y + (h - fh as f32) / 2.0).round();
            push(out, (tex, fw, fh), at_x, at_y);
        }
    }
}

fn refused() -> [f32; 4] {
    [
        0x3a as f32 / 255.0,
        0x1a as f32 / 255.0,
        0x1a as f32 / 255.0,
        1.0,
    ]
}

fn push(out: &mut Vec<Draw>, (tex, w, h): Face, x: f32, y: f32) {
    out.push(Draw::Tex {
        x: x.round(),
        y: y.round(),
        w: w as f32,
        h: h as f32,
        tex,
        alpha: 1.0,
    });
}
