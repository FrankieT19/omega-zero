//! The Wi-Fi screen, opened from the quick menu: switching Wi-Fi and network time, and the
//! networks in reach.

use omega_input::Action;
use omega_ui::{Draw, QuickRow, WifiMenuFaces};

use crate::app::{App, Phase};
use crate::label_screen::LabelsScreen;
use crate::list_view::ListView;
use crate::wifi_screen::{Outcome as WifiOutcome, Typing};

impl App {
    pub(crate) fn wifi_input(&mut self, action: Action) {
        let now = self.now();
        let Phase::Wifi(screen) = &mut self.phase else {
            return;
        };
        match screen.input(&mut self.wifi, action, now) {
            WifiOutcome::Close => {
                self.phase = Phase::QuickMenu {
                    row: QuickRow::Wifi,
                }
            }
            WifiOutcome::Labels => self.phase = Phase::Labels(LabelsScreen::default()),
            WifiOutcome::Stay => {}
        }
        if self.state.wifi != self.wifi.enabled() {
            self.state.wifi = self.wifi.enabled();
            self.persist();
        }
    }

    /// The Wi-Fi screen's list, while it is open and not taking a passphrase.
    pub fn wifi_view(&mut self) -> Option<ListView> {
        match &mut self.phase {
            Phase::Wifi(screen) if screen.typing.is_none() => Some(screen.view(&self.wifi)),
            _ => None,
        }
    }

    /// The Wi-Fi screen's keyboard, and the network it is for.
    pub fn wifi_typing(&self) -> Option<&Typing> {
        match &self.phase {
            Phase::Wifi(screen) => screen.typing.as_ref(),
            _ => None,
        }
    }

    pub fn set_wifi_faces(&mut self, faces: WifiMenuFaces) {
        self.wifi_faces = Some(faces);
    }

    pub(crate) fn draw_wifi(&self, out: &mut Vec<Draw>) {
        if let Some(faces) = &self.wifi_faces {
            faces.draw(out);
        }
    }
}
