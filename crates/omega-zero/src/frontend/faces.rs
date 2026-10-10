//! What the screens' syncs share: the textures of a list screen, the faces made from its
//! view, and the upload that reuses a texture as what it shows changes.

use omega_gfx::{out_w, Compositor, TexId};
use omega_ui::{
    arrows_hint_face, hint_face, network_name_face, plain_face, quick_caret_face, quick_value_face,
    signal_face, wifi_status_face, UndoFace, WifiMenuFaces, WifiRowFaces,
};

use crate::list_view::{LegendHint, ListView, RowLabel, RowValue};

/// Textures for a list screen, Wi-Fi's or Labels', and for the keyboard, each reused as what it
/// shows changes.
#[derive(Default)]
pub(crate) struct WifiTexes {
    pub(crate) view: Option<ListView>,
    pub(crate) rows: Vec<[Option<TexId>; 3]>,
    pub(crate) status: Vec<Option<TexId>>,
    pub(crate) legend: Vec<(TexId, u32)>,
    pub(crate) carets: Option<[(TexId, u32, u32); 2]>,
    pub(crate) typed: Option<(String, String, bool)>,
    pub(crate) prompt: Option<TexId>,
    pub(crate) field: Option<TexId>,
}

pub(crate) fn wifi_faces(
    view: &ListView,
    compositor: &mut Compositor,
    state: &mut WifiTexes,
) -> WifiMenuFaces {
    if state.legend.is_empty() {
        state.legend = LegendHint::ALL
            .iter()
            .map(|h| {
                let f = match h {
                    LegendHint::Back => hint_face("B", "Back"),
                    LegendHint::Change => arrows_hint_face("Change"),
                    LegendHint::Join => hint_face("A", "Join"),
                    LegendHint::Forget => hint_face("Y", "Forget"),
                    LegendHint::SignIn => hint_face("A", "Sign in"),
                    LegendHint::SignOut => hint_face("Y", "Sign out"),
                    LegendHint::Start => hint_face("A", "Start"),
                    LegendHint::Stop => hint_face("A", "Stop"),
                    LegendHint::Open => hint_face("A", "Open"),
                };
                (compositor.create_texture(f.w, f.h, &f.rgba), f.w)
            })
            .collect();
    }
    let carets = *state.carets.get_or_insert_with(|| {
        [false, true].map(|right| {
            let f = quick_caret_face(right);
            (compositor.create_texture(f.w, f.h, &f.rgba), f.w, f.h)
        })
    });
    if state.rows.len() < view.rows.len() {
        state.rows.resize(view.rows.len(), [None; 3]);
    }
    let rows = view
        .rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let lit = view.lit == Some(i);
            let held = &mut state.rows[i];
            let label = match &row.label {
                RowLabel::Setting(name) => quick_value_face(name, true),
                RowLabel::Name(name) => network_name_face(name, true),
            };
            let label = sized(compositor, &mut held[0], label);
            let value = row.value.as_ref().map(|v| {
                let face = match v {
                    RowValue::Setting(true) => quick_value_face("On", lit),
                    RowValue::Setting(false) => quick_value_face("Off", lit),
                    RowValue::Said(text) | RowValue::Choice(text) => quick_value_face(text, lit),
                    RowValue::Name(name) => {
                        plain_face(name, out_w() as f32 * 0.45, 27.0, omega_ui::MENU_H, lit)
                    }
                };
                sized(compositor, &mut held[1], face)
            });
            let icon = row.signal.map(|(bars, locked)| {
                sized(compositor, &mut held[2], signal_face(bars, locked, lit))
            });
            WifiRowFaces {
                label,
                value,
                icon,
                setting: matches!(row.value, Some(RowValue::Setting(_) | RowValue::Choice(_))),
            }
        })
        .collect();
    if state.status.len() < view.status.len() {
        state.status.resize(view.status.len(), None);
    }
    let status = view
        .status
        .iter()
        .enumerate()
        .map(|(i, line)| sized(compositor, &mut state.status[i], wifi_status_face(line)))
        .collect();
    WifiMenuFaces {
        rows,
        lit: view.lit,
        status,
        legend: view
            .legend
            .iter()
            .map(|h| state.legend[h.index()])
            .collect(),
        carets,
    }
}

pub(crate) fn sized(
    compositor: &mut Compositor,
    held: &mut Option<TexId>,
    face: UndoFace,
) -> (TexId, u32, u32) {
    let (w, h) = (face.w, face.h);
    (upload(compositor, held, face), w, h)
}

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
