//! The on-screen keyboard, which the Network screen raises for a passphrase and the Labels
//! screen for an account.

use omega_ui::{Draw, KeyboardFaces, KeyboardScreen, TexId};

use crate::app::{App, Phase};
use crate::list_view::TypingView;
use crate::wifi::shown;

impl App {
    /// The keyboard in use, and what it is asking for.
    pub fn typing(&self) -> Option<TypingView<'_>> {
        match &self.phase {
            Phase::Network(screen) => screen.typing.as_ref().map(|t| TypingView {
                prompt: if t.refused {
                    "A passphrase has 8 to 63 characters".to_string()
                } else {
                    format!("Passphrase for {}", shown(&t.ssid))
                },
                keys: &t.keys,
                refused: t.refused,
            }),
            Phase::Labels(screen) => screen.typing.as_ref().map(|t| TypingView {
                prompt: t.prompt(),
                keys: &t.keys,
                refused: false,
            }),
            _ => None,
        }
    }

    pub fn set_keyboard_faces(&mut self, faces: KeyboardFaces) {
        self.keyboard_faces = Some(faces);
    }

    pub fn keyboard_faces_ready(&self) -> bool {
        self.keyboard_faces.is_some()
    }

    pub fn set_typing_faces(&mut self, prompt: (TexId, u32, u32), field: (TexId, u32, u32)) {
        self.typing_faces = Some([prompt, field]);
    }

    pub(crate) fn draw_keyboard(&self, t: &TypingView, out: &mut Vec<Draw>) {
        KeyboardScreen {
            keys: t.keys,
            faces: self.keyboard_faces.as_ref(),
            prompt: self.typing_faces.map(|f| f[0]),
            field: self.typing_faces.map(|f| f[1]),
            refused: t.refused,
        }
        .draw(out)
    }
}
