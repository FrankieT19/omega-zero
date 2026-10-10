//! The Background screen, opened from the quick menu: its list, drawn over the background it
//! is choosing.

use omega_input::Action;
use omega_ui::{Draw, QuickRow, WifiMenuFaces};

use crate::app::{App, Phase};
use crate::background_screen::Outcome as BackgroundOutcome;
use crate::list_view::ListView;

impl App {
    pub(crate) fn background_input(&mut self, action: Action) {
        let (look, wallpaper) = (self.backdrop(), self.wallpaper.is_some());
        let Phase::Background(screen) = &mut self.phase else {
            return;
        };
        match screen.input(look, wallpaper, action) {
            BackgroundOutcome::Close => {
                self.phase = Phase::QuickMenu {
                    row: QuickRow::Background,
                }
            }
            BackgroundOutcome::Changed(b) => {
                self.state.backdrop = Some(b.style);
                self.state.backdrop_moving = b.moving;
                self.state.backdrop_colour = b.colour;
                self.persist();
            }
            BackgroundOutcome::Stay => {}
        }
    }

    /// The Background screen's list, while it is open.
    pub fn background_view(&self) -> Option<ListView> {
        match &self.phase {
            Phase::Background(screen) => {
                Some(screen.view(self.backdrop(), self.wallpaper.is_some()))
            }
            _ => None,
        }
    }

    pub fn set_background_faces(&mut self, faces: WifiMenuFaces) {
        self.background_faces = Some(faces);
    }

    pub(crate) fn draw_background_screen(&self, out: &mut Vec<Draw>) {
        self.draw_background(out);
        if let Some(faces) = &self.background_faces {
            faces.draw_over(out);
        }
    }
}
