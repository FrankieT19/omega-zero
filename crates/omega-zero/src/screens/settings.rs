//! The quick menu, opened with START on the shelf: the settings left and right change in place,
//! and the rows that open a screen of their own.

use omega_input::{Action, Btn};
use omega_store::FF_SPEEDS;
use omega_ui::{quick_window, Draw, QuickMenu, QuickMenuFaces, QuickRow, QuickValue, TexId};

use crate::app::{App, Phase};
use crate::background_screen::BackgroundScreen;
use crate::screens::clock::clock_screen;
use crate::wifi_screen::WifiScreen;

impl App {
    pub fn quick_menu(&self) -> Option<QuickRow> {
        match self.phase {
            Phase::QuickMenu { row } => Some(row),
            _ => None,
        }
    }

    pub fn quick_value(&self, row: QuickRow) -> Option<QuickValue> {
        match row {
            QuickRow::FastForward => QuickValue::speed(self.state.ff_speed),
            QuickRow::FastForwardSound => Some(QuickValue::flag(self.state.ff_sound)),
            QuickRow::ColourCorrection => Some(QuickValue::flag(self.state.colour_correction)),
            QuickRow::Rumble => Some(QuickValue::flag(self.state.rumble)),
            QuickRow::Background | QuickRow::Wifi | QuickRow::DateTime | QuickRow::About => None,
        }
    }

    pub fn set_quick_menu_faces(&mut self, faces: QuickMenuFaces) {
        self.quick_menu_faces = Some(faces);
    }

    pub fn set_quick_clock_faces(&mut self, dim: (TexId, u32, u32), lit: (TexId, u32, u32)) {
        self.quick_clock_faces = Some([dim, lit]);
    }

    pub fn set_quick_wifi_faces(&mut self, dim: (TexId, u32, u32), lit: (TexId, u32, u32)) {
        self.quick_wifi_faces = Some([dim, lit]);
    }

    pub(crate) fn open_quick_menu(&mut self) {
        self.quick_first = 0;
        self.phase = Phase::QuickMenu {
            row: QuickRow::ALL[0],
        };
    }

    /// The quick menu's rows on this handheld: Wi-Fi only where there is a radio.
    pub fn quick_rows(&self) -> Vec<QuickRow> {
        QuickRow::ALL
            .into_iter()
            .filter(|r| *r != QuickRow::Wifi || self.wifi.present())
            .collect()
    }

    pub(crate) fn quick_menu_input(&mut self, row: QuickRow, action: Action) {
        let rows = self.quick_rows();
        let row = match action {
            Action::GbaDown(Btn::Up) => row.up_in(&rows),
            Action::GbaDown(Btn::Down) => row.down_in(&rows),
            Action::GbaDown(Btn::Left) => return self.change_setting(row, false),
            Action::GbaDown(Btn::Right) => return self.change_setting(row, true),
            Action::GbaDown(Btn::A) => return self.open_quick_row(row),
            Action::GbaDown(Btn::B | Btn::Start) | Action::QuickMenu => {
                self.phase = Phase::Shelf;
                return;
            }
            _ => return,
        };
        self.quick_first = self.quick_first_for(row);
        self.phase = Phase::QuickMenu { row };
    }

    /// The first quick menu row to show with `row` lit, moving the list as little as it can.
    fn quick_first_for(&self, row: QuickRow) -> usize {
        let rows = self.quick_rows();
        let at = rows.iter().position(|r| *r == row).unwrap_or(0);
        quick_window(at, rows.len(), self.quick_first)
    }

    fn open_quick_row(&mut self, row: QuickRow) {
        match row {
            QuickRow::DateTime => {
                self.phase = clock_screen(self.utc_secs(), self.state.utc_offset_min, true);
            }
            QuickRow::About => self.phase = Phase::About,
            QuickRow::Wifi => self.phase = Phase::Wifi(WifiScreen::default()),
            QuickRow::Background => self.phase = Phase::Background(BackgroundScreen::default()),
            QuickRow::FastForward
            | QuickRow::FastForwardSound
            | QuickRow::ColourCorrection
            | QuickRow::Rumble => {}
        }
    }

    pub(crate) fn change_setting(&mut self, row: QuickRow, right: bool) {
        let s = &mut self.state;
        match row {
            QuickRow::FastForward => {
                let to = ff_next(s.ff_speed, right);
                if to == s.ff_speed {
                    return;
                }
                s.ff_speed = to;
            }
            QuickRow::FastForwardSound => s.ff_sound = !s.ff_sound,
            QuickRow::ColourCorrection => {
                s.colour_correction = !s.colour_correction;
                self.colour_pending = Some(s.colour_correction);
            }
            QuickRow::Rumble => s.rumble = !s.rumble,
            QuickRow::Background | QuickRow::Wifi | QuickRow::DateTime | QuickRow::About => return,
        }
        self.persist();
    }

    pub(crate) fn draw_quick_menu(&self, row: QuickRow, out: &mut Vec<Draw>) {
        QuickMenu {
            row,
            rows: &self.quick_rows(),
            values: QuickRow::ALL.map(|r| self.quick_value(r)),
            clock: self.quick_clock_faces,
            wifi: self.quick_wifi_faces,
            faces: self.quick_menu_faces.as_ref(),
            first: self.quick_first_for(row),
        }
        .draw(out)
    }
}

fn ff_next(from: u8, right: bool) -> u8 {
    let at = FF_SPEEDS.iter().position(|&v| v == from).unwrap_or(0);
    let to = if right { at + 1 } else { at.saturating_sub(1) };
    FF_SPEEDS[to.min(FF_SPEEDS.len() - 1)]
}
