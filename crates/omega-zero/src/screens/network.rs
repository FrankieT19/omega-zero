//! Network, opened from Settings: switching Wi-Fi and network time, the way into the SSH
//! screen, and the networks in reach.

use omega_input::Action;
use omega_ui::{Draw, SettingsRow, WifiMenuFaces};

use crate::app::{App, Phase};
use crate::list_view::ListView;
use crate::wifi_screen::{Outcome as WifiOutcome, Typing};

impl App {
    pub(crate) fn network_input(&mut self, action: Action) {
        let now = self.now();
        let Phase::Network(screen) = &mut self.phase else {
            return;
        };
        match screen.input(&mut self.wifi, action, now) {
            WifiOutcome::Close => {
                self.phase = Phase::Settings {
                    row: SettingsRow::Network,
                }
            }
            WifiOutcome::Ssh => self.open_ssh(),
            WifiOutcome::Stay => {}
        }
        if self.state.wifi != self.wifi.enabled() {
            self.state.wifi = self.wifi.enabled();
            self.persist();
        }
    }

    /// The Network screen's list, while it is open and not taking a passphrase.
    pub fn network_view(&mut self) -> Option<ListView> {
        match &mut self.phase {
            Phase::Network(screen) if screen.typing.is_none() => Some(screen.view(&self.wifi)),
            _ => None,
        }
    }

    /// The Network screen's keyboard, and the network it is for.
    pub fn network_typing(&self) -> Option<&Typing> {
        match &self.phase {
            Phase::Network(screen) => screen.typing.as_ref(),
            _ => None,
        }
    }

    pub fn set_network_faces(&mut self, faces: WifiMenuFaces) {
        self.network_faces = Some(faces);
    }

    pub(crate) fn draw_network(&self, out: &mut Vec<Draw>) {
        if let Some(faces) = &self.network_faces {
            faces.draw(out);
        }
    }
}
