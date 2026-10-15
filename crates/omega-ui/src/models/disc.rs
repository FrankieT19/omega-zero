//! The PlayStation disc, and the stack a game on several discs stands as. Placeholder until
//! the disc model lands: a disc is drawn as a GBA cart, and a stack as its first disc.

use omega_gfx::MeshVertex;
use omega_store::{Cart, Platform};

use crate::cart::{CartFace, Spec};

/// How thick a disc is, as a share of its width.
pub const THICK: f32 = crate::spin::THICK;

/// How big a disc stands on the shelf, as a share of its own size.
pub const SHELF_SCALE: f32 = 1.0;

/// The most discs of one game the shelf draws in its stack.
pub const MAX_STACK: usize = 4;

/// How many discs `cart` stands as on the shelf: one, or each disc of a game on several, up to
/// [`MAX_STACK`].
pub fn stack_of(cart: &Cart) -> usize {
    match cart.platform {
        Platform::Ps1 => omega_store::ps1::discs(&cart.rom).len().clamp(1, MAX_STACK),
        _ => 1,
    }
}

/// The disc's outline in drawing units, all the way round clockwise, and the drawing's size.
pub(crate) fn outline() -> (Vec<(f32, f32)>, (f32, f32)) {
    crate::spin::gba_outline()
}

/// The disc in few polygons.
pub(crate) fn mesh() -> Vec<MeshVertex> {
    let (points, units) = outline();
    crate::spin::extruded(&points, units, THICK)
}

/// How the disc's face is laid out.
pub(crate) fn spec() -> Spec {
    crate::cart::gba_spec()
}

/// The disc's face, when it is not built as a cart's is.
pub(crate) fn face(_cart: &Cart) -> Option<CartFace> {
    None
}
