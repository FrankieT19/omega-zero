//! A level on one line, as assets/ui/components/omega-gauge.js: a title tab naming it, then a
//! well of cells, as many lit as the level reaches, and an icon at its end, like the life gauge
//! in Mega Man Zero. It is a panel's header with the strip beside the tab made the gauge, and
//! shows the volume, the brightness, the blue light and the rewind left over a game.
//!
//! It is 15 art pixels tall, as tall as the bottom bar: a panel's frame line round it, and
//! inside it 13 rows. The tab fills them, its title in the middle; beside it, on the strip's
//! bright colour, the well is inset a pixel above, below and at the end, its cells on the
//! title's capitals' rows and the icon 3 pixels after them.

use super::palette::Palette;
use super::pixel::{blank, fill, ink, letters, mirror, nine_slice, paint, text_width};
use super::pixel::{ACCENT, ADVANCE, CAP};
use crate::plate::UndoFace;

/// The frame round it, as round a panel: its stripe, the corner pixel left out, round the frame
/// colour.
#[rustfmt::skip]
const FRAME: [&str; 2] = [
    ".s",
    "sA",
];

/// The well's corner: its shadow line, the same way, round the well.
#[rustfmt::skip]
const WELL: [&str; 2] = [
    ".e",
    "ew",
];

/// How tall the gauge is, its frame line included, and how tall inside it: the tab's frame lines,
/// two rows either side of the capitals, and the capitals.
pub const GAUGE_H: u32 = 15;
const H: u32 = GAUGE_H - 2;

/// The room inside the tab on either side of its title.
const TAB_PAD: u32 = 4;

/// The room inside the well either side of what it holds, and between the cells and the icon.
const WELL_PAD: u32 = 2;
const ICON_GAP: u32 = 3;

/// A cell's width, and the room between two.
const CELL_W: u32 = 3;
const CELL_GAP: u32 = 1;

/// An icon's side: as tall as the capitals.
const ICON: u32 = 7;

/// The icon at a gauge's end, a 7x7 sprite drawn in cream.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum GaugeIcon {
    Volume,
    Muted,
    Headphones,
    Brightness,
    BlueLight,
    Rewind,
    FastForward,
}

impl GaugeIcon {
    pub const ALL: [GaugeIcon; 7] = [
        GaugeIcon::Volume,
        GaugeIcon::Muted,
        GaugeIcon::Headphones,
        GaugeIcon::Brightness,
        GaugeIcon::BlueLight,
        GaugeIcon::Rewind,
        GaugeIcon::FastForward,
    ];

    /// The sprite, rows of '#' for the ink.
    #[rustfmt::skip]
    pub fn rows(self) -> [&'static str; ICON as usize] {
        match self {
            GaugeIcon::Volume => [
                "...#...",
                "..##.#.",
                "####..#",
                "####..#",
                "####..#",
                "..##.#.",
                "...#...",
            ],
            GaugeIcon::Muted => [
                "...#...",
                "..##...",
                "###.#.#",
                "###..#.",
                "###.#.#",
                "..##...",
                "...#...",
            ],
            GaugeIcon::Headphones => [
                ".#####.",
                "#.....#",
                "#.....#",
                "#.....#",
                "##...##",
                "##...##",
                "##...##",
            ],
            GaugeIcon::Brightness => [
                "#..#..#",
                ".#...#.",
                "..###..",
                "#.###.#",
                "..###..",
                ".#...#.",
                "#..#..#",
            ],
            GaugeIcon::BlueLight => [
                "..##...",
                ".##....",
                "##.....",
                "##.....",
                "##....#",
                ".##..##",
                "..####.",
            ],
            GaugeIcon::Rewind => [
                "...#..#",
                "..##.##",
                ".######",
                "#######",
                ".######",
                "..##.##",
                "...#..#",
            ],
            GaugeIcon::FastForward => [
                "#..#...",
                "##.##..",
                "######.",
                "#######",
                "######.",
                "##.##..",
                "#..#...",
            ],
        }
    }
}

/// How many of `cells` are lit for `value` of `max`: round(value / max * cells), so 0 lights
/// none and `max` all of them. `value` is held to 0..=max, and a `max` of 0 is taken as 100.
pub fn lit_cells(value: u32, max: u32, cells: u32) -> u32 {
    let max = if max == 0 { 100 } else { max };
    let value = value.min(max);
    (value as f64 / max as f64 * cells as f64).round() as u32
}

/// How wide the tab is with `label` in it: its frame line on the left, the title and the room
/// either side of it.
fn tab_width(label: &str) -> u32 {
    1 + 2 * TAB_PAD + text_width(label)
}

/// How wide the well is with `cells` and, if there is one, the icon: its shadow line either
/// side, the room inside it, the cells and the icon after them.
fn well_width(cells: u32, icon: bool) -> u32 {
    let cells = (cells * (CELL_W + CELL_GAP)).saturating_sub(CELL_GAP);
    let icon = if icon { ICON_GAP + ICON } else { 0 };
    2 + 2 * WELL_PAD + cells + icon
}

/// How wide the gauge is: its frame line either side, the tab, its slanted end, and the strip,
/// which is the well and the pixel after it.
pub fn gauge_width(label: &str, cells: u32, icon: bool) -> u32 {
    1 + tab_width(label) + H + 1 + well_width(cells, icon) + 1 + 1
}

/// The tab's slanted end, as a panel's: one pixel further right each row, its top and bottom
/// rows in the frame colour as the tab's outline, then a frame pixel and a stripe pixel, then
/// nothing, where the strip shows.
fn slant() -> Vec<String> {
    let h = H as usize;
    (0..h)
        .map(|y| {
            let fill = if y == 0 || y == h - 1 { "A" } else { "t" };
            fill.repeat(y) + "As" + &".".repeat(h + 1 - y - 2)
        })
        .collect()
}

/// The gauge titled `label`, `cells` cells with `lit_cells(value, max, cells)` of them lit from
/// the left, and `icon` after them: lit cells in the label colour, the rest in the plate colour,
/// the title and the icon in cream. `label` is drawn in capitals whatever case it is given in.
pub fn gauge_face(
    label: &str,
    value: u32,
    max: u32,
    cells: u32,
    icon: Option<GaugeIcon>,
    pal: &Palette,
) -> UndoFace {
    let label = label.to_uppercase();
    let w = gauge_width(&label, cells, icon.is_some());
    let mut face = blank(w, GAUGE_H);
    let rows = H as i32;
    nine_slice(
        &mut face,
        &mirror(&FRAME),
        [0, 0, w as i32, GAUGE_H as i32],
        [1; 4],
        [1; 4],
        pal,
    );

    // the tab, outlined in the frame colour on every side but its slanted one, its title 4
    // pixels in and its capitals two rows below its frame line
    let tab = tab_width(&label) as i32;
    fill(&mut face, [1, 1, tab, rows], pal.tab);
    fill(&mut face, [1, 1, tab, 1], pal.frame);
    fill(&mut face, [1, rows, tab, 1], pal.frame);
    fill(&mut face, [1, 1, 1, rows], pal.frame);
    let capitals = &letters(&label, ADVANCE)[ACCENT..ACCENT + CAP];
    let top = 1 + 1 + 2;
    ink(&mut face, capitals, 1 + 1 + TAB_PAD as i32, top, pal.cream);

    // the slant and the strip, on the strip's bright colour
    let slant_x = 1 + tab;
    fill(
        &mut face,
        [slant_x, 1, w as i32 - slant_x - 1, rows],
        pal.frame_hi,
    );
    paint(&mut face, &slant(), slant_x, 1, pal);

    // the well, a pixel in from the strip's top, bottom and end
    let well_x = slant_x + rows + 1;
    let well = [
        well_x,
        2,
        well_width(cells, icon.is_some()) as i32,
        rows - 2,
    ];
    nine_slice(&mut face, &mirror(&WELL), well, [1; 4], [1; 4], pal);

    let lit = lit_cells(value, max, cells);
    let mut x = well_x + 1 + WELL_PAD as i32;
    for i in 0..cells {
        let colour = if i < lit { pal.label } else { pal.plate };
        fill(&mut face, [x, top, CELL_W as i32, CAP as i32], colour);
        x += (CELL_W + CELL_GAP) as i32;
    }
    if let Some(icon) = icon {
        let x = x - CELL_GAP as i32 + ICON_GAP as i32;
        ink(&mut face, &icon.rows(), x, top, pal.cream);
    }
    face
}
