//! The shelf: the carts in a row over the background, the slot under them and the bar across
//! the top. Each is a layer of its own as the shelf slides away to a menu and back.

use omega_ui::{draw_empty_slot, draw_slot_name, ease, Draw, TopBar};

use crate::app::App;
use crate::nav::Offsets;
use crate::screens::shift_all;

const CORE_PICKER_RECEDE: f32 = 0.26;
const CORE_PICKER_DIM: f32 = 0.614;

impl App {
    /// The shelf, its carts moved `off.carts_dx` across, its top bar `off.bar_dy` and its
    /// slot `off.slot_dy` down.
    pub(crate) fn draw_shelf(&self, off: Offsets, out: &mut Vec<Draw>) {
        let carts = out.len();
        match (self.core_picker_shown(), self.selected_stem()) {
            (Some(picker), Some(stem)) => {
                let open = ease(picker.openness(self.now()));
                let dim = 1.0 + (CORE_PICKER_DIM - 1.0) * open;
                self.shelf()
                    .draw_row(Some(stem), 0.0, CORE_PICKER_RECEDE * open, dim, out);
                shift_all(&mut out[carts..], off.carts_dx, 0.0);
                let slot = out.len();
                draw_empty_slot(out);
                shift_all(&mut out[slot..], 0.0, off.slot_dy);
            }
            _ => {
                self.shelf()
                    .draw_row(None, self.shelf_shake(), 0.0, 1.0, out);
                shift_all(&mut out[carts..], off.carts_dx, 0.0);
                let slot = out.len();
                draw_empty_slot(out);
                draw_slot_name(self.shelf_platform, self.slot_name_alpha(), out);
                shift_all(&mut out[slot..], 0.0, off.slot_dy);
            }
        }
        let bar = out.len();
        self.draw_top_bar(out);
        shift_all(&mut out[bar..], 0.0, off.bar_dy);
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
