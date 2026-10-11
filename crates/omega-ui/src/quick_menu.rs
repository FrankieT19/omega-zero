//! What is left of the old full-screen menus: their line of text, which the link menus over a
//! game still show. Settings itself is settings_menu.rs.

use crate::plate::UndoFace;
use crate::text;

pub(crate) const MENU_PX: f32 = 18.0;
const MENU_MIN_PX: f32 = 18.0;
pub const MENU_H: u32 = 40;
pub const MENU_PAD: u32 = 18;
pub const MENU_INK: [u8; 3] = [0xf6, 0xf4, 0xef];

/// `label` as one of the old menus' lines: in the label font at `MENU_PX`, cream, centred on a
/// clear face `MENU_H` tall with `MENU_PAD` either side.
pub fn menu_face(label: &str) -> UndoFace {
    let Some(font) = text::label_font() else {
        return UndoFace {
            rgba: Vec::new(),
            w: 0,
            h: 0,
        };
    };
    let ink = text::line_width(font, label, MENU_PX, 0.0).ceil() as u32;
    let w = ink + 2 * MENU_PAD;
    let mut rgba = vec![0u8; (w * MENU_H * 4) as usize];
    let layout = text::fit(font, label, w as f32, 1, MENU_PX, MENU_MIN_PX);
    text::draw_centred(&mut rgba, w, MENU_H, &layout, MENU_INK);
    UndoFace { rgba, w, h: MENU_H }
}
