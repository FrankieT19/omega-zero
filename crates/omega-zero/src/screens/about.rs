//! About, opened from the quick menu: the handheld's label, with the build's serial on it.

use omega_input::{Action, Btn};
use omega_ui::{draw_sticker, Draw, QuickRow, TexId};

use crate::app::{App, Phase};

impl App {
    pub(crate) fn about_input(&mut self, action: Action) {
        if action == Action::GbaDown(Btn::B) || action == Action::QuickMenu {
            self.phase = Phase::QuickMenu {
                row: QuickRow::About,
            }
        }
    }

    pub fn set_sticker_face(&mut self, face: TexId) {
        self.sticker_face = Some(face);
    }

    pub(crate) fn draw_about(&self, out: &mut Vec<Draw>) {
        self.draw_background(out);
        draw_sticker(self.sticker_face, out);
    }
}
