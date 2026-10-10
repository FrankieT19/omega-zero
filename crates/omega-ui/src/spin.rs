use omega_gfx::{out_h, spun_x, Draw, TexId};

/// How thick a cart is, as a share of its width: a Game Pak is about 8 mm thick and 57 wide.
pub const THICK: f32 = 0.14;

/// How dark a cart's side is against its face, from the back of the side to the front.
const SIDE_SHADE: [f32; 2] = [0.34, 0.52];

/// The side is built from slices of the face at most this far apart across the canvas, close
/// enough that the steps between them do not show.
const SLICE_PX: f32 = 0.75;
const MAX_SLICES: usize = 16;

/// A side narrower than this is not drawn at all.
const MIN_SIDE_PX: f32 = 0.5;

/// How far in from the edge of its box a cart's outline comes, as a share of its width: past
/// that, a slice of the face is always hidden behind the face, so it is not drawn there.
pub const OUTLINE_INSET: f32 = 0.13;

/// How a cart is held: turned `yaw` radians about its upright axis, and lifted `lift` canvas
/// pixels off where it would stand.
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct Pose {
    pub yaw: f32,
    pub lift: f32,
}

impl Pose {
    pub const REST: Pose = Pose {
        yaw: 0.0,
        lift: 0.0,
    };

    /// This pose `k` of the way back to rest.
    pub fn toward_rest(self, k: f32) -> Pose {
        let keep = 1.0 - k.clamp(0.0, 1.0);
        Pose {
            yaw: self.yaw * keep,
            lift: self.lift * keep,
        }
    }
}

/// A cart face at `x, y, w, h`, turned `turn` radians in the canvas and spun `yaw` about its
/// upright axis. An unspun face is a plain `Tex` or `Turned`, exactly as before carts turned.
pub fn posed_face(
    (x, y, w, h): (f32, f32, f32, f32),
    tex: TexId,
    alpha: f32,
    turn: f32,
    yaw: f32,
) -> Draw {
    match (yaw == 0.0, turn == 0.0) {
        (true, true) => Draw::Tex {
            x,
            y,
            w,
            h,
            tex,
            alpha,
        },
        (true, false) => Draw::Turned {
            x,
            y,
            w,
            h,
            tex,
            alpha,
            turn,
        },
        (false, _) => Draw::Spun {
            x,
            y,
            w,
            h,
            tex,
            colour: [1.0, 1.0, 1.0, alpha],
            turn,
            yaw,
            depth: 0.0,
            clip: None,
        },
    }
}

/// The side of a cart whose face is drawn by `posed_face` with the same rect, turn and yaw: slices of
/// the face, darkened by `shade` and drawn `alpha`, stepping back to the cart's thickness on
/// the side that turns towards the eye. They go out before the face, which covers all but the
/// side. A cart turned in the canvas draws every slice whole, since its side is not upright.
pub fn push_cart_side(
    out: &mut Vec<Draw>,
    (x, y, w, h): (f32, f32, f32, f32),
    tex: TexId,
    turn: f32,
    yaw: f32,
    shade: f32,
    alpha: f32,
) {
    let thick = THICK * w;
    let side = thick * yaw.sin().abs();
    if side < MIN_SIDE_PX || alpha <= 0.0 {
        return;
    }
    let slices = ((side / SLICE_PX).ceil() as usize).clamp(1, MAX_SLICES);
    let clip = (turn == 0.0).then(|| side_band(x, w, yaw, thick));
    for k in (1..=slices).rev() {
        let f = k as f32 / slices as f32;
        let dark = shade * (SIDE_SHADE[1] + (SIDE_SHADE[0] - SIDE_SHADE[1]) * f);
        out.push(Draw::Spun {
            x,
            y,
            w,
            h,
            tex,
            colour: [dark, dark, dark, alpha],
            turn,
            yaw,
            depth: thick * f,
            clip,
        });
    }
}

/// The upright band of the canvas where the side of a cart `w` wide at `x`, spun `yaw`, can
/// show: from as far in as its outline comes on the side nearer the eye, out to where the
/// back of the cart reaches.
fn side_band(x: f32, w: f32, yaw: f32, thick: f32) -> [f32; 4] {
    let half = w / 2.0;
    let near = if yaw.sin() > 0.0 { -half } else { half };
    let edge = x + half + spun_x(near, yaw, 0.0);
    let back = x + half + spun_x(near, yaw, thick);
    let inset = OUTLINE_INSET * w;
    let (left, right) = match near < 0.0 {
        true => (back.min(edge) - 1.0, edge + inset + 1.0),
        false => (edge - inset - 1.0, back.max(edge) + 1.0),
    };
    [left, 0.0, right - left, out_h() as f32]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cart::{CART_H, CART_W, GB_CART_H, GB_CART_W};
    use crate::silhouette::{cart_mask, gb_cart_mask, GbShell};

    /// The furthest any row of `mask` keeps in from either side of its box.
    fn widest_inset(mask: &[u8], w: usize, h: usize) -> usize {
        (0..h)
            .filter_map(|y| {
                let row = &mask[y * w..(y + 1) * w];
                let first = row.iter().position(|&c| c > 0)?;
                let last = row.iter().rposition(|&c| c > 0)?;
                Some(first.max(w - 1 - last))
            })
            .max()
            .unwrap_or(0)
    }

    #[test]
    fn every_cart_outline_stays_within_the_band_its_side_is_drawn_in() {
        for (name, mask, w, h) in [
            ("GBA", cart_mask(), CART_W, CART_H),
            ("GB", gb_cart_mask(GbShell::Notched), GB_CART_W, GB_CART_H),
            ("GBC", gb_cart_mask(GbShell::Rounded), GB_CART_W, GB_CART_H),
        ] {
            let inset = widest_inset(mask, w as usize, h as usize) as f32;
            assert!(
                inset <= OUTLINE_INSET * w as f32,
                "the {name} outline comes {inset} px in, past the {} px its side is drawn in",
                OUTLINE_INSET * w as f32
            );
        }
    }
}
