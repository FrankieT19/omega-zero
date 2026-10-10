//! The bar across the top of the shelf: what the handheld is connected to on the left, the
//! time in the middle and the battery on the right.
//!
//! Everything on it stands in one band, a sprite cell tall: the icons are drawn on the text's
//! own pixel grid, and the words hang so their capitals fill the same band. All of it is
//! printed in the ink its [`ChromeColours`] give, the palette's cream or a theme's white, so
//! the faces are baked in that ink and baked again when it changes.

use omega_gfx::{out_w, Draw, TexId};
use omega_power::Battery;

use crate::battery::{draw_gauge, gauge_width};
use crate::footer::{draw_printed, Printed};
use crate::hud::HUD_INK;
use crate::plate::{word_cap_top, word_tail, UndoFace, HINT_H};
use crate::slot_chrome::ChromeColours;
use crate::sprite::{cropped, sprite_face, Rows, CELL_PX, SPRITE_PX};
use crate::text;

pub const TOP_BAR_H: f32 = 36.0;
const MARGIN: f32 = 24.0;
const GAP: f32 = 4.0 * SPRITE_PX as f32;
const SSH_PX: f32 = 18.0;
const RULE_H: f32 = 2.0;

/// What an item is drawn in while it is on, and while it is off, on a card whose theme.txt sets
/// the chrome: the white and grey the bar was printed in before it took the palette.
pub const LINK_ON: [u8; 3] = HUD_INK;
pub const LINK_OFF: [u8; 3] = [0x5e, 0x5d, 0x66];

#[rustfmt::skip]
const WIFI: Rows = [
    "..###..",
    ".#...#.",
    "#.....#",
    "..###..",
    ".#...#.",
    ".......",
    "...#...",
];

#[rustfmt::skip]
const BLUETOOTH: Rows = [
    "...#...",
    "...##..",
    ".#.#.#.",
    "..###..",
    ".#.#.#.",
    "...##..",
    "...#...",
];

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Link {
    Wifi,
    Bluetooth,
    Ssh,
}

impl Link {
    pub const ALL: [Link; 3] = [Link::Wifi, Link::Bluetooth, Link::Ssh];

    /// The icon's sprite. A cell this small has no room to cross an icon out, so off is told
    /// by its ink alone and the shape stays put.
    fn sprite(self) -> Option<&'static Rows> {
        match self {
            Link::Wifi => Some(&WIFI),
            Link::Bluetooth => Some(&BLUETOOTH),
            Link::Ssh => None,
        }
    }
}

/// What the handheld is connected to, as the bar shows it.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct Links {
    pub wifi: bool,
    /// None on a handheld with no Bluetooth radio, which then shows no icon for it.
    pub bluetooth: Option<bool>,
    /// The handheld can be reached over SSH and SFTP.
    pub ssh: bool,
}

/// One item's face, on or off in the ink of `colours`: an icon one cell square, or "ssh" in
/// bold cut to its ink.
pub fn link_face(link: Link, on: bool, colours: &ChromeColours) -> UndoFace {
    let ink = colours.link_ink(on);
    match link.sprite() {
        Some(rows) => sprite_face(rows, ink),
        None => ssh_face(ink),
    }
}

/// "ssh", in its own case, in the UI font's bold face. The h stands a cell tall, the same as
/// the icons, and the word has no descender, so its ink fills the band exactly.
fn ssh_face(ink: [u8; 3]) -> UndoFace {
    let Some(font) = text::bold_font() else {
        return UndoFace {
            rgba: Vec::new(),
            w: 0,
            h: 0,
        };
    };
    let word = "ssh";
    let w = text::line_width(font, word, SSH_PX, 0.0).ceil() as u32;
    let layout = text::Layout {
        lines: vec![word.to_string()],
        px: SSH_PX,
        tracking: 0.0,
    };
    cropped(tinted(
        &text::coverage_in(font, w, HINT_H, &layout),
        w,
        HINT_H,
        ink,
    ))
}

fn tinted(mask: &[u8], w: u32, h: u32, ink: [u8; 3]) -> UndoFace {
    let rgba = mask
        .iter()
        .flat_map(|&a| [ink[0], ink[1], ink[2], a])
        .collect();
    UndoFace { rgba, w, h }
}

pub type Face = (TexId, u32, u32);

/// Every item's faces, off first then on.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct LinkFaces {
    pub wifi: [Face; 2],
    pub bluetooth: [Face; 2],
    pub ssh: [Face; 2],
}

impl LinkFaces {
    fn of(&self, link: Link, on: bool) -> Face {
        let pair = match link {
            Link::Wifi => self.wifi,
            Link::Bluetooth => self.bluetooth,
            Link::Ssh => self.ssh,
        };
        pair[usize::from(on)]
    }
}

pub struct TopBar<'a> {
    /// What the bar and its rule are drawn in, the housing and the edge, and the ink the
    /// battery's capsule is drawn in. The faces handed in below are baked in the same ink.
    pub colours: ChromeColours,
    pub links: Links,
    pub faces: Option<&'a LinkFaces>,
    pub clock: Printed,
    pub battery: Option<Battery>,
    pub percent: Printed,
    pub bolt: Option<TexId>,
}

/// The top of the band every item stands in, centred in the bar above its rule.
fn band() -> f32 {
    ((TOP_BAR_H - RULE_H - CELL_PX) / 2.0).round()
}

impl TopBar<'_> {
    pub fn draw(&self, out: &mut Vec<Draw>) {
        let w = out_w() as f32;
        out.push(Draw::Rect {
            x: 0.0,
            y: 0.0,
            w,
            h: TOP_BAR_H - RULE_H,
            colour: self.colours.housing,
        });
        out.push(Draw::Rect {
            x: 0.0,
            y: TOP_BAR_H - RULE_H,
            w,
            h: RULE_H,
            colour: self.colours.edge,
        });
        if let Some(faces) = self.faces {
            let mut x = MARGIN;
            for (link, on) in self.shown() {
                let (tex, fw, fh) = faces.of(link, on);
                out.push(Draw::Tex {
                    x,
                    y: band() + ((CELL_PX - fh as f32) / 2.0).round(),
                    w: fw as f32,
                    h: fh as f32,
                    tex,
                    alpha: 1.0,
                });
                x += fw as f32 + GAP;
            }
        }
        if self.clock.w > 0 {
            let x = ((w - self.clock.w as f32 + word_tail()) / 2.0).round();
            draw_printed(x, band() - word_cap_top(), self.clock, out);
        }
        // the figure's ink, not its face, ends at the margin, as the Wi-Fi icon's starts at it
        let x = w - MARGIN - gauge_width(self.percent) + word_tail();
        let ink = self.colours.ink;
        draw_gauge(x, band(), self.battery, self.percent, self.bolt, ink, out);
    }

    /// The items along the left, in order, each with whether it is on.
    pub fn shown(&self) -> Vec<(Link, bool)> {
        let l = self.links;
        let mut items = vec![(Link::Wifi, l.wifi)];
        if let Some(on) = l.bluetooth {
            items.push((Link::Bluetooth, on));
        }
        items.push((Link::Ssh, l.ssh));
        items
    }
}
