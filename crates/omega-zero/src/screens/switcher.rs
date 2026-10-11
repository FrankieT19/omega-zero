//! The save-state switcher's keys, MENU ×2 in a game. B RETURN and A LOAD are the bar's actions
//! (bar.rs's `Actions::Switcher`), with nothing between them; the rest stand on a line of their
//! own just above the bar, the kit's hint line titled SAVE (`omega_ui::kit::hints`), on a ground
//! of its own where the bar's hints would sit on the game's screenshot: Y DELETE, and X UNDO SAVE
//! or X UNDO LOAD while the last save or load can still be undone. The line stands in the middle
//! (`Polaroids::keys_rect`) and keeps the room of its widest keys while the switcher is open.

use omega_ui::kit::bar::Hint;
use omega_ui::kit::hints::hint_line_width;

use crate::app::{App, PendingUndo};

/// The title of the switcher's line of keys.
pub const KEYS_TITLE: &str = "SAVE";

/// Y in the save-state switcher: it deletes the save shown.
pub const DELETE: Hint<'static> = Hint {
    key: "Y",
    word: "DELETE",
};

/// X in the save-state switcher while the last save, or the last load, can still be undone.
pub const UNDO_SAVE: Hint<'static> = Hint {
    key: "X",
    word: "UNDO SAVE",
};
pub const UNDO_LOAD: Hint<'static> = Hint {
    key: "X",
    word: "UNDO LOAD",
};

/// How wide the switcher's line of keys is with `hints` on it, having been `held` wide since the
/// switcher opened (0 as it opens): as wide as they need, or as it has been if that is wider. It
/// stands in the middle, so this keeps Y DELETE still when X UNDO SAVE goes.
pub fn keys_width(hints: &[Hint], held: u32) -> u32 {
    hint_line_width(KEYS_TITLE, hints).max(held)
}

impl App {
    /// The keys on the switcher's line: Y DELETE, then X UNDO SAVE or X UNDO LOAD while the last
    /// save or load can still be undone.
    pub fn switcher_hints(&self) -> Vec<Hint<'static>> {
        [DELETE].into_iter().chain(self.undo_hint()).collect()
    }

    /// X's key on the switcher's line while the last save or load can still be undone.
    fn undo_hint(&self) -> Option<Hint<'static>> {
        if !self.undo_available(self.now()) {
            return None;
        }
        match self.pending.as_ref()?.0 {
            PendingUndo::Save { .. } => Some(UNDO_SAVE),
            PendingUndo::Load { .. } => Some(UNDO_LOAD),
        }
    }
}
