//! The bar along the bottom of a screen, as assets/ui/components/omega-bar.js: an action at
//! each end (action.rs) and, between them, the game's plate, which sits a little under the
//! actions' top rails and runs on behind both buttons' rounded ends, so the bar's bottom line
//! never breaks. The keys a screen has beyond the two actions are shown on the plate as hints:
//! each its key in a small dark cap, then the word, centred on the plate.
//!
//! The bar fills the screen's width, 320 art pixels at 640 and 360 at 720, and is `BAR` tall,
//! the actions' height; their notches rise `RISE` above it, over whatever is there. It is three
//! faces: the plate ([`plate_face`]) and the two actions (`action_face`), which [`bar_layout`]
//! places. Draw the plate, then the actions over it ([`BarLayout::plate_quad`],
//! [`BarLayout::action_quad`]); each action moves on its own, which is how a change of actions
//! sinks the old ones and raises the new ([`action_swap`]).

use omega_gfx::{Draw, TexId};

use super::action::{ActionFace, Side, BAR, TALL};
use super::palette::Palette;
use super::pixel::{blank, fill, ink, letters, nine_slice, quad, text_width, ACCENT, ADVANCE, CAP};
use crate::plate::UndoFace;
use crate::sprite::SPRITE_PX;

/// How far down the bar the plate starts, under the actions' top rails, and how far it runs on
/// behind each button's rounded end.
pub const PLATE_TOP: u32 = 2;
pub const UNDER: u32 = 5;
/// The plate's height: down to the bar's bottom line, which is its own last row.
pub const PLATE_H: u32 = BAR - PLATE_TOP;

/// How tall a key's cap is: a dark row above and below its letters' capitals.
pub const KEY_H: u32 = 9;
/// How far into the plate, under its top line, the hints' line starts: the top line is the bar's
/// row 2, so their line starts on row 3 and their capitals, two rows down it, are on rows 5 to
/// 11, the rows of the words in the actions' buttons. Each key's cap starts a row above its
/// capitals.
const HINTS_TOP: u32 = 0;
/// The gap between a key's cap and its word, and between two hints, in pixels of ink to ink.
const GAP: u32 = 3;
const BETWEEN: u32 = 8;

/// A key's cap: its left end, the column that stretches, its right end; dark, with its four
/// corner pixels left out.
#[rustfmt::skip]
const KEY: [&str; KEY_H as usize] = [
    ".k.",
    "kkk",
    "kkk",
    "kkk",
    "kkk",
    "kkk",
    "kkk",
    "kkk",
    ".k.",
];

/// A key and what it does, shown on the plate: "Y" "FORGET". The word may be empty.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Hint<'a> {
    pub key: &'a str,
    pub word: &'a str,
}

/// How wide a key's cap is: its letters, two dark pixels before them and one after (the
/// letters' own blank pixel makes it two).
pub fn cap_width(key: &str) -> u32 {
    text_width(key) + 3
}

/// How wide `hint` is, its cap's left to the end of its word's box (whose last pixel is blank).
pub fn hint_width(hint: &Hint) -> u32 {
    let word = match hint.word {
        "" => 0,
        word => GAP + text_width(word),
    };
    cap_width(hint.key) + word
}

/// How wide `hints` are together from their first ink to their last, what they are centred
/// on the plate by: each hint, `BETWEEN` pixels of ink to ink between them, and the last
/// word's blank pixel left off.
pub fn hints_width(hints: &[Hint]) -> u32 {
    let boxes: u32 = hints.iter().map(hint_width).sum();
    let between = hints.len().saturating_sub(1) as u32 * (BETWEEN - 1);
    let blank = match hints.last() {
        Some(last) if !last.word.is_empty() => 1,
        _ => 0,
    };
    boxes + between - blank
}

/// The plate `w` art pixels wide and `PLATE_H` tall, with `hints` on it: its top line, a lit
/// row, the plate colour, a shadow row, and the bar's bottom line in the frame colour, level
/// with the actions' own. The hints are centred on it, rounded down to a whole pixel, and start
/// at its left when they are wider than it.
pub fn plate_face(w: u32, hints: &[Hint], pal: &Palette) -> UndoFace {
    let mut face = blank(w, PLATE_H);
    let (wi, h) = (w as i32, PLATE_H as i32);
    fill(&mut face, [0, 0, wi, h], pal.plate);
    fill(&mut face, [0, 0, wi, 1], pal.edge);
    fill(&mut face, [0, 1, wi, 1], pal.band);
    fill(&mut face, [0, h - 2, wi, 1], pal.edge);
    fill(&mut face, [0, h - 1, wi, 1], pal.frame);

    let y = (1 + HINTS_TOP) as i32;
    let mut x = ((wi - hints_width(hints) as i32) / 2).max(0);
    for (i, hint) in hints.iter().enumerate() {
        if i > 0 {
            x += (BETWEEN - 1) as i32;
        }
        // the key's letters start two dark pixels in: the cap's end column and one more
        // the cap starts a row above the capitals, which are ACCENT rows down the hint's line
        let cap_y = y + (ACCENT - (KEY_H as usize - CAP) / 2) as i32;
        let cap = [x, cap_y, cap_width(hint.key) as i32, KEY_H as i32];
        nine_slice(&mut face, &KEY, cap, [0, 1, 0, 1], [0, 1, 0, 1], pal);
        ink(&mut face, &letters(hint.key, ADVANCE), x + 2, y, pal.cream);
        x += cap[2];
        if !hint.word.is_empty() {
            x += GAP as i32;
            ink(&mut face, &letters(hint.word, ADVANCE), x, y, pal.cream);
            x += text_width(hint.word) as i32;
        }
    }
    face
}

/// Where a bar's pieces go, in art pixels from the bar's top left (the bar is `BAR` tall and
/// stands on the screen's bottom edge).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BarLayout {
    /// The bar's width: the screen's, in art pixels.
    pub width: u32,
    /// The plate's `[x, y, w, h]`: from `UNDER` inside the left action to `UNDER` inside the
    /// right one, `PLATE_TOP` down. Its face is `plate_face(plate[2], ..)`.
    pub plate: [u32; 4],
    /// Where each action's face goes, its top left: at the bar's ends, its notch rising
    /// above the bar (y is minus the action's `rise`).
    pub left: [i32; 2],
    pub right: [i32; 2],
}

/// The bar `width_u` art pixels wide with `left` and `right` at its ends: the plate takes what
/// is left between them and `UNDER` more under each.
pub fn bar_layout(width_u: u32, left: &ActionFace, right: &ActionFace) -> BarLayout {
    let (lw, rw) = (left.face.w, right.face.w);
    let plate_w = (width_u + 2 * UNDER).saturating_sub(lw + rw);
    BarLayout {
        width: width_u,
        plate: [lw.saturating_sub(UNDER), PLATE_TOP, plate_w, PLATE_H],
        left: [0, -(left.rise as i32)],
        right: [width_u as i32 - rw as i32, -(right.rise as i32)],
    }
}

impl BarLayout {
    /// How far down a screen `screen_h` pixels tall the bar's top is, in screen pixels.
    pub fn top(screen_h: u32) -> f32 {
        screen_h.saturating_sub(BAR * SPRITE_PX) as f32
    }

    /// The plate, its face uploaded as `tex`, on a screen `screen_h` pixels tall.
    pub fn plate_quad(&self, tex: TexId, screen_h: u32) -> Draw {
        let [x, y, w, h] = self.plate;
        let px = |art: u32| (art * SPRITE_PX) as f32;
        quad(tex, w, h, px(x), Self::top(screen_h) + px(y))
    }

    /// The action at `side`, `action` uploaded as `tex`, on a screen `screen_h` pixels tall,
    /// `sink` art pixels below its place (0 at rest, `SINK` out of sight).
    pub fn action_quad(
        &self,
        side: Side,
        tex: TexId,
        action: &ActionFace,
        screen_h: u32,
        sink: u32,
    ) -> Draw {
        let [x, y] = match side {
            Side::Left => self.left,
            Side::Right => self.right,
        };
        let px = |art: i32| (art * SPRITE_PX as i32) as f32;
        let (w, h) = (action.face.w, action.face.h);
        quad(tex, w, h, px(x), Self::top(screen_h) + px(y + sink as i32))
    }
}

/// How far an action sinks to be out of sight: its whole face, notch and all.
pub const SINK: u32 = TALL;

/// What a bar shows partway through a change of actions (the shelf's to a menu's, or back).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Swap {
    /// Whether it is the new actions, and the plate and hints laid out for them, that show.
    pub new: bool,
    /// How far down they are sunk from their place, in art pixels.
    pub sink: u32,
}

/// The bar `p` of the way through a slide that changes its actions (`p` the slide's eased
/// progress, 0 to 1), as flow/player.js moves it: the old actions sink out of the screen over
/// the first half and the new ones rise into place over the second, a whole art pixel at a
/// time. A screen's hints change at halfway too, and do even between two screens that keep
/// their actions (where the actions stay put).
pub fn action_swap(p: f32) -> Swap {
    let p = p.clamp(0.0, 1.0);
    let (new, k) = if p < 0.5 {
        (false, p / 0.5)
    } else {
        (true, (1.0 - p) / 0.5)
    };
    Swap {
        new,
        sink: (k * SINK as f32).round() as u32,
    }
}
