use omega_gfx::{out_w, Draw, TexId, FOCAL};
use omega_store::{Cart, Platform};

use crate::cart::{cart_box, gb_shell_of, label_colour, label_text, CART_H};
use crate::hud::Millis;
use crate::silhouette::GbShell;
use crate::slot_chrome::{draw_empty_slot, ease, MOUTH_H, SLOT_RAISE};
use crate::spin::{self, Pose};
use crate::stage::{stage_y, MOCKUP_H};
use crate::status_bar::TOP_BAR_H;

const SIDE_SCALE: f32 = 0.78;
const SIDE_ALPHA: f32 = 0.55;
/// How big a Game Boy pak stands on the shelf, as a share of its own size: at its own size it
/// is taller than the room between the bar across the top and the slot. It comes up to its own
/// size on the way in.
pub const GB_SHELF_SCALE: f32 = 0.9;

/// How big a cart for `platform` stands on the shelf, as a share of its own size.
pub fn shelf_scale(platform: Platform) -> f32 {
    match platform {
        Platform::Gba => 1.0,
        Platform::Gb | Platform::Gbc => GB_SHELF_SCALE,
    }
}

/// The size a cart for `platform` stands on the shelf at, and spaces its neighbours by.
pub fn shelf_box(platform: Platform) -> (f32, f32) {
    let (w, h) = cart_box(platform);
    let scale = shelf_scale(platform);
    (w as f32 * scale, h as f32 * scale)
}

/// Where a cart `h` tall stands on the shelf, between the bar across the top and the slot on
/// the bar across the bottom: a pak taller than a GBA cart is centred there, and anything
/// shorter stands on the floor a GBA cart centred there would. A pak shrunk on the shelf is
/// still taller than a GBA cart, so it is centred where it would stand at its own size.
pub fn rest_y(h: f32) -> f32 {
    rest_y_under(h, SLOT_RAISE)
}

/// Where a cart `h` tall would stand with the slot `raise` above the bottom, as `rest_y` puts
/// it under the slot where it is.
pub fn rest_y_under(h: f32, raise: f32) -> f32 {
    let (top, floor) = (TOP_BAR_H, MOCKUP_H - raise - MOUTH_H);
    let y = if h > CART_H as f32 {
        top + (floor - top - h) / 2.0
    } else {
        (top + floor + CART_H as f32) / 2.0 - h
    };
    y + stage_y()
}

pub fn foot_y(h: f32) -> f32 {
    rest_y(h) + h
}

/// How far above where it stands a cart `w` by `h` can reach as it sways: it floats up, and
/// the side that turns towards the eye grows.
pub fn sway_reach(w: f32, h: f32) -> f32 {
    let grown = FOCAL / (FOCAL - w / 2.0 * SWAY.sin());
    BOB + h / 2.0 * (grown - 1.0)
}
const OMEGA: f32 = 16.0;
const PART: f32 = 130.0;

const SLOTS: i32 = 3;

/// The selected cart sways this far either way about its upright axis, in radians, once the
/// shelf stands still...
const SWAY: f32 = 10.0 * std::f32::consts::PI / 180.0;
const SWAY_S: f32 = 3.6;
/// ...and floats this many canvas pixels up and down, out of step with the sway so the two
/// never fall into one mechanical beat.
const BOB: f32 = 4.0;
const BOB_S: f32 = 2.4;
/// Both periods divide this, so the clock that drives them wraps here without a jump.
const LOOP_S: f32 = 7.2;
/// How fast the sway comes in once the shelf stops, and goes once it moves, per second.
const CALM_IN: f32 = 1.6;
const CALM_OUT: f32 = 8.0;
/// Carts lean into a scroll this many radians per cart a second, up to LEAN_MAX, and stand
/// straight once the scroll is slower than LEAN_FROM carts a second.
const LEAN: f32 = 0.045;
const LEAN_MAX: f32 = 0.35;
const LEAN_FROM: f32 = 0.05;
/// A turn or a lift smaller than these cannot be seen, so the cart is drawn square and level.
const STILL_YAW: f32 = 0.002;
const STILL_LIFT: f32 = 0.05;

const REPEAT_DELAY_MS: Millis = 400;
const REPEAT_MS: [Millis; 4] = [110, 85, 65, 50];

pub struct Shelf {
    pub carts: Vec<Cart>,
    pub index: usize,
    pub scroll: f32,
    faces: Vec<TexId>,
    shadow: Option<TexId>,
    gb_shadow: Option<TexId>,
    gbc_shadow: Option<TexId>,
    shells: Vec<Option<GbShell>>,
    ride: f32,
    vel: f32,
    held: Option<(i32, Millis, usize)>,
    calm: f32,
    age: f32,
    still: bool,
}

impl Shelf {
    pub fn new(carts: Vec<Cart>) -> Self {
        Shelf {
            shells: carts.iter().map(gb_shell_of).collect(),
            carts,
            index: 0,
            scroll: 0.0,
            faces: Vec::new(),
            shadow: None,
            gb_shadow: None,
            gbc_shadow: None,
            ride: 0.0,
            vel: 0.0,
            held: None,
            calm: 0.0,
            age: 0.0,
            still: false,
        }
    }

    pub fn select(&mut self, i: usize) {
        self.index = i;
        self.scroll = i as f32;
        self.ride = i as f32;
        self.vel = 0.0;
    }

    pub fn set_shadow(&mut self, face: TexId) {
        self.shadow = Some(face);
    }

    pub fn set_gb_shadow(&mut self, shell: GbShell, face: TexId) {
        match shell {
            GbShell::Notched => self.gb_shadow = Some(face),
            GbShell::Rounded => self.gbc_shadow = Some(face),
        }
    }

    pub fn set_faces(&mut self, faces: Vec<TexId>) {
        self.faces = faces;
    }

    pub fn find(&self, stem: &str) -> Option<(&Cart, Option<TexId>)> {
        let i = self.carts.iter().position(|c| c.stem == stem)?;
        Some((&self.carts[i], self.faces.get(i).copied()))
    }

    pub fn left(&mut self) {
        self.step(-1);
    }

    pub fn right(&mut self) {
        self.step(1);
    }

    pub fn jump_next_letter(&mut self) {
        self.jump(1);
    }

    pub fn jump_prev_letter(&mut self) {
        self.jump(-1);
    }

    fn start_of_letter(&self, from: usize) -> usize {
        let n = self.carts.len();
        let letter = omega_store::initial(&self.carts[from].stem);
        let mut at = from;
        for _ in 0..n {
            let before = (at as i32 - 1).rem_euclid(n as i32) as usize;
            if omega_store::initial(&self.carts[before].stem) != letter {
                break;
            }
            at = before;
        }
        at
    }

    fn jump(&mut self, dir: i32) {
        let n = self.carts.len();
        if n < 2 {
            return;
        }
        let wrap = |i: i32| i.rem_euclid(n as i32) as usize;
        let here = omega_store::initial(&self.carts[self.index].stem);
        if self
            .carts
            .iter()
            .all(|c| omega_store::initial(&c.stem) == here)
        {
            return;
        }
        let target = match dir > 0 {
            true => {
                let mut at = self.index;
                for _ in 0..n {
                    at = wrap(at as i32 + 1);
                    if omega_store::initial(&self.carts[at].stem) != here {
                        break;
                    }
                }
                at
            }
            false => {
                let start = self.start_of_letter(self.index);
                match start == self.index {
                    true => self.start_of_letter(wrap(start as i32 - 1)),
                    false => start,
                }
            }
        };
        let ahead = (target as i32 - self.index as i32).rem_euclid(n as i32);
        let delta = match dir > 0 {
            true => ahead,
            false => ahead - n as i32,
        };
        self.index = target;
        self.ride += delta as f32;
    }

    pub fn hold_left(&mut self, now: Millis) {
        self.hold(-1, now);
    }

    pub fn hold_right(&mut self, now: Millis) {
        self.hold(1, now);
    }

    fn hold(&mut self, by: i32, now: Millis) {
        self.step(by);
        self.held = Some((by, now + REPEAT_DELAY_MS, 0));
    }

    pub fn release_left(&mut self) {
        self.release(-1);
    }

    pub fn release_right(&mut self) {
        self.release(1);
    }

    fn release(&mut self, by: i32) {
        if matches!(self.held, Some((held, _, _)) if held == by) {
            self.held = None;
        }
    }

    pub fn release_hold(&mut self) {
        self.held = None;
    }

    pub fn tick(&mut self, now: Millis) {
        let Some((by, due, fired)) = self.held else {
            return;
        };
        if now < due {
            return;
        }
        self.step(by);
        let rate = REPEAT_MS[fired.min(REPEAT_MS.len() - 1)];
        self.held = Some((by, now + rate, fired + 1));
    }

    fn step(&mut self, by: i32) {
        let n = self.carts.len();
        if n < 2 {
            return;
        }
        self.index = (self.index as i32 + by).rem_euclid(n as i32) as usize;
        self.ride += by as f32;
    }

    pub fn scroll_target(&self) -> f32 {
        let n = self.carts.len();
        if n == 0 {
            return 0.0;
        }
        let from = self.ride;
        let n = n as f32;
        from + (self.index as f32 - from + n / 2.0).rem_euclid(n) - n / 2.0
    }

    pub fn cart_at_offset(&self, off: i32) -> Option<usize> {
        let n = self.carts.len() as i32;
        if n == 0 {
            return None;
        }
        if n == 1 {
            return (off == 0).then_some(self.index);
        }
        Some((self.index as i32 + off).rem_euclid(n) as usize)
    }

    /// Where the selected cart's left edge is on the row right now, and how big it is drawn
    /// there as a share of its own size.
    pub fn selected_at(&self) -> (f32, f32) {
        let platform = self
            .carts
            .get(self.index)
            .map_or(Platform::Gba, |c| c.platform);
        let w = shelf_box(platform).0;
        let offset = self.scroll_target() - self.scroll;
        let scale = shrink(offset);
        (
            out_w() as f32 / 2.0 + offset * w - w * scale / 2.0,
            scale * shelf_scale(platform),
        )
    }

    /// How the selected cart is held right now, wherever it is in the row.
    pub fn pose(&self) -> Pose {
        self.pose_at(self.scroll_target() - self.scroll)
    }

    /// How a cart `offset` places from the middle of the row is held: the one in the middle
    /// sways and floats while the shelf stands still, and every cart leans into a scroll.
    fn pose_at(&self, offset: f32) -> Pose {
        let calm = ease(self.calm) * (1.0 - offset.abs().min(1.0));
        let lean = match self.vel.abs() < LEAN_FROM {
            true => 0.0,
            false => (-self.vel * LEAN).clamp(-LEAN_MAX, LEAN_MAX),
        };
        let wave = |period: f32| (self.age * std::f32::consts::TAU / period).sin();
        let seen = |v: f32, least: f32| if v.abs() < least { 0.0 } else { v };
        Pose {
            yaw: seen(SWAY * calm * wave(SWAY_S) + lean, STILL_YAW),
            lift: seen(BOB * calm * wave(BOB_S), STILL_LIFT),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.calm_down(dt);
        let accel = -2.0 * OMEGA * self.vel - OMEGA * OMEGA * (self.scroll - self.scroll_target());
        self.vel += accel * dt;
        self.scroll += self.vel * dt;
    }

    /// Keeps the selected cart from swaying while something else has it, such as the core
    /// picker: it settles square and level, and starts again once let go.
    pub fn hold_still(&mut self, still: bool) {
        self.still = still;
    }

    fn calm_down(&mut self, dt: f32) {
        let moving = (self.scroll - self.scroll_target()).abs() > 0.01 || self.vel.abs() > 0.2;
        if moving || self.still {
            self.calm = (self.calm - CALM_OUT * dt).max(0.0);
        } else {
            if self.calm == 0.0 {
                self.age = 0.0;
            }
            self.calm = (self.calm + CALM_IN * dt).min(1.0);
        }
        self.age = (self.age + dt).rem_euclid(LOOP_S);
    }

    pub fn draw(&self, shake: f32, out: &mut Vec<Draw>) {
        self.draw_row(None, shake, 0.0, 1.0, out);
        draw_empty_slot(out);
    }

    pub fn draw_row(
        &self,
        hidden: Option<&str>,
        shake: f32,
        recede: f32,
        dim: f32,
        out: &mut Vec<Draw>,
    ) {
        let recede = recede.clamp(0.0, 1.0);
        let dim = dim.clamp(0.0, 1.0);
        let target = self.scroll_target();
        for slot in -SLOTS..=SLOTS {
            let Some(i) = self.cart_at_offset(slot) else {
                continue;
            };
            let cart = &self.carts[i];
            if hidden == Some(cart.stem.as_str()) {
                continue;
            }
            let offset = target + slot as f32 - self.scroll;
            let t = offset.abs().min(1.0);
            let scale = shrink(offset);
            let alpha = (1.0 + (SIDE_ALPHA - 1.0) * t) * (1.0 - recede);
            let (cw, ch) = shelf_box(cart.platform);
            let (w, h) = (cw * scale, ch * scale);
            let away = offset.signum() * (1.0 + offset.abs());
            let x = out_w() as f32 / 2.0 + offset * cw - w / 2.0 + away * PART * recede;
            if x + w <= 0.0 || x >= out_w() as f32 || alpha <= 0.0 {
                continue;
            }
            let x = x + shake;
            let y = foot_y(ch) - h;
            let face = self.faces.get(i).copied();
            // A cart still waiting for its face stands still: only a face can turn.
            let pose = match face {
                Some(_) => self.pose_at(offset).toward_rest(recede),
                None => Pose::REST,
            };
            let y = y - pose.lift;
            if let Some(tex) = face {
                spin::push_cart_side(
                    out,
                    (x, y, w, h),
                    tex,
                    0.0,
                    pose.yaw,
                    alpha * dim,
                    recede_alpha(alpha),
                );
            }
            if alpha < 1.0 {
                let backing = match self.shells.get(i).copied().flatten() {
                    None => self.shadow,
                    Some(GbShell::Notched) => self.gb_shadow,
                    Some(GbShell::Rounded) => self.gbc_shadow,
                };
                if let Some(tex) = backing {
                    out.push(spin::posed_face(
                        (x, y, w, h),
                        tex,
                        recede_alpha(alpha),
                        0.0,
                        pose.yaw,
                    ));
                }
            }
            out.push(match face {
                Some(tex) => spin::posed_face((x, y, w, h), tex, alpha * dim, 0.0, pose.yaw),
                None => {
                    let c = label_colour(&label_text(cart));
                    Draw::Rect {
                        x,
                        y,
                        w,
                        h,
                        colour: [
                            c[0] as f32 / 255.0,
                            c[1] as f32 / 255.0,
                            c[2] as f32 / 255.0,
                            alpha * dim,
                        ],
                    }
                }
            });
        }
    }
}

fn shrink(offset: f32) -> f32 {
    1.0 + (SIDE_SCALE - 1.0) * offset.abs().min(1.0)
}

fn recede_alpha(face_alpha: f32) -> f32 {
    (face_alpha / SIDE_ALPHA).clamp(0.0, 1.0)
}
