//! The bar's faces: both pairs of actions, made again when the palette or the font changes, and
//! the plate between each pair, made again when the hints on it change or the actions are made
//! again, since it fills the room they leave. Each is small and made only then, so a slide that
//! swaps the actions uploads nothing once both plates are made.

use omega_gfx::{out_w, Compositor, TexId};
use omega_ui::kit::action::{action_pair, Font, Side};
use omega_ui::kit::bar::{bar_layout, plate_face, Hint};
use omega_ui::SPRITE_PX;

use crate::app::App;
use crate::frontend::faces::upload_nearest;
use crate::screens::bar::{Actions, BarSet};

/// The bar's textures, each reused as what it shows changes, and what each was made for.
#[derive(Default)]
pub(crate) struct BarTexes {
    /// Each pair of actions' faces, left then right.
    actions: [[Option<TexId>; 2]; 3],
    /// The palette colour and the font the actions were made in.
    actions_built: Option<(u8, Font)>,
    plates: [Option<TexId>; 3],
    /// The palette colour and the hints each plate was made with.
    plates_built: [Option<(u8, Vec<Hint<'static>>)>; 3],
}

pub(crate) fn sync_bar(app: &mut App, compositor: &mut Compositor, texes: &mut BarTexes) {
    let (colour, font) = (app.palette_colour(), app.font());
    let pal = app.palette();
    if texes.actions_built != Some((colour, font)) {
        let width = out_w() / SPRITE_PX;
        let sets = Actions::ALL.map(|actions| {
            let [left, right] = action_pair(
                actions.action(Side::Left),
                actions.action(Side::Right),
                font,
                pal,
            );
            let held = &mut texes.actions[actions.index()];
            let mut up = |i: usize, face: &omega_ui::UndoFace| {
                let tex = upload_nearest(compositor, &mut held[i], face);
                (tex, face.w, face.h)
            };
            BarSet {
                layout: bar_layout(width, &left, &right),
                left: up(0, &left.face),
                right: up(1, &right.face),
            }
        });
        app.set_bar_sets(sets);
        texes.actions_built = Some((colour, font));
        texes.plates_built = Default::default();
    }
    let Some(place) = app.bar_place() else {
        return;
    };
    let at = place.actions.index();
    let want = (colour, app.bar_hints_shown());
    let made = app.bar_faces().is_some_and(|f| f.plates[at].is_some());
    if made && texes.plates_built[at].as_ref() == Some(&want) {
        return;
    }
    let Some(set) = app.bar_faces().map(|f| f.sets[at]) else {
        return;
    };
    let face = plate_face(set.layout.plate[2], &want.1, pal);
    let tex = upload_nearest(compositor, &mut texes.plates[at], &face);
    app.set_bar_plate(place.actions, tex);
    texes.plates_built[at] = Some(want);
}
