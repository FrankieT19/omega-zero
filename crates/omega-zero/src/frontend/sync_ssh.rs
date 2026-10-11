//! The SSH panel's faces, on textures of their own so the Network modal it slides against
//! keeps its own: the panel baked for the palette and for how many lines it holds, each line
//! and the note again only when what it says changes.

use omega_gfx::Compositor;
use omega_ui::kit::row::{row_face, row_width, Row};
use omega_ui::kit::text::text_face;
use omega_ui::network::{ssh_chrome, ssh_rect, SshFaces};

use crate::app::App;
use crate::frontend::sync_network::{room_for, Held};
use crate::screens::ssh::SshView;

#[derive(Default)]
pub(crate) struct SshTexes {
    chrome: Held<(u8, usize)>,
    well: [u32; 4],
    lines: Vec<Held<(&'static str, String, u8)>>,
    note: Held<(&'static str, u8)>,
    /// What the faces were last made for.
    shown: Option<(SshView, u8)>,
}

pub(crate) fn sync_ssh(app: &mut App, compositor: &mut Compositor, state: &mut SshTexes) {
    let Some(view) = app.ssh_view() else {
        return;
    };
    let colour = app.palette_colour();
    let now = (view, colour);
    if state.shown.as_ref() == Some(&now) {
        return;
    }
    let (view, colour) = now;
    let pal = app.palette();

    let count = view.lines.len();
    let well = &mut state.well;
    let chrome = state.chrome.face(compositor, (colour, count), |&(_, n)| {
        let panel = ssh_chrome(n, pal);
        *well = panel.well;
        panel.face
    });
    let width = row_width(state.well[2]);
    room_for(&mut state.lines, count);
    let lines = view
        .lines
        .iter()
        .zip(&mut state.lines)
        .map(|(&(label, ref value), held)| {
            held.face(
                compositor,
                (label, value.clone(), colour),
                |(label, value, _)| {
                    let row = Row {
                        label,
                        value: Some(value),
                        ..Row::default()
                    };
                    row_face(width, &row, pal)
                },
            )
        })
        .collect();
    let note = state
        .note
        .face(compositor, (view.note, colour), |(note, _)| {
            text_face(note, pal.dim)
        });

    app.set_ssh_faces(SshFaces {
        chrome,
        at: ssh_rect(count),
        well: state.well,
        lines,
        note: Some(note),
    });
    state.shown = Some((view, colour));
}
