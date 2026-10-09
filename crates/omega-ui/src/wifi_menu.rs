//! The Wi-Fi screen: a list in the quick menu's style, whose first rows switch Wi-Fi and
//! network time and whose others are the networks in reach, with two lines under it saying
//! how the handheld is connected.

use crate::draw::{out_h, out_w, Draw, TexId};
use crate::icon::symbols_font;
use crate::plate::{centred_hints, UndoFace, HINT_H, LEGEND_GAP};
use crate::power_menu::{MENU_H, MENU_INK, MENU_PAD};
use crate::quick_menu::{QUICK_EDGE, QUICK_PITCH};
use crate::slot_chrome::{edge, opening};
use crate::stage::stage_y;
use crate::text;

pub type Face = (TexId, u32, u32);

const LIST_TOP: f32 = 24.0;
const BAR_INSET: f32 = 4.0;
const TYPE_DROP: f32 = 4.0;
const CARET_GAP: f32 = 14.0;
const ICON_GAP: f32 = 12.0;
const STATUS_H: u32 = 26;
const STATUS_PX: f32 = 18.0;
const NAME_PX: f32 = 27.0;
const ICON_PX: f32 = 28.0;
const DIM_INK: [u8; 3] = [0x9a, 0x9a, 0xa4];

fn legend_y() -> f32 {
    427.0 + stage_y()
}

fn status_y() -> f32 {
    legend_y() - 2.0 * STATUS_H as f32 - 24.0
}

/// How many rows the list shows at once on this canvas.
pub fn wifi_rows_shown() -> usize {
    (((status_y() - 8.0 - LIST_TOP) / QUICK_PITCH).floor() as usize).max(3)
}

/// The first row to show so the lit one is in view, moving the window as little as it can
/// from where it was.
pub fn wifi_window(lit: usize, rows: usize, was: usize) -> usize {
    let shown = wifi_rows_shown();
    if rows <= shown {
        return 0;
    }
    let first = if lit < was {
        lit
    } else if lit >= was + shown {
        lit + 1 - shown
    } else {
        was
    };
    first.min(rows - shown)
}

fn row_top(i: usize) -> f32 {
    LIST_TOP + QUICK_PITCH * i as f32
}

/// A line of text in its own case, for names that are not omega-zero's to set in capitals: a
/// network's, an address. Too long, it ends in an ellipsis.
pub fn plain_face(line: &str, max_w: f32, px: f32, h: u32, lit: bool) -> UndoFace {
    let Some(font) = text::label_font() else {
        return UndoFace {
            rgba: Vec::new(),
            w: 0,
            h: 0,
        };
    };
    let mut shown = line.to_string();
    let room = max_w - 2.0 * MENU_PAD as f32;
    if text::line_width(font, &shown, px, 0.0) > room {
        while !shown.is_empty() && text::line_width(font, &format!("{shown}…"), px, 0.0) > room {
            shown.pop();
        }
        shown = format!("{}…", shown.trim_end());
    }
    let ink = text::line_width(font, &shown, px, 0.0).ceil() as u32;
    let w = ink + 2 * MENU_PAD;
    let layout = text::Layout {
        lines: vec![shown],
        px,
        tracking: 0.0,
    };
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    text::draw_centred(
        &mut rgba,
        w,
        h,
        &layout,
        if lit { MENU_INK } else { DIM_INK },
    );
    UndoFace { rgba, w, h }
}

/// A network's name in the list.
pub fn network_name_face(name: &str, lit: bool) -> UndoFace {
    let room = out_w() as f32 * 0.5;
    plain_face(name, room, NAME_PX, MENU_H, lit)
}

fn status_room() -> f32 {
    out_w() as f32 - 2.0 * QUICK_EDGE
}

/// A line under the list.
pub fn wifi_status_face(line: &str) -> UndoFace {
    plain_face(line, status_room(), STATUS_PX, STATUS_H, false)
}

/// Whether `line` fits under the list whole, without being cut short with an ellipsis.
pub fn status_fits(line: &str) -> bool {
    text::label_font().is_some_and(|font| {
        text::line_width(font, line, STATUS_PX, 0.0) <= status_room() - 2.0 * MENU_PAD as f32
    })
}

/// Signal strength as one to four bars, with a padlock on networks that need a passphrase.
pub fn signal_glyph(bars: u8, locked: bool) -> char {
    // Material Design's wifi-strength-1 to 4, each followed by its alert and lock variants.
    let plain = 0xf091f + 3 * (u32::from(bars.clamp(1, 4)) - 1);
    char::from_u32(if locked { plain + 2 } else { plain }).unwrap_or(' ')
}

pub fn signal_face(bars: u8, locked: bool, lit: bool) -> UndoFace {
    let Some(symbols) = symbols_font() else {
        return UndoFace {
            rgba: Vec::new(),
            w: 0,
            h: 0,
        };
    };
    let (m, cov) = symbols.rasterize(signal_glyph(bars, locked), ICON_PX);
    let (w, h) = (m.width as u32, MENU_H);
    let top = ((h as i32 - m.height as i32) / 2).max(0);
    let ink = if lit { MENU_INK } else { DIM_INK };
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for gy in 0..m.height {
        let y = top + gy as i32;
        if y >= h as i32 {
            break;
        }
        for gx in 0..m.width {
            let at = ((y as u32 * w + gx as u32) * 4) as usize;
            rgba[at..at + 4].copy_from_slice(&[ink[0], ink[1], ink[2], cov[gy * m.width + gx]]);
        }
    }
    UndoFace { rgba, w, h }
}

pub struct WifiRowFaces {
    pub label: Face,
    pub value: Option<Face>,
    pub icon: Option<Face>,
    /// The value is a setting changed with left and right, and shows carets while lit.
    pub setting: bool,
}

pub struct WifiMenuFaces {
    /// The rows in view, top first.
    pub rows: Vec<WifiRowFaces>,
    /// Which of them is lit.
    pub lit: Option<usize>,
    pub status: Vec<Face>,
    pub legend: Vec<(TexId, u32)>,
    pub carets: [Face; 2],
}

impl WifiMenuFaces {
    pub fn draw(&self, out: &mut Vec<Draw>) {
        out.push(Draw::Rect {
            x: 0.0,
            y: 0.0,
            w: out_w() as f32,
            h: out_h() as f32,
            colour: opening(),
        });
        self.draw_over(out);
    }

    /// The list over whatever is already drawn, such as the background it is choosing.
    pub fn draw_over(&self, out: &mut Vec<Draw>) {
        if let Some(lit) = self.lit {
            out.push(Draw::Rect {
                x: 0.0,
                y: row_top(lit) + BAR_INSET,
                w: out_w() as f32,
                h: QUICK_PITCH - 2.0 * BAR_INSET,
                colour: edge(),
            });
        }
        let (right, pad) = (out_w() as f32 - QUICK_EDGE, MENU_PAD as f32);
        for (i, row) in self.rows.iter().enumerate() {
            let y = row_top(i) + TYPE_DROP;
            push(out, row.label, QUICK_EDGE - pad, y);
            let mut edge_x = right;
            if let Some(icon) = row.icon {
                edge_x -= icon.1 as f32;
                push(out, icon, edge_x, y);
                edge_x -= ICON_GAP;
            }
            let Some(value) = row.value else {
                continue;
            };
            if !(row.setting && self.lit == Some(i)) {
                push(out, value, edge_x + pad - value.1 as f32, y);
                continue;
            }
            let [left, right_caret] = self.carets;
            let rx = edge_x - right_caret.1 as f32;
            push(out, right_caret, rx, y);
            let vx = rx - CARET_GAP + pad - value.1 as f32;
            push(out, value, vx, y);
            push(out, left, vx + pad - CARET_GAP - left.1 as f32, y);
        }
        let mut y = status_y();
        for line in &self.status {
            push(out, *line, QUICK_EDGE - MENU_PAD as f32, y);
            y += STATUS_H as f32;
        }
        for (tex, w, x) in centred_hints(&self.legend, LEGEND_GAP) {
            push(out, (tex, w, HINT_H), x, legend_y());
        }
    }
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
