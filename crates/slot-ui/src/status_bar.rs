//! The bar across the top of the shelf: what the handheld is connected to on the left, the
//! time in the middle and the battery on the right.

use slot_gfx::{out_w, Draw, TexId};
use slot_power::Battery;

use crate::battery::{draw_gauge, gauge_width, GAUGE_H};
use crate::footer::{draw_printed, Printed};
use crate::hud::HUD_INK;
use crate::icon::symbols_font;
use crate::plate::{UndoFace, HINT_H};
use crate::slot_chrome::{edge, housing};
use crate::text;

pub const TOP_BAR_H: f32 = 36.0;
const MARGIN: f32 = 24.0;
const ICON_PX: f32 = 20.0;
const ICON_GAP: f32 = 10.0;
const SSH_GAP: f32 = 12.0;
const SSH_PX: f32 = 18.0;
const RULE_H: f32 = 2.0;

/// What an item is drawn in while it is on, and while it is off.
pub const LINK_ON: [u8; 3] = HUD_INK;
pub const LINK_OFF: [u8; 3] = [0x5e, 0x5d, 0x66];

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Link {
    Wifi,
    Bluetooth,
    Ssh,
}

impl Link {
    pub const ALL: [Link; 3] = [Link::Wifi, Link::Bluetooth, Link::Ssh];

    /// The Symbols Nerd Font glyph for an icon, on and off: off is crossed out.
    pub fn glyph(self, on: bool) -> Option<char> {
        match (self, on) {
            (Link::Wifi, true) => Some('\u{f05a9}'),
            (Link::Wifi, false) => Some('\u{f05aa}'),
            (Link::Bluetooth, true) => Some('\u{f00af}'),
            (Link::Bluetooth, false) => Some('\u{f00b2}'),
            (Link::Ssh, _) => None,
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

/// One item's face, `HINT_H` tall: an icon, or "ssh" in bold.
pub fn link_face(link: Link, on: bool) -> UndoFace {
    let ink = if on { LINK_ON } else { LINK_OFF };
    match link.glyph(on) {
        Some(glyph) => icon(glyph, ink),
        None => ssh_face(ink),
    }
}

fn blank() -> UndoFace {
    UndoFace {
        rgba: Vec::new(),
        w: 0,
        h: 0,
    }
}

/// An icon centred in one box shared by every icon the bar draws, so swapping on for off
/// never moves anything beside it.
fn icon(glyph: char, ink: [u8; 3]) -> UndoFace {
    let Some(font) = symbols_font() else {
        return blank();
    };
    let glyphs = [Link::Wifi, Link::Bluetooth]
        .into_iter()
        .flat_map(|l| [l.glyph(true), l.glyph(false)])
        .flatten();
    let w = glyphs
        .map(|g| font.metrics(g, ICON_PX).width as u32)
        .max()
        .unwrap_or(0);
    let h = HINT_H;
    let (m, cov) = font.rasterize(glyph, ICON_PX);
    let mut mask = vec![0u8; (w * h) as usize];
    let (x0, y0) = (
        (w as i32 - m.width as i32) / 2,
        (h as i32 - m.height as i32) / 2,
    );
    for gy in 0..m.height {
        for gx in 0..m.width {
            let (x, y) = (x0 + gx as i32, y0 + gy as i32);
            if x >= 0 && y >= 0 && (x as u32) < w && (y as u32) < h {
                mask[(y as u32 * w + x as u32) as usize] = cov[gy * m.width + gx];
            }
        }
    }
    tinted(&mask, w, h, ink)
}

/// "ssh", in its own case and in bold: the label face drawn twice a pixel apart, until the
/// UI font has a bold face of its own.
fn ssh_face(ink: [u8; 3]) -> UndoFace {
    let Some(font) = text::label_font() else {
        return blank();
    };
    let word = "ssh";
    let w = text::line_width(font, word, SSH_PX, 0.0).ceil() as u32 + 1;
    let layout = text::Layout {
        lines: vec![word.to_string()],
        px: SSH_PX,
        tracking: 0.0,
    };
    let thin = text::coverage(w, HINT_H, &layout);
    tinted(&embolden(&thin, w), w, HINT_H, ink)
}

/// Widens every stroke of a mask `w` wide by a pixel to the right.
fn embolden(mask: &[u8], w: u32) -> Vec<u8> {
    mask.iter()
        .enumerate()
        .map(|(i, &a)| {
            let left = if (i as u32).is_multiple_of(w) {
                0
            } else {
                mask[i - 1]
            };
            a.max(left)
        })
        .collect()
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
    pub links: Links,
    pub faces: Option<&'a LinkFaces>,
    pub clock: Printed,
    pub battery: Option<Battery>,
    pub percent: Printed,
    pub bolt: Option<TexId>,
}

fn middle(h: f32) -> f32 {
    ((TOP_BAR_H - RULE_H - h) / 2.0).round()
}

impl TopBar<'_> {
    pub fn draw(&self, out: &mut Vec<Draw>) {
        let w = out_w() as f32;
        out.push(Draw::Rect {
            x: 0.0,
            y: 0.0,
            w,
            h: TOP_BAR_H - RULE_H,
            colour: housing(),
        });
        out.push(Draw::Rect {
            x: 0.0,
            y: TOP_BAR_H - RULE_H,
            w,
            h: RULE_H,
            colour: edge(),
        });
        if let Some(faces) = self.faces {
            let mut x = MARGIN;
            for (link, on) in self.shown() {
                let (tex, fw, fh) = faces.of(link, on);
                if link == Link::Ssh {
                    x += SSH_GAP - ICON_GAP;
                }
                out.push(Draw::Tex {
                    x,
                    y: middle(fh as f32),
                    w: fw as f32,
                    h: fh as f32,
                    tex,
                    alpha: 1.0,
                });
                x += fw as f32 + ICON_GAP;
            }
        }
        if self.clock.w > 0 {
            let x = ((w - self.clock.w as f32) / 2.0).round();
            draw_printed(x, middle(HINT_H as f32), self.clock, out);
        }
        let x = w - MARGIN - gauge_width(self.percent);
        draw_gauge(
            x,
            middle(GAUGE_H),
            self.battery,
            self.percent,
            self.bolt,
            out,
        );
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
