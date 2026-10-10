use crate::draw::{out_h, out_w, Draw, TexId};
use crate::plate::{arrows_hint_face, centred_hints, hint_face, UndoFace, HINT_H, LEGEND_GAP};
use crate::power_menu::{MENU_H, MENU_INK, MENU_PAD, MENU_PX};
use crate::slot_chrome::edge;
use crate::stage::stage_y;
use crate::text;

/// The two groups the settings are listed in.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SettingsGroup {
    /// How the games play: the settings left and right change in place, and the cart labels.
    Configs,
    /// The handheld itself: its clock, its network, its background and what it is.
    General,
}

impl SettingsGroup {
    pub fn label(self) -> &'static str {
        match self {
            SettingsGroup::Configs => "Configs",
            SettingsGroup::General => "General",
        }
    }
}

/// A row of the settings START opens on the shelf, in the order they are listed.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SettingsRow {
    FastForward,
    FastForwardSound,
    ColourCorrection,
    Rumble,
    Labels,
    DateTime,
    Network,
    Background,
    About,
}

impl SettingsRow {
    pub const ALL: [SettingsRow; 9] = [
        SettingsRow::FastForward,
        SettingsRow::FastForwardSound,
        SettingsRow::ColourCorrection,
        SettingsRow::Rumble,
        SettingsRow::Labels,
        SettingsRow::DateTime,
        SettingsRow::Network,
        SettingsRow::Background,
        SettingsRow::About,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn label(self) -> &'static str {
        match self {
            SettingsRow::FastForward => "Fast Forward",
            SettingsRow::FastForwardSound => "Fast Forward Sound",
            SettingsRow::ColourCorrection => "Colour Correction",
            SettingsRow::Rumble => "Rumble",
            SettingsRow::Labels => "Labels",
            SettingsRow::DateTime => "Date & Time",
            SettingsRow::Network => "Network",
            SettingsRow::Background => "Background",
            SettingsRow::About => "About",
        }
    }

    pub fn group(self) -> SettingsGroup {
        match self {
            SettingsRow::FastForward
            | SettingsRow::FastForwardSound
            | SettingsRow::ColourCorrection
            | SettingsRow::Rumble
            | SettingsRow::Labels => SettingsGroup::Configs,
            SettingsRow::DateTime
            | SettingsRow::Network
            | SettingsRow::Background
            | SettingsRow::About => SettingsGroup::General,
        }
    }

    /// A opens a screen of its own for this row, rather than left and right changing it.
    pub fn opens(self) -> bool {
        matches!(
            self,
            SettingsRow::Labels
                | SettingsRow::DateTime
                | SettingsRow::Network
                | SettingsRow::Background
                | SettingsRow::About
        )
    }

    /// The row above this one among `rows`, the rows a handheld shows, wrapping at the top.
    pub fn up_in(self, rows: &[SettingsRow]) -> SettingsRow {
        let n = rows.len();
        let at = rows.iter().position(|r| *r == self).unwrap_or(0);
        rows[(at + n - 1) % n]
    }

    pub fn down_in(self, rows: &[SettingsRow]) -> SettingsRow {
        let at = rows.iter().position(|r| *r == self).unwrap_or(0);
        rows[(at + 1) % rows.len()]
    }

    pub fn up(self) -> SettingsRow {
        self.up_in(&SettingsRow::ALL)
    }

    pub fn down(self) -> SettingsRow {
        self.down_in(&SettingsRow::ALL)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum QuickValue {
    Speed2,
    Speed3,
    Speed4,
    Speed6,
    On,
    Off,
}

impl QuickValue {
    pub const ALL: [QuickValue; 6] = [
        QuickValue::Speed2,
        QuickValue::Speed3,
        QuickValue::Speed4,
        QuickValue::Speed6,
        QuickValue::On,
        QuickValue::Off,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn text(self) -> &'static str {
        match self {
            QuickValue::Speed2 => "2×",
            QuickValue::Speed3 => "3×",
            QuickValue::Speed4 => "4×",
            QuickValue::Speed6 => "6×",
            QuickValue::On => "On",
            QuickValue::Off => "Off",
        }
    }

    pub fn speed(frames: u8) -> Option<QuickValue> {
        match frames {
            2 => Some(QuickValue::Speed2),
            3 => Some(QuickValue::Speed3),
            4 => Some(QuickValue::Speed4),
            6 => Some(QuickValue::Speed6),
            _ => None,
        }
    }

    pub fn flag(on: bool) -> QuickValue {
        if on {
            QuickValue::On
        } else {
            QuickValue::Off
        }
    }
}

pub const QUICK_PITCH: f32 = 52.0;
/// Where the first row in view sits with every row this build has.
pub fn quick_top() -> f32 {
    quick_top_of(quick_rows_shown(SettingsRow::ALL.len()))
}

/// How many of `n` rows are in view at once: all of them while they fit above the legend,
/// centred as they always were, and as many as do on a canvas too short for them, the rest
/// scrolled to.
pub fn quick_rows_shown(n: usize) -> usize {
    let fit = ((2.0 * legend_y() - out_h() as f32) / QUICK_PITCH).floor();
    n.min(fit.max(3.0) as usize)
}

/// The first of `n` rows to show so the lit one is in view, moving the window as little as it
/// can from where it was.
pub fn quick_window(lit: usize, n: usize, was: usize) -> usize {
    let shown = quick_rows_shown(n);
    if n <= shown {
        return 0;
    }
    let first = if lit < was {
        lit
    } else if lit >= was + shown {
        lit + 1 - shown
    } else {
        was
    };
    first.min(n - shown)
}

/// Where the first of `n` rows sits, the rows centred on the canvas.
pub fn quick_top_of(n: usize) -> f32 {
    (out_h() as f32 - QUICK_PITCH * n as f32) / 2.0
}
pub const QUICK_EDGE: f32 = 32.0;
const BAR_INSET: f32 = 4.0;
const TYPE_DROP: f32 = 4.0;
const CARET_GAP: f32 = 14.0;
const CARET_PX: f32 = 24.0;
fn legend_y() -> f32 {
    427.0 + stage_y()
}
const DIM_INK: [u8; 3] = [0x9a, 0x9a, 0xa4];

pub fn quick_label_face(row: SettingsRow) -> UndoFace {
    quick_text_face(row.label(), MENU_INK)
}

pub fn quick_value_face(text: &str, lit: bool) -> UndoFace {
    quick_text_face(text, if lit { MENU_INK } else { DIM_INK })
}

fn quick_text_face(label: &str, colour: [u8; 3]) -> UndoFace {
    let Some(font) = text::label_font() else {
        return UndoFace {
            rgba: Vec::new(),
            w: 0,
            h: 0,
        };
    };
    let layout = text::fit(font, label, out_w() as f32, 1, MENU_PX, MENU_PX);
    let set = layout
        .lines
        .iter()
        .map(|l| text::line_width(font, l, layout.px, layout.tracking))
        .fold(0.0, f32::max);
    let w = set.ceil() as u32 + 2 * MENU_PAD;
    let mut rgba = vec![0u8; (w * MENU_H * 4) as usize];
    text::draw_centred(&mut rgba, w, MENU_H, &layout, colour);
    UndoFace { rgba, w, h: MENU_H }
}

pub fn quick_caret_face(right: bool) -> UndoFace {
    let glyph = if right { '\u{f0da}' } else { '\u{f0d9}' };
    let (Some(symbols), Some(label)) = (crate::icon::symbols_font(), text::label_font()) else {
        return UndoFace {
            rgba: Vec::new(),
            w: 0,
            h: 0,
        };
    };
    let (m, cov) = symbols.rasterize(glyph, CARET_PX);
    let (w, h) = (m.width as u32, MENU_H);
    let centre = match label.horizontal_line_metrics(MENU_PX) {
        Some(v) => {
            (h as f32 - v.new_line_size) / 2.0 + v.ascent
                - label.metrics('H', MENU_PX).height as f32 / 2.0
        }
        None => h as f32 / 2.0,
    };
    let top = (centre - m.height as f32 / 2.0).round() as i32;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for gy in 0..m.height {
        let y = top + gy as i32;
        if y < 0 || y >= h as i32 {
            continue;
        }
        for gx in 0..m.width {
            let at = ((y as u32 * w + gx as u32) * 4) as usize;
            let a = cov[gy * m.width + gx];
            rgba[at..at + 4].copy_from_slice(&[MENU_INK[0], MENU_INK[1], MENU_INK[2], a]);
        }
    }
    UndoFace { rgba, w, h }
}

pub fn quick_legend_faces() -> [UndoFace; 3] {
    [
        hint_face("B", "Back"),
        arrows_hint_face("Change"),
        hint_face("A", "Open"),
    ]
}

pub struct QuickMenuFaces {
    pub labels: Vec<(TexId, u32, u32)>,
    pub values: Vec<[(TexId, u32, u32); 2]>,
    pub carets: [(TexId, u32, u32); 2],
    pub legend: [(TexId, u32); 3],
}

pub struct QuickMenu<'a> {
    pub row: SettingsRow,
    /// The rows this handheld shows: all of them, but for Network on one without a radio.
    pub rows: &'a [SettingsRow],
    pub values: [Option<QuickValue>; SettingsRow::ALL.len()],
    pub clock: Option<[(TexId, u32, u32); 2]>,
    /// What Network says: off, on, or the name of the network joined.
    pub network: Option<[(TexId, u32, u32); 2]>,
    pub faces: Option<&'a QuickMenuFaces>,
    /// The first row in view, from [`quick_window`].
    pub first: usize,
}

impl QuickMenu<'_> {
    /// The list over whatever is already drawn: the backdrop, under every menu.
    pub fn draw(&self, out: &mut Vec<Draw>) {
        out.push(Draw::Rect {
            x: 0.0,
            y: self.row_top(self.row) + BAR_INSET,
            w: out_w() as f32,
            h: QUICK_PITCH - 2.0 * BAR_INSET,
            colour: edge(),
        });
        let Some(faces) = self.faces else {
            return;
        };
        let (right, pad) = (out_w() as f32 - QUICK_EDGE, MENU_PAD as f32);
        let shown = quick_rows_shown(self.rows.len());
        for &row in self.rows.iter().skip(self.first).take(shown) {
            let y = self.row_top(row) + TYPE_DROP;
            let lit = row == self.row;
            if let Some(&(tex, w, h)) = faces.labels.get(row.index()) {
                push(out, tex, QUICK_EDGE - pad, y, w, h);
            }
            let value = match row {
                SettingsRow::DateTime => self.clock.map(|c| c[lit as usize]),
                SettingsRow::Network => self.network.map(|c| c[lit as usize]),
                _ => self.values[row.index()]
                    .and_then(|v| faces.values.get(v.index()))
                    .map(|v| v[lit as usize]),
            };
            let Some((tex, w, h)) = value else {
                continue;
            };
            if !lit || row.opens() {
                push(out, tex, right + pad - w as f32, y, w, h);
                continue;
            }
            let [(left_tex, lw, lh), (right_tex, rw, rh)] = faces.carets;
            let rx = right - rw as f32;
            push(out, right_tex, rx, y, rw, rh);
            let vx = rx - CARET_GAP + pad - w as f32;
            push(out, tex, vx, y, w, h);
            push(out, left_tex, vx + pad - CARET_GAP - lw as f32, y, lw, lh);
        }
        let [back, change, open] = faces.legend;
        let other = if self.row.opens() { open } else { change };
        for (tex, w, x) in centred_hints(&[back, other], LEGEND_GAP) {
            push(out, tex, x, legend_y(), w, HINT_H);
        }
    }
}

impl QuickMenu<'_> {
    fn row_top(&self, row: SettingsRow) -> f32 {
        let at = self.rows.iter().position(|r| *r == row).unwrap_or(0);
        let shown = quick_rows_shown(self.rows.len());
        quick_top_of(shown) + QUICK_PITCH * at.saturating_sub(self.first) as f32
    }
}

fn push(out: &mut Vec<Draw>, tex: TexId, x: f32, y: f32, w: u32, h: u32) {
    out.push(Draw::Tex {
        x: x.round(),
        y: y.round(),
        w: w as f32,
        h: h as f32,
        tex,
        alpha: 1.0,
    });
}
