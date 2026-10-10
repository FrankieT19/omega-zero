//! One Wi-Fi network in a list, as assets/ui/components/omega-list-row.js: a row (row.rs) with
//! the network's name on the left in its own case and, on the right, what it is to the
//! handheld, a padlock if it needs a password and how strong it comes in.
//!
//! The icons stand on the capitals' rows, the padlock 5 art pixels wide like a letter and the
//! bars 7, 3 pixels apart and a letter's width (6) clear of the value, so they read as a pair of
//! their own; their room is kept when a row has none, so every row's value, padlock and bars
//! line up down the list. The icons are drawn in the name's colour. Everything else is a row's:
//! `ROW_H` tall, placed in a panel's well by `row_rect`, the same band, words and colours.

use super::palette::Palette;
use super::pixel::text_width;
use super::row::{art, base, colour, inked, line, name, word, Ink, Item};
use crate::plate::UndoFace;

/// The top bar's Wi-Fi icon (WIFI in status_bar.rs), its three parts numbered in the order they
/// light: 1 the dot, 2 the small arc, 3 the big one.
#[rustfmt::skip]
const SIGNAL: [&str; 7] = [
    "..333..",
    ".3...3.",
    "3.....3",
    "..222..",
    ".2...2.",
    ".......",
    "...1...",
];

/// A padlock as tall as the capitals and as wide as a letter: the shackle, then the body with
/// its keyhole.
#[rustfmt::skip]
const LOCK: [&str; 7] = [
    ".###.",
    "#...#",
    "#...#",
    "#####",
    "##.##",
    "##.##",
    "#####",
];

/// How far the padlock stays clear of the value, and the bars of the padlock.
const LOCK_GAP: i32 = 6;
const SIGNAL_GAP: i32 = 3;

/// The most bars a signal lights.
pub const BARS: u8 = 3;

/// One network, as omega-list-row's attributes.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct ListRow<'a> {
    /// The network's name, from the left, in its own case (names are the player's).
    pub label: &'a str,
    /// What it is to the handheld, from the right: CONNECTED, JOINING, SAVED. Optional.
    pub value: Option<&'a str>,
    /// 0 to 3: the Wi-Fi bars, that many of the three lit and the rest drawn faint (more than 3
    /// is 3). Without it there are no bars, and their room is kept.
    pub signal: Option<u8>,
    /// A padlock beside the bars, for a network with a password; its room is kept without.
    pub lock: bool,
    /// The row under the cursor, as a `Row`'s.
    pub lit: bool,
    /// A network that cannot be joined, as a `Row`'s.
    pub dim: bool,
}

/// The bars `level` of the way lit: the parts reached in `on`, the rest in `off`.
fn bars(level: u8, on: char, off: char) -> Vec<String> {
    SIGNAL
        .iter()
        .map(|row| {
            row.chars()
                .map(|c| match c.to_digit(10) {
                    Some(part) if part <= u32::from(level) => on,
                    Some(_) => off,
                    None => c,
                })
                .collect()
        })
        .collect()
}

/// A network's row `width_u` art pixels wide and `ROW_H` tall, drawn as omega-list-row draws
/// `row`.
pub fn list_row_face(width_u: u32, row: &ListRow, pal: &Palette) -> UndoFace {
    let ink = Ink::of(row.lit, row.dim);
    let value = row.value.filter(|v| !v.is_empty());

    // left to right after the name: the value, handing its last blank pixel back as a row's
    // does, then the padlock and the bars, whose room is always there
    let mut items = Vec::new();
    if let Some(value) = value {
        items.push(Item {
            w: text_width(value),
            before: 0,
            after: -1,
        });
    }
    let lock = Item {
        w: LOCK[0].len() as u32,
        before: LOCK_GAP,
        after: 0,
    };
    let signal = Item {
        w: SIGNAL[0].len() as u32,
        before: SIGNAL_GAP,
        after: 0,
    };
    items.extend([lock, signal]);
    let (room, xs) = line(width_u, &items);
    let mut xs = xs.into_iter();

    let mut face = base(width_u, row.lit, pal);
    name(&mut face, row.label, room, colour(pal, ink.name));
    if let Some(value) = value {
        let x = xs.next().expect("the value's place");
        word(&mut face, value, x, colour(pal, ink.value));
    }
    let (lock_x, signal_x) = (xs.next(), xs.next());
    if row.lock {
        let x = lock_x.expect("the padlock's place");
        art(&mut face, &inked(&LOCK, ink.name), x, pal);
    }
    if let Some(level) = row.signal {
        let x = signal_x.expect("the bars' place");
        art(&mut face, &bars(level.min(BARS), ink.name, ink.off), x, pal);
    }
    face
}
