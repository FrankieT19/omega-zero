//! The Labels screen, opened from Settings: the ScreenScraper account, and a sync that fetches
//! the real labels of the carts that have none.

use omega_input::Action;
use omega_ui::{Draw, SettingsRow, WifiMenuFaces};

use crate::app::{App, Phase};
use crate::label_screen::Outcome as LabelsOutcome;
use crate::list_view::ListView;
use crate::nav::Dir;

impl App {
    pub(crate) fn labels_input(&mut self, action: Action) {
        let root = self.root.clone();
        let Phase::Labels(screen) = &mut self.phase else {
            return;
        };
        let before = screen.clone();
        let outcome = screen.input(&mut self.labels, action, root.as_deref());
        // the keyboard slides in over the list and away again; from the user name to the
        // password it stays where it is
        match (before.typing.is_some(), screen.typing.is_some()) {
            (false, true) => self.slide_from(Phase::Labels(before), Dir::Deeper),
            (true, false) => self.slide_from(Phase::Labels(before), Dir::Back),
            _ => {}
        }
        match outcome {
            LabelsOutcome::Close => self.go(
                Phase::Settings {
                    row: SettingsRow::Labels,
                },
                Dir::Back,
            ),
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
            faces.draw_over(out);
        }
    }
}
