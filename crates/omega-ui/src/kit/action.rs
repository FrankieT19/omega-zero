//! The bottom bar's actions, as assets/ui/components/action-left.js and action-right.js: the
//! game's STATUS / L1 BUTTON on the left and R1 BUTTON / ELF LIST on the right. Left to right,
//! the left one is the arrow in its notch at the screen's edge, a big word, and a button pill
//! with a hint; the right one is the same mirrored, parts and all.
//!
//! It is `BAR` art pixels tall and as wide as its text, with room kept when there is none. The
//! notch rises `RISE` pixels above it, over whatever is there, as the game's does over the
//! backdrop, so the face is `BAR + RISE` tall and clear above the bar beside the notch.

use super::palette::Palette;
use super::pixel::{
    big_word, blank, fill, flip, ink, letters, nine_slice, paint, text_width, ADVANCE,
};
use crate::plate::UndoFace;

/// The bar's height in art pixels.
pub const BAR: u32 = 21;
/// How far the notch rises above the bar.
pub const RISE: u32 = 6;
/// The notch's width.
pub const NOTCH: u32 = 16;
/// The face's height, the notch's.
pub const TALL: u32 = BAR + RISE;
/// Where the bracket over the notch starts down toward the bar's top rail.
const KNEE: u32 = 4;

/// A return arrow: a big head at the top pointing in, toward the word, on a thick band that runs
/// back, turns down and comes round underneath.
#[rustfmt::skip]
const ARROW: [&str; 12] = [
    "........#...",
    "........##..",
    "....#######.",
    "..##########",
    ".##########.",
    "####....##..",
    "###.....#...",
    "###.........",
    "####........",
    ".#######....",
    "..######....",
    "....####....",
];

/// The button pill, the bar's full height: its left edge, the column that stretches (frame
/// colour on top, a band, a frame line, dark, frame colour at the bottom), then the rounded end.
#[rustfmt::skip]
const PILL: [&str; BAR as usize] = [
    "AAAA....",
    "AbbbAA..",
    "AbbbbbA.",
    "AbbbbbA.",
    "AAAAAAAA",
    "AkkkkkkA",
    "AkkkkkkA",
    "AkkkkkkA",
    "AkkkkkkA",
    "AkkkkkkA",
    "AkkkkkkA",
    "AkkkkkkA",
    "AkkkkkkA",
    "AkkkkkkA",
    "AkkkkkkA",
    "AkkkkkkA",
    "AkkkkkkA",
    "AkkkkkA.",
    "AkkkkkA.",
    "AkkkAA..",
    "AAAA....",
];
/// How wide the pill's rounded end is.
const PILL_END: u32 = 6;

/// The room either side of the word: 4 dark pixels before it, 3 after its shadow column.
const WORD_PAD: [u32; 2] = [4, 3];
/// How wide the word's box is kept when there is no word.
const WORD_MIN: u32 = 46;

/// The hint's box: never narrower than this, with this room before and after its text.
const HINT_MIN: u32 = 43;
const HINT_PAD: [u32; 2] = [4, 3];
/// How far the hint's box reaches into the pill's rounded end, and how far down the bar its
/// line starts: its capitals fill rows 9 to 15, four dark rows each side between the band and
/// the bottom.
const HINT_REACH: u32 = 5;
const HINT_TOP: u32 = 7;

/// Which end of the bar an action stands at.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Side {
    Left,
    Right,
}

/// An action's face.
pub struct ActionFace {
    pub face: UndoFace,
    /// How far down the face the bar starts: the face goes on the screen this far above it.
    pub rise: u32,
}

/// The notch as rows of colour keys, drawn for the left action: the bracket over it, the dark
/// inside, the arrow over its shadow.
fn notch() -> Vec<String> {
    let (tall, notch, knee, rise) = (TALL as usize, NOTCH as usize, KNEE as usize, RISE as usize);
    let mut grid = vec![vec!['.'; notch]; tall];
    let run = |grid: &mut Vec<Vec<char>>, y: usize, from: usize, to: usize, key: char| {
        let to = to.min(notch);
        if from < to {
            grid[y][from..to].fill(key);
        }
    };
    // the bracket's frame line: along the top, down 45 degrees, then along the bar's rail
    let line = |r: usize| match r {
        0 => (0, knee + 1),
        r if r < rise => (knee + r, knee + r + 2),
        _ => (knee + rise, notch),
    };
    for y in 2..tall - 1 {
        let stripe_from = match y {
            y if y <= rise => knee + y - 1,
            y if y == rise + 1 => knee + rise,
            _ => notch,
        };
        run(&mut grid, y, 0, stripe_from, 'k');
    }
    for r in 0..=rise {
        let (from, to) = line(r);
        run(&mut grid, r, from, to, 'A');
    }
    // the stripe under it: where the line was a row above and is not on this row
    for (r, row) in grid.iter_mut().enumerate().take(rise + 2).skip(1) {
        let (from, to) = line(r - 1);
        for cell in &mut row[from..to] {
            if *cell != 'A' {
                *cell = 's';
            }
        }
    }
    run(&mut grid, tall - 1, 0, notch, 'A');

    // the arrow over its own shadow, one pixel down and right, for some depth, in the middle of
    // the bar between its rail and its bottom line
    for (dx, dy, key) in [(1, 1, 's'), (0, 0, 'H')] {
        for (y, row) in ARROW.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                if ch == '#' {
                    grid[rise + 4 + y + dy][2 + x + dx] = key;
                }
            }
        }
    }
    grid.into_iter()
        .map(|row| row.into_iter().collect())
        .collect()
}

/// How wide the word's box is with `word` in it: the word and its room, or `WORD_MIN`.
fn word_box(word: &str) -> u32 {
    match word {
        "" => WORD_MIN,
        _ => WORD_PAD[0] + big_word(word)[0].len() as u32 + WORD_PAD[1],
    }
}

/// How wide the hint's box is with `pill` in it.
fn hint_box(pill: &str) -> u32 {
    HINT_MIN.max(HINT_PAD[0] + text_width(pill) + HINT_PAD[1])
}

/// How wide an action is in art pixels with `word` and `pill`, at either side.
pub fn action_width(word: &str, pill: &str) -> u32 {
    NOTCH + word_box(word) + button_width(pill)
}

/// The pill's width: its edge, the hint's box less what reaches into the rounded end, the end.
fn button_width(pill: &str) -> u32 {
    1 + hint_box(pill) - HINT_REACH + PILL_END
}

/// The action at `side` of the bar: `word` big beside the arrow, `pill` small in the button.
/// Either may be empty, which keeps their room.
pub fn action_face(side: Side, word: &str, pill: &str, pal: &Palette) -> ActionFace {
    let w = action_width(word, pill);
    let mut face = blank(w, TALL);
    let (word_w, button_w) = (word_box(word) as i32, button_width(pill) as i32);
    let (bar, rise) = (BAR as i32, RISE as i32);
    let (button_x, word_x, notch_x) = match side {
        Side::Left => (NOTCH as i32 + word_w, NOTCH as i32, 0),
        Side::Right => (0, button_w, button_w + word_w),
    };

    let notch = match side {
        Side::Left => notch(),
        Side::Right => flip(&notch()),
    };
    paint(&mut face, &notch, notch_x, 0, pal);

    // the word's box: the dark of the bar, its top rail (frame colour over a stripe) and its
    // bottom line, then the word with its top on the rail
    fill(&mut face, [word_x, rise, word_w, bar], pal.dark);
    fill(&mut face, [word_x, rise + 1, word_w, 1], pal.stripe);
    fill(&mut face, [word_x, rise, word_w, 1], pal.frame);
    fill(&mut face, [word_x, rise + bar - 1, word_w, 1], pal.frame);
    if !word.is_empty() {
        paint(
            &mut face,
            &big_word(word),
            word_x + WORD_PAD[0] as i32,
            rise,
            pal,
        );
    }

    // the pill, its rounded end toward the word, and the hint in the middle of its dark inside
    let (rows, slice) = match side {
        Side::Left => (Vec::from(PILL.map(String::from)), [0, PILL_END, 0, 1]),
        Side::Right => (flip(&PILL), [0, 1, 0, PILL_END]),
    };
    let button = [button_x, rise, button_w, bar];
    nine_slice(&mut face, &rows, button, slice, slice, pal);
    let hint_x = match side {
        Side::Left => button_x + 1,
        Side::Right => button_x + PILL_END as i32 - HINT_REACH as i32,
    };
    // centred as text-align does it; what is left over is always even, since the box's room
    // (36) and every word (6 a letter) are, so the hint never lands on half a pixel
    let room = hint_box(pill) - HINT_PAD[0] - HINT_PAD[1];
    let centred = (room.saturating_sub(text_width(pill)) / 2) as i32;
    let text_x = hint_x + HINT_PAD[0] as i32 + centred;
    ink(
        &mut face,
        &letters(pill, ADVANCE),
        text_x,
        rise + HINT_TOP as i32,
        pal.cream,
    );

    ActionFace { face, rise: RISE }
}
