//! The About screen's label, the sticker on the back of the handheld redrawn as pixel art for
//! the canvas it is shown on, at one texel to an art pixel and never scaled: its words in Pixel
//! Code at one face pixel to an art pixel, the ANBERNIC RG SP wordmark and the DC symbol as
//! sprites, and the barcode's bars whole art pixels wide.
//!
//! The rating rows sit at its top left and the white panel with the barcode and the build's
//! serial at its top right, the wordmark and the author under the panel. The credits run under
//! the rating rows beside the panel and then across the sticker under it, and slot's credit is
//! the last line, in grey.

use omega_gfx::{Draw, TexId};

use crate::barcode::code39;
use crate::kit::layout::{centred, page};
use crate::kit::pixel::{big_word, blank, fill, ink, letters, Font, ACCENT, ADVANCE, LINE};
use crate::plate::UndoFace;
use crate::sprite::SPRITE_PX;

type Rgb = [u8; 3];

const BLACK: Rgb = [0x23, 0x1f, 0x20];
const WHITE: Rgb = [0xff, 0xff, 0xff];
/// The line owed to slot, kept plain and last.
const GREY: Rgb = [0x8a, 0x86, 0x87];

pub const DC: char = '\u{2393}';
const COPY: char = '\u{a9}';

pub struct StickerFields<'a> {
    pub battery: Option<u8>,
    pub serial: &'a str,
    pub dirty_digit: char,
}

/// What omega-zero is made with and from, as one paragraph the sticker wraps to its width.
pub const CREDITS: &str = "EMULATION BY MGBA AND GPSP. UNDERLYING OS IS BASEOS BY PVAIBHAV. \
    TYPE IS PIXEL CODE BY QWERASD AND NERD FONTS SYMBOLS BY RYAN L MCINTYRE. PANEL MASK FROM \
    GIGAHERZ'S LCD3X. LABELS FROM SCREENSCRAPER. BUILT WITH CLAUDE. FREE SOFTWARE UNDER THE \
    GPL-3.0.";

pub const ORIGIN: [&str; 2] = ["S/OMEGA-ZERO", "MADE ON BASEOS"];

/// Who made it.
pub const COPYRIGHT: &str = "\u{a9} 2026 OT\u{c1}VIO (@OAURAMOS)";

pub const HOME: &str = "SEE README.";

/// What omega-zero is based on, the sticker's last line.
pub const BASED_ON: &str = "BASED ON SLOT BY BRANDON T. KOWALSKI (GPL-3.0).";

pub fn head_rows(f: &StickerFields) -> [String; 3] {
    [
        "MODEL NO. AGS-102".into(),
        format!("INPUT: 5V{DC}1.5A"),
        match f.battery {
            Some(p) => format!("BATTERY: LI-ION ({p}%)"),
            None => "BATTERY: LI-ION".to_string(),
        },
    ]
}

/// Every line of words on the sticker, the credits as one.
pub fn sticker_lines(f: &StickerFields) -> Vec<String> {
    let mut out: Vec<String> = head_rows(f).into();
    out.extend(ORIGIN.iter().map(|s| s.to_string()));
    out.push(format!("{} {}", f.serial, f.dirty_digit));
    out.push(COPYRIGHT.to_string());
    out.push(HOME.to_string());
    out.push(CREDITS.to_string());
    out.push(BASED_ON.to_string());
    out
}

/// The art pixels left clear either side of the sticker.
const SIDE: u32 = 10;
/// The sticker's margin inside its edge.
const MARGIN: i32 = 6;
/// The left column, as wide as the longest rating row (22 letters), and the room between it
/// and the panel.
const COLUMN: i32 = 22 * ADVANCE as i32;
const GUTTER: i32 = 6;
/// The panel: its top, the bars' height and the room round them.
const PANEL_TOP: i32 = 4;
const BARS_H: i32 = 24;
const PANEL_PAD: i32 = 4;
/// How tall the serial's big figures are, and the box round the digit after it.
const BIG_H: i32 = 10;
const BOX: [i32; 2] = [11, 14];
/// The notch in the sticker's bottom edge: how deep, how wide across its top, and how far it
/// widens each row down.
const NOTCH: [i32; 3] = [10, 48, 2];
/// The rows from a line's top to its capitals' top.
const CAP_TOP: i32 = ACCENT as i32;

/// Where everything goes on a sticker `w` art pixels wide.
struct Plan {
    w: i32,
    h: i32,
    panel: [i32; 4],
    wordmark: [i32; 2],
    copyright: [i32; 2],
    home: [i32; 2],
    origin: [[i32; 2]; 2],
    /// The credits' lines and where each starts, then slot's.
    credits: Vec<(String, [i32; 2])>,
    based_on: Vec<(String, [i32; 2])>,
}

fn plan(w: i32) -> Plan {
    let panel_x = MARGIN + COLUMN + GUTTER;
    let panel_w = w - MARGIN - panel_x;
    let panel_h = PANEL_PAD + BARS_H + 3 + BOX[1] + 3;
    let right = w - MARGIN;
    let mut y = PANEL_TOP + panel_h + 4;
    let wordmark = [right - wordmark_width(), y];
    y += WORDMARK.len() as i32 + 3;
    let copyright = [right - line_width(COPYRIGHT) + 1, y - CAP_TOP];
    y += LINE as i32;
    let home = [right - line_width(HOME) + 1, y - CAP_TOP];
    let beside_until = y + LINE as i32;

    let top = PANEL_TOP + 3 * LINE as i32 + 4;
    let origin = [[MARGIN, top], [MARGIN, top + LINE as i32]];
    // beside what is under the panel the lines keep to the left, two letters clear of the
    // author's line, then they run across the sticker
    let beside = copyright[0] - MARGIN - 2 * ADVANCE as i32;
    let room = |top: i32| match top < beside_until {
        true => beside,
        false => w - 2 * MARGIN,
    };
    let (credits, top) = wrap(CREDITS, top + 2 * LINE as i32 + 4, room);
    let (based_on, top) = wrap(BASED_ON, top + 3, room);
    let h = top + 3 + NOTCH[0];
    Plan {
        w,
        h,
        panel: [panel_x, PANEL_TOP, panel_w, panel_h],
        wordmark,
        copyright,
        home,
        origin,
        credits,
        based_on,
    }
}

/// `text` set in lines from `top` down, each as long as `room` says a line at its top can be,
/// and where the next line would start.
fn wrap(text: &str, mut top: i32, room: impl Fn(i32) -> i32) -> (Vec<(String, [i32; 2])>, i32) {
    let mut lines = Vec::new();
    let mut words = text.split_whitespace().peekable();
    while words.peek().is_some() {
        let mut line = String::new();
        while let Some(word) = words.peek() {
            let longer = match line.is_empty() {
                true => word.to_string(),
                false => format!("{line} {word}"),
            };
            if line_width(&longer) - 1 > room(top) && !line.is_empty() {
                break;
            }
            line = longer;
            words.next();
        }
        lines.push((line, [MARGIN, top]));
        top += LINE as i32;
    }
    (lines, top)
}

/// How wide `line` is printed, the blank column after its last letter included: a letter's
/// advance each, two more for the © drawn wider.
fn line_width(line: &str) -> i32 {
    line.chars()
        .map(|c| match c {
            COPY => COPYRIGHT_W + 1,
            _ => ADVANCE as i32,
        })
        .sum()
}

/// The sticker's size in art pixels on this canvas: as wide as the page but for `SIDE` each
/// side, as tall as what it holds.
pub fn sticker_size() -> (u32, u32) {
    let p = plan((page()[0] - 2 * SIDE) as i32);
    (p.w as u32, p.h as u32)
}

pub fn sticker_face(f: &StickerFields) -> UndoFace {
    let (w, _) = sticker_size();
    let p = plan(w as i32);
    let mut face = blank(p.w as u32, p.h as u32);
    shape(&mut face, p.w, p.h);

    for (n, row) in head_rows(f).iter().enumerate() {
        print(
            &mut face,
            row,
            MARGIN,
            PANEL_TOP + n as i32 * LINE as i32,
            WHITE,
        );
    }
    for (line, at) in ORIGIN.iter().zip(p.origin) {
        print(&mut face, line, at[0], at[1], WHITE);
    }
    for (line, [x, y]) in &p.credits {
        print(&mut face, line, *x, *y, WHITE);
    }
    for (line, [x, y]) in &p.based_on {
        print(&mut face, line, *x, *y, GREY);
    }

    panel(&mut face, p.panel, f);
    paint(&mut face, &WORDMARK, p.wordmark[0], p.wordmark[1], WHITE);
    print(&mut face, COPYRIGHT, p.copyright[0], p.copyright[1], WHITE);
    print(&mut face, HOME, p.home[0], p.home[1], WHITE);
    face
}

/// The sticker's black shape: a rectangle with its corners rounded and the notch out of the
/// middle of its bottom edge.
fn shape(face: &mut UndoFace, w: i32, h: i32) {
    const CORNER: [i32; 4] = [3, 2, 1, 1];
    let [deep, top_w, slope] = NOTCH;
    for y in 0..h {
        let side = CORNER
            .get(y as usize)
            .or(CORNER.get((h - 1 - y) as usize))
            .copied()
            .unwrap_or(0);
        let into = y - (h - deep);
        let half = match into >= 0 {
            true => top_w / 2 + into * slope,
            false => 0,
        };
        fill(face, [side, y, w / 2 - half - side, 1], BLACK);
        fill(face, [w / 2 + half, y, w - side - w / 2 - half, 1], BLACK);
    }
}

/// The white panel: its corners cut, the Code 39 barcode of the build across it, its bars one
/// art pixel narrow and two wide, and under them the serial in big figures beside the digit
/// that says whether the build was dirty, in a box.
fn panel(face: &mut UndoFace, [x, y, w, h]: [i32; 4], f: &StickerFields) {
    fill(face, [x + 1, y, w - 2, h], WHITE);
    fill(face, [x, y + 1, w, h - 2], WHITE);

    let bars_y = y + PANEL_PAD;
    if let Some(run) = code39(&format!("*{}-{}*", f.serial, f.dirty_digit)) {
        let width = |wide: bool| if wide { 2 } else { 1 };
        let total: i32 =
            run.iter().map(|&wide| width(wide)).sum::<i32>() + run.len() as i32 / 9 - 1;
        let mut bx = x + (w - total) / 2;
        for symbol in run.chunks_exact(9) {
            for (n, &wide) in symbol.iter().enumerate() {
                if n % 2 == 0 {
                    fill(face, [bx, bars_y, width(wide), BARS_H], BLACK);
                }
                bx += width(wide);
            }
            bx += 1;
        }
    }

    let big = big_word(f.serial, Font::Bold);
    let big_w = big.first().map_or(0, |r| r.len()) as i32;
    let total = big_w + 3 + BOX[0];
    let sx = x + (w - total) / 2;
    let box_y = bars_y + BARS_H + 3;
    let sy = box_y + (BOX[1] - BIG_H) / 2;
    paint_keyed(face, &big, sx, sy, 'L', BLACK);
    let bx = sx + big_w + 3;
    let [bw, bh] = BOX;
    fill(face, [bx, box_y, bw, 1], BLACK);
    fill(face, [bx, box_y + bh - 1, bw, 1], BLACK);
    fill(face, [bx, box_y, 1, bh], BLACK);
    fill(face, [bx + bw - 1, box_y, 1, bh], BLACK);
    let digit = big_word(&f.dirty_digit.to_string(), Font::Bold);
    paint_keyed(face, &digit, bx + 2, sy, 'L', BLACK);
}

/// `line` in Pixel Code with its line's top at `y`: its capitals two rows down, as every line in
/// the kit. The DC symbol and the © are drawn as sprites of the same height.
fn print(face: &mut UndoFace, line: &str, x: i32, y: i32, colour: Rgb) {
    let mut at = x;
    let mut run = String::new();
    let flush = |face: &mut UndoFace, run: &mut String, at: &mut i32| {
        if !run.is_empty() {
            ink(face, &letters(run, ADVANCE), *at, y, colour);
            *at += (run.chars().count() * ADVANCE) as i32;
            run.clear();
        }
    };
    for c in line.chars() {
        match c {
            DC => {
                flush(face, &mut run, &mut at);
                paint(face, &DC_SPRITE, at, y + CAP_TOP, colour);
                at += ADVANCE as i32;
            }
            COPY => {
                flush(face, &mut run, &mut at);
                paint(face, &COPYRIGHT_SPRITE, at, y + CAP_TOP, colour);
                at += COPYRIGHT_W + 1;
            }
            _ => run.push(c),
        }
    }
    flush(face, &mut run, &mut at);
}

/// `rows`' '#' pixels painted in `colour` with their top left at `x, y`.
fn paint(face: &mut UndoFace, rows: &[&str], x: i32, y: i32, colour: Rgb) {
    for (dy, row) in rows.iter().enumerate() {
        for (dx, key) in row.bytes().enumerate() {
            if key == b'#' {
                fill(face, [x + dx as i32, y + dy as i32, 1, 1], colour);
            }
        }
    }
}

fn paint_keyed(face: &mut UndoFace, rows: &[String], x: i32, y: i32, key: char, colour: Rgb) {
    for (dy, row) in rows.iter().enumerate() {
        for (dx, k) in row.chars().enumerate() {
            if k == key {
                fill(face, [x + dx as i32, y + dy as i32, 1, 1], colour);
            }
        }
    }
}

/// ⎓, direct current, as tall as a capital: a bar over three dashes.
const DC_SPRITE: [&str; 7] = [
    ".....", ".....", "#####", ".....", "#.#.#", ".....", ".....",
];

/// ©, a C in a ring as tall as a capital, a pixel wider each side than a letter.
const COPYRIGHT_W: i32 = 7;
const COPYRIGHT_SPRITE: [&str; 7] = [
    "..###..", ".#...#.", "#.###.#", "#.#...#", "#.###.#", ".#...#.", "..###..",
];

/// ANBERNIC in heavy italic capitals, then RG SP in squared ones, as the handheld's wordmark
/// has them.
const WORDMARK: [&str; 9] = [
    "...#####..##...##.######..######.######..##...##.##..#####...........................................",
    "..##...##.###..##.##...##.##.....##...##.###..##.##.##...##....#######..########....########.#######.",
    "..##...##.###..##.##...##.##.....##...##.###..##.##.##.........##....##.##..........##.......##....##",
    ".##...##.####.##.##..##..##.....##...##.####.##.##.##..........##....##.##..........##.......##....##",
    ".#######.##.####.######..#####..######..##.####.##.##..........#######..##..####....########.#######.",
    ".##...##.##..###.##...##.##.....##.##...##..###.##.##..........##...##..##....##..........##.##......",
    "##...##.##..###.##...##.##.....##..##..##..###.##.##...........##....##.##....##..........##.##......",
    "##...##.##...##.##...##.##.....##...##.##...##.##.##...##......##....##.########....########.##......",
    "##...##.##...##.######..######.##...##.##...##.##..#####.............................................",
];

fn wordmark_width() -> i32 {
    WORDMARK.iter().map(|r| r.len()).max().unwrap_or(0) as i32
}

/// Where the sticker sits on the canvas, in screen pixels: in the middle of the page above the
/// bottom bar, at `SPRITE_PX` to a texel, on whole art pixels.
pub fn sticker_rect() -> (f32, f32, f32, f32) {
    let (w, h) = sticker_size();
    let [x, y, w, h] = centred(w, h);
    let px = |art: u32| (art * SPRITE_PX) as f32;
    (px(x), px(y), px(w), px(h))
}

pub fn draw_sticker(face: Option<TexId>, out: &mut Vec<Draw>) {
    let Some(tex) = face else {
        return;
    };
    let (x, y, w, h) = sticker_rect();
    out.push(Draw::Tex {
        x,
        y,
        w,
        h,
        tex,
        alpha: 1.0,
    });
}
