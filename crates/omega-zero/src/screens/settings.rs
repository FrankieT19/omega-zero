//! Settings, opened with START on the shelf: the configs left and right change in place, and
//! the rows that open a screen of their own, in two groups.

use omega_input::{Action, Btn};
use omega_store::FF_SPEEDS;
use omega_ui::{quick_window, Draw, QuickMenu, QuickMenuFaces, QuickValue, TexId};

pub use omega_ui::{SettingsGroup, SettingsRow};

use crate::app::{App, Phase};
use crate::background_screen::BackgroundScreen;
use crate::label_screen::LabelsScreen;
use crate::screens::clock::clock_screen;
use crate::wifi_screen::WifiScreen;

impl App {
    /// The lit row, while Settings is open.
    pub fn settings_row(&self) -> Option<SettingsRow> {
        match self.phase {
            Phase::Settings { row } => Some(row),
            _ => None,
        }
    }

    /// What a config row is set to; a row that opens a screen shows none of these.
    pub fn settings_value(&self, row: SettingsRow) -> Option<QuickValue> {
        match row {
            SettingsRow::FastForward => QuickValue::speed(self.state.ff_speed),
            SettingsRow::FastForwardSound => Some(QuickValue::flag(self.state.ff_sound)),
            SettingsRow::ColourCorrection => Some(QuickValue::flag(self.state.colour_correction)),
            SettingsRow::Rumble => Some(QuickValue::flag(self.state.rumble)),
            SettingsRow::Labels
            | SettingsRow::DateTime
            | SettingsRow::Network
            | SettingsRow::Background
            | SettingsRow::About => None,
        }
    }

    pub fn set_settings_faces(&mut self, faces: QuickMenuFaces) {
        self.settings_faces = Some(faces);
    }

    pub fn set_settings_clock_faces(&mut self, dim: (TexId, u32, u32), lit: (TexId, u32, u32)) {
        self.settings_clock_faces = Some([dim, lit]);
    }

    pub fn set_settings_network_faces(&mut self, dim: (TexId, u32, u32), lit: (TexId, u32, u32)) {
        self.settings_network_faces = Some([dim, lit]);
    }

    pub(crate) fn open_settings(&mut self) {
        self.settings_first = 0;
        self.phase = Phase::Settings {
            row: SettingsRow::ALL[0],
        };
    }

    /// The rows Settings lists on this handheld: Network only where there is a radio.
    pub fn settings_rows(&self) -> Vec<SettingsRow> {
        SettingsRow::ALL
            .into_iter()
            .filter(|r| *r != SettingsRow::Network || self.wifi.present())
            .collect()
    }

    pub(crate) fn settings_input(&mut self, row: SettingsRow, action: Action) {
        let rows = self.settings_rows();
        let row = match action {
            Action::GbaDown(Btn::Up) => row.up_in(&rows),
            Action::GbaDown(Btn::Down) => row.down_in(&rows),
            Action::GbaDown(Btn::Left) => return self.change_setting(row, false),
            Action::GbaDown(Btn::Right) => return self.change_setting(row, true),
            Action::GbaDown(Btn::A) => return self.open_settings_row(row),
            Action::GbaDown(Btn::B | Btn::Start) | Action::QuickMenu => {
                self.phase = Phase::Shelf;
                return;
            }
            _ => return,
        };
        self.settings_first = self.settings_first_for(row);
        self.phase = Phase::Settings { row };
    }

    /// The first row to show with `row` lit, moving the list as little as it can.
    fn settings_first_for(&self, row: SettingsRow) -> usize {
        let rows = self.settings_rows();
        let at = rows.iter().position(|r| *r == row).unwrap_or(0);
        quick_window(at, rows.len(), self.settings_first)
    }

    fn open_settings_row(&mut self, row: SettingsRow) {
        let to = match row {
            SettingsRow::Labels => Phase::Labels(LabelsScreen::default()),
            SettingsRow::DateTime => clock_screen(self.utc_secs(), self.state.utc_offset_min, true),
            SettingsRow::Network => Phase::Network(WifiScreen::default()),
            SettingsRow::Background => Phase::Background(BackgroundScreen::default()),
            SettingsRow::About => Phase::About,
            SettingsRow::FastForward
            | SettingsRow::FastForwardSound
            | SettingsRow::ColourCorrection
            | SettingsRow::Rumble => return,
        };
        self.phase = to;
    }

    pub(crate) fn change_setting(&mut self, row: SettingsRow, right: bool) {
        let s = &mut self.state;
        match row {
            SettingsRow::FastForward => {
                let to = ff_next(s.ff_speed, right);
                if to == s.ff_speed {
                    return;
                }
                s.ff_speed = to;
            }
            SettingsRow::FastForwardSound => s.ff_sound = !s.ff_sound,
            SettingsRow::ColourCorrection => {
                s.colour_correction = !s.colour_correction;
                self.colour_pending = Some(s.colour_correction);
            }
            SettingsRow::Rumble => s.rumble = !s.rumble,
            SettingsRow::Labels
            | SettingsRow::DateTime
            | SettingsRow::Network
            | SettingsRow::Background
            | SettingsRow::About => return,
        }
        self.persist();
    }

    pub(crate) fn draw_settings(&self, row: SettingsRow, out: &mut Vec<Draw>) {
        QuickMenu {
            row,
            rows: &self.settings_rows(),
            values: SettingsRow::ALL.map(|r| self.settings_value(r)),
            clock: self.settings_clock_faces,
            network: self.settings_network_faces,
            faces: self.settings_faces.as_ref(),
            first: self.settings_first_for(row),
        }
        .draw(out)
    }
}

fn ff_next(from: u8, right: bool) -> u8 {
    let at = FF_SPEEDS.iter().position(|&v| v == from).unwrap_or(0);
    let to = if right { at + 1 } else { at.saturating_sub(1) };
    FF_SPEEDS[to.min(FF_SPEEDS.len() - 1)]
}
