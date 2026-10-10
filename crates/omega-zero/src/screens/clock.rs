//! The clock: asked for at first boot, and opened from the quick menu's Date & Time.

use omega_input::{Action, Btn};
use omega_ui::{ClockPicker, Draw, QuickRow, TexId};

use crate::app::{App, Phase};

impl App {
    pub(crate) fn clock_input(&mut self, action: Action) {
        let Phase::SetClock {
            picker, from_menu, ..
        } = &mut self.phase
        else {
            return;
        };
        match action {
            Action::GbaDown(Btn::Left) | Action::ShelfLeft => picker.left(),
            Action::GbaDown(Btn::Right) | Action::ShelfRight => picker.right(),
            Action::GbaDown(Btn::Up) => picker.up(),
            Action::GbaDown(Btn::Down) => picker.down(),
            Action::GbaDown(Btn::A) | Action::Insert => self.confirm_clock(),
            Action::GbaDown(Btn::B) if *from_menu => {
                self.phase = Phase::QuickMenu {
                    row: QuickRow::DateTime,
                }
            }
            _ => {}
        }
    }

    pub fn confirm_clock(&mut self) {
        let Phase::SetClock {
            picker,
            seed,
            from_menu,
        } = &self.phase
        else {
            return;
        };
        let (moved, offset, from_menu) = (picker.secs() - *seed, picker.offset_min(), *from_menu);
        let utc = self.utc_secs() + moved;
        if let Some(power) = &mut self.power {
            power.set_clock(utc);
        }
        self.state.utc_offset_min = offset as i16;
        self.state.clock_set = true;
        self.persist();
        if from_menu {
            self.phase = Phase::QuickMenu {
                row: QuickRow::DateTime,
            };
        } else {
            self.start();
        }
    }

    pub fn picker(&self) -> Option<&ClockPicker> {
        match &self.phase {
            Phase::SetClock { picker, .. } => Some(picker),
            _ => None,
        }
    }

    pub fn set_clock_faces(&mut self, line: TexId, hint: TexId) {
        self.clock_faces = Some((line, hint));
    }

    pub(crate) fn draw_clock(&self, picker: &ClockPicker, from_menu: bool, out: &mut Vec<Draw>) {
        let (line, hint) = match self.clock_faces {
            Some((line, hint)) => (Some(line), Some(hint)),
            None => (None, None),
        };
        let back = self
            .quick_menu_faces
            .as_ref()
            .filter(|_| from_menu)
            .map(|f| f.legend[0]);
        picker.draw(line, hint, back, out);
    }
}

pub(crate) fn clock_screen(utc: i64, offset_min: i16, from_menu: bool) -> Phase {
    Phase::SetClock {
        picker: ClockPicker::local(utc, offset_min),
        seed: utc - utc.rem_euclid(60),
        from_menu,
    }
}
