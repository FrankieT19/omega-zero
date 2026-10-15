//! The SNES cart, in its two shells: the American one, and the Super Famicom's, which Japan and
//! Europe shared. Placeholder until the SNES model lands: both are drawn as a GBA cart.

use omega_store::{Cart, Region};

use crate::cart::Spec;

/// The SNES cart shells a model is built for.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum SnesShell {
    /// The American cart.
    Ntsc,
    /// The Super Famicom's, also Europe's.
    Famicom,
}

/// How thick the cart is, as a share of its width.
pub const THICK: f32 = crate::spin::THICK;

/// How big the cart stands on the shelf, as a share of its own size.
pub const SHELF_SCALE: f32 = 1.0;

/// The shell `cart` comes in: the American cart for a game from the USA, the Super Famicom's
/// for any other.
pub fn shell_of(cart: &Cart) -> SnesShell {
    match omega_store::snes::region(&cart.rom, &cart.stem) {
        Some(Region::Usa) => SnesShell::Ntsc,
        _ => SnesShell::Famicom,
    }
}

/// The cart's outline in drawing units, all the way round clockwise, and the drawing's size.
pub(crate) fn outline(_shell: SnesShell) -> (Vec<(f32, f32)>, (f32, f32)) {
    crate::spin::gba_outline()
}

/// How the cart's face is laid out.
pub(crate) fn spec(_shell: SnesShell) -> Spec {
    crate::cart::gba_spec()
}
