//! The Controls screen's faces, made the first time it opens: what it says never changes.

use omega_gfx::Compositor;
use omega_ui::{hint_face, plain_face, quick_value_face, UndoFace, MENU_PAD};

use crate::app::App;
use crate::screens::controls::{
    column_room, column_x, ControlsColumn, ControlsFaces, LINE_H, LINE_PX,
};

pub(crate) fn sync_controls(app: &mut App, compositor: &mut Compositor) {
    if app.controls_faces_ready() {
        return;
    }
    let Some(view) = app.controls_view() else {
        return;
    };
    let mut up = |f: UndoFace| (compositor.create_texture(f.w, f.h, &f.rgba), f.w, f.h);
    let title = up(quick_value_face(view.title, true));
    let pad = 2.0 * MENU_PAD as f32;
    let mut columns: Vec<ControlsColumn> = Vec::new();
    for group in &view.groups {
        // each column starts after the widest line of the one before, and may run to the
        // margin: a line too long for that ends in an ellipsis
        let before = columns.last().map(|c| {
            let ink = c.lines.iter().map(|l| l.1 as f32 - pad).fold(0.0, f32::max);
            (c.x, ink)
        });
        let x = column_x(before);
        let room = column_room(x) + pad;
        let lines = group
            .lines
            .iter()
            .map(|(keys, does)| {
                up(plain_face(
                    &format!("{keys}  {does}"),
                    room,
                    LINE_PX,
                    LINE_H,
                    true,
                ))
            })
            .collect();
        columns.push(ControlsColumn {
            x,
            heading: up(quick_value_face(group.name, false)),
            lines,
        });
    }
    let back = hint_face("B", "Back");
    let back = (
        compositor.create_texture(back.w, back.h, &back.rgba),
        back.w,
    );
    app.set_controls_faces(ControlsFaces {
        title,
        columns,
        back,
    });
}
