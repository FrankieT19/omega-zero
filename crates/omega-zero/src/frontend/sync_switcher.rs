//! The save-state switcher's photos, made when it opens, and its SAVE STATES panel, baked again
//! when what it says changes: the selection, the saves left, an age as time goes by, or the
//! palette. The panel is as wide as the widest age among the saves, so it holds still as the
//! selection moves. Its line of keys is baked again when its keys or the palette change. It
//! stands in the middle and keeps the room of the widest keys it has had since the switcher
//! opened, so when the undo runs out X UNDO SAVE goes and Y DELETE stays where it was, the well
//! empty after it.

use omega_gfx::{Compositor, TexId};
use omega_store::format_stamp;
use omega_ui::kit::bar::Hint;
use omega_ui::kit::hints::hint_line_face;
use omega_ui::kit::saves::{saves_face, saves_width};
use omega_ui::photo_face;

use crate::app::{App, Phase};
use crate::frontend::faces::upload_nearest;
use crate::screens::switcher::{keys_width, KEYS_TITLE};

#[derive(Default)]
pub(crate) struct Switcher {
    open: bool,
    /// The panel's texture, reused each time the switcher opens.
    panel: Option<TexId>,
    /// What the panel was last baked from, since the switcher opened.
    baked: Option<Panel>,
    /// The line of keys' texture, reused each time the switcher opens.
    keys: Option<TexId>,
    /// The keys, the palette colour and the width the line was last baked with, since the
    /// switcher opened.
    keys_baked: Option<(Vec<Hint<'static>>, u8, u32)>,
}

/// What the panel shows: the age of the save shown, which of how many it is, the panel's width
/// and the palette colour it is drawn in.
#[derive(Clone, PartialEq, Eq, Debug)]
struct Panel {
    age: String,
    count: usize,
    at: usize,
    w: u32,
    colour: u8,
}

pub(crate) fn sync_switcher(
    app: &mut App,
    compositor: &mut Compositor,
    pool: &mut Vec<TexId>,
    state: &mut Switcher,
) {
    if !matches!(app.phase(), Phase::Polaroids { .. }) {
        state.open = false;
        return;
    }
    if !state.open {
        state.open = true;
        state.baked = None;
        state.keys_baked = None;
        let faces: Vec<_> = app.polaroid_entries().iter().map(photo_face).collect();
        let ids = faces
            .iter()
            .enumerate()
            .map(|(i, f)| match pool.get(i) {
                Some(id) => {
                    compositor.update_texture(*id, f.w, f.h, &f.rgba);
                    *id
                }
                None => {
                    let id = compositor.create_texture_nearest(f.w, f.h, &f.rgba);
                    pool.push(id);
                    id
                }
            })
            .collect();
        app.set_polaroid_faces(ids);
    }
    if app.polaroids().is_none() {
        return;
    }
    sync_keys(app, compositor, state);
    sync_panel(app, compositor, state);
}

/// The SAVE STATES panel, baked again when what it says or the palette changes.
fn sync_panel(app: &mut App, compositor: &mut Compositor, state: &mut Switcher) {
    let Some(p) = app.polaroids() else {
        return;
    };
    let ages = p.ages(&format_stamp(app.wall_secs()));
    let (count, at) = p.dots();
    let panel = Panel {
        w: ages
            .iter()
            .map(|a| saves_width(a, count))
            .max()
            .unwrap_or(0),
        age: ages.get(at).cloned().unwrap_or_default(),
        count,
        at,
        colour: app.palette_colour(),
    };
    if state.baked.as_ref() == Some(&panel) {
        return;
    }
    let face = saves_face(panel.w, &panel.age, count, at, app.palette());
    let tex = upload_nearest(compositor, &mut state.panel, &face);
    app.set_polaroid_panel(tex, face.w, face.h);
    state.baked = Some(panel);
}

/// The line of keys, baked again when its keys or the palette change, as wide as the widest
/// keys it has had since the switcher opened.
fn sync_keys(app: &mut App, compositor: &mut Compositor, state: &mut Switcher) {
    let hints = app.switcher_hints();
    let held = state.keys_baked.as_ref().map_or(0, |baked| baked.2);
    let w = keys_width(&hints, held);
    let want = (hints, app.palette_colour(), w);
    if state.keys_baked.as_ref() == Some(&want) {
        return;
    }
    let face = hint_line_face(w, KEYS_TITLE, &want.0, app.palette());
    let tex = upload_nearest(compositor, &mut state.keys, &face);
    app.set_polaroid_keys(tex, face.w, face.h);
    state.keys_baked = Some(want);
}
