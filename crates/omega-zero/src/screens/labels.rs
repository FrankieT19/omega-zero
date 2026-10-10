//! The Labels screen, opened from the Wi-Fi screen: the ScreenScraper account, and a sync that
//! fetches the real labels of the carts that have none.

use omega_input::Action;
use omega_ui::{Draw, WifiMenuFaces};

use crate::app::{App, Phase};
use crate::label_screen::Outcome as LabelsOutcome;
use crate::list_view::ListView;
use crate::wifi_screen::{Sel as WifiSel, WifiScreen};

impl App {
    pub(crate) fn labels_input(&mut self, action: Action) {
        let root = self.root.clone();
        let Phase::Labels(screen) = &mut self.phase else {
            return;
        };
        match screen.input(&mut self.labels, action, root.as_deref()) {
            LabelsOutcome::Close => {
                self.phase = Phase::Wifi(WifiScreen {
                    sel: WifiSel::Labels,
                    ..WifiScreen::default()
                })
            }
            LabelsOutcome::Sync if self.online() => {
                let wants = self.label_wants();
                self.labels.start(wants);
            }
            LabelsOutcome::Sync | LabelsOutcome::Stay => {}
        }
    }

    /// The Labels screen's list, while it is open and not taking an account.
    pub fn labels_view(&self) -> Option<ListView> {
        match &self.phase {
            Phase::Labels(screen) if screen.typing.is_none() => {
                Some(screen.view(&self.labels, self.online()))
            }
            _ => None,
        }
    }

    pub fn set_labels_faces(&mut self, faces: WifiMenuFaces) {
        self.labels_faces = Some(faces);
    }

    pub(crate) fn draw_labels(&self, out: &mut Vec<Draw>) {
        if let Some(faces) = &self.labels_faces {
            faces.draw(out);
        }
    }
}
