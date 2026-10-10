//! The QUICKMENU panel, baked the first time it opens and again only for a new palette: what it
//! says never changes.

use omega_gfx::Compositor;

use crate::app::App;
use crate::frontend::lone::Baked;
use crate::screens::controls::controls_face;

pub(crate) fn sync_controls(app: &mut App, compositor: &mut Compositor, held: &mut Baked<u8>) {
    let Some(view) = app.controls_view() else {
        return;
    };
    let pal = app.palette();
    if let Some(panel) = held.bake(compositor, app.palette_colour(), |_| {
        controls_face(&view, pal)
    }) {
        app.set_controls_panel(panel);
    }
}
