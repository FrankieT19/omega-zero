//! The opening's lines, each made again when what it says changes: a key typed, a count run
//! up. Once the opening is over their textures shrink to a pixel and give their memory back.

use omega_gfx::{Compositor, TexId};
use omega_ui::opening_line;

use crate::app::App;
use crate::frontend::faces::upload_nearest;

#[derive(Default)]
pub(crate) struct OpeningTexes {
    /// Each row's texture and what it says.
    rows: Vec<(Option<TexId>, String)>,
}

pub(crate) fn sync_opening(app: &mut App, compositor: &mut Compositor, state: &mut OpeningTexes) {
    if !app.opening_shown() {
        for (tex, _) in state.rows.drain(..) {
            if let Some(tex) = tex {
                compositor.update_texture(tex, 1, 1, &[0; 4]);
            }
        }
        return;
    }
    for (row, text) in app.opening_lines() {
        if state.rows.len() <= row {
            state.rows.resize_with(row + 1, Default::default);
        }
        let (tex, said) = &mut state.rows[row];
        if *said == text {
            continue;
        }
        let face = opening_line(&text);
        let id = upload_nearest(compositor, tex, &face);
        *said = text;
        app.set_opening_face(row, id, face.w, face.h);
    }
}
