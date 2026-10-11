//! What the screens' syncs share: the uploads that reuse a texture as what it shows changes.

use omega_gfx::{Compositor, TexId};
use omega_ui::{Printed, UndoFace};

pub(crate) fn upload(
    compositor: &mut Compositor,
    held: &mut Option<TexId>,
    face: omega_ui::UndoFace,
) -> TexId {
    upload_rgba(compositor, held, face.w, face.h, &face.rgba)
}

pub(crate) fn upload_rgba(
    compositor: &mut Compositor,
    held: &mut Option<TexId>,
    w: u32,
    h: u32,
    rgba: &[u8],
) -> TexId {
    match *held {
        Some(id) => {
            compositor.update_texture(id, w, h, rgba);
            id
        }
        None => {
            let id = compositor.create_texture(w, h, rgba);
            *held = Some(id);
            id
        }
    }
}

/// A kit face (one texel to an art pixel, drawn at 2x by `omega_ui::kit::pixel::quad`) into the
/// texture `held`, made on first use with NEAREST filtering so it stays sharp at 2x, and filled
/// in place after that (an updated texture keeps its filter).
pub(crate) fn upload_nearest(
    compositor: &mut Compositor,
    held: &mut Option<TexId>,
    face: &omega_ui::UndoFace,
) -> TexId {
    match *held {
        Some(id) => {
            compositor.update_texture(id, face.w, face.h, &face.rgba);
            id
        }
        None => {
            let id = compositor.create_texture_nearest(face.w, face.h, &face.rgba);
            *held = Some(id);
            id
        }
    }
}

/// A word printed into a texture of its own, and what it was printed from, so its sync prints
/// it again only when that changes: what it says, or the ink it is in.
pub(crate) struct Printing<K> {
    tex: Option<TexId>,
    from: Option<K>,
}

impl<K> Default for Printing<K> {
    fn default() -> Self {
        Printing {
            tex: None,
            from: None,
        }
    }
}

impl<K: PartialEq> Printing<K> {
    /// The word `face` prints from `from`, uploaded into this texture when it was last printed
    /// from something else; None while it still shows `from`.
    pub(crate) fn print(
        &mut self,
        compositor: &mut Compositor,
        from: K,
        face: impl FnOnce(&K) -> UndoFace,
    ) -> Option<Printed> {
        if self.from.as_ref() == Some(&from) {
            return None;
        }
        let face = face(&from);
        let w = face.w;
        let tex = upload(compositor, &mut self.tex, face);
        self.from = Some(from);
        Some(Printed::new(tex, w))
    }
}
