//! The shelf: the carts in a row over the background, the slot under them and the bar across
//! the top.

use omega_ui::{draw_empty_slot, draw_slot_name, ease, Draw, TopBar};

use crate::app::App;

const CORE_PICKER_RECEDE: f32 = 0.26;
const CORE_PICKER_DIM: f32 = 0.614;

impl App {
    pub(crate) fn draw_shelf(&self, out: &mut Vec<Draw>) {
        self.draw_background(out);
        match (self.core_picker_shown(), self.selected_stem()) {
            (Some(picker), Some(stem)) => {
                let open = ease(picker.openness(self.now()));
                let dim = 1.0 + (CORE_PICKER_DIM - 1.0) * open;
                self.shelf()
                    .draw_row(Some(stem), 0.0, CORE_PICKER_RECEDE * open, dim, out);
                draw_empty_slot(out);
            }
            _ => {
                self.shelf().draw(self.shelf_shake(), out);
                draw_slot_name(self.shelf_platform, self.slot_name_alpha(), out);
            }
        }
        self.draw_top_bar(out);
    }

    pub(crate) fn draw_top_bar(&self, out: &mut Vec<Draw>) {
        TopBar {
            links: self.links(),
            faces: self.link_faces.as_ref(),
            clock: self.shelf_clock,
            battery: self.battery,
            percent: self.battery_percent,
            bolt: self.bolt,
        }
        .draw(out);
    }
}
