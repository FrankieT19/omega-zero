use omega_store::{Cart, Platform};
use omega_ui::{
    posed_face, push_cart_side, rest_y, Draw, Pose, Shelf, TexId, CART_H, OUTLINE_INSET, THICK,
};

const DT: f32 = 1.0 / 60.0;

fn shelf_with(n: usize) -> Shelf {
    Shelf::new(
        (0..n)
            .map(|i| Cart {
                platform: Platform::Gba,
                stem: format!("Game {i}"),
                rom: format!("Games/GBA/Game {i}.gba").into(),
                label: None,
                code: String::new(),
                shell: None,
                title: format!("GAME {i}"),
            })
            .collect(),
    )
}

fn faced(n: usize) -> Shelf {
    let mut s = shelf_with(n);
    s.set_faces((0..n).map(|i| TexId::from_raw(40 + i)).collect());
    s
}

fn run(s: &mut Shelf, seconds: f32) {
    for _ in 0..(seconds / DT).round() as usize {
        s.update(DT);
    }
}

fn row(s: &Shelf) -> Vec<Draw> {
    let mut out = Vec::new();
    s.draw_row(None, 0.0, 0.0, 1.0, &mut out);
    out
}

#[test]
fn the_selected_cart_sways_ten_degrees_either_way_and_floats_four_pixels() {
    let mut s = faced(3);
    run(&mut s, 2.0);
    let (mut yaw, mut lift) = ((f32::MAX, f32::MIN), (f32::MAX, f32::MIN));
    for _ in 0..8 * 60 {
        s.update(DT);
        let p = s.pose();
        yaw = (yaw.0.min(p.yaw), yaw.1.max(p.yaw));
        lift = (lift.0.min(p.lift), lift.1.max(p.lift));
    }
    let ten = 10f32.to_radians();
    for (name, (low, high), most) in [("sway", yaw, ten), ("float", lift, 4.0)] {
        assert!(
            high <= most * 1.0001 && high > most * 0.99,
            "the {name} reached {high} one way, for {most}"
        );
        assert!(
            low >= -most * 1.0001 && low < -most * 0.99,
            "the {name} reached {low} the other way, for {most}"
        );
    }
}

#[test]
fn a_shelf_that_has_just_stopped_eases_into_the_sway() {
    let mut s = faced(3);
    run(&mut s, 0.1);
    assert!(
        s.pose().yaw.abs() < 1f32.to_radians(),
        "the sway came in at full swing: {}",
        s.pose().yaw
    );
}

#[test]
fn the_sway_stops_while_the_shelf_scrolls_and_the_carts_lean_into_it() {
    let mut s = faced(5);
    run(&mut s, 3.0);
    s.right();
    run(&mut s, 0.15);
    let p = s.pose();
    assert!(
        p.yaw < -0.05,
        "carts sliding left did not lean into it: {}",
        p.yaw
    );
    assert_eq!(p.lift, 0.0, "the cart kept floating through the scroll");

    run(&mut s, 0.6);
    let spun: Vec<TexId> = row(&s)
        .iter()
        .filter_map(|d| match *d {
            Draw::Spun { tex, .. } => Some(tex),
            _ => None,
        })
        .collect();
    let selected = TexId::from_raw(40 + s.index);
    assert!(
        spun.iter().all(|t| *t == selected),
        "a cart beside the selected one still leans once the scroll has settled"
    );
}

#[test]
fn a_cart_still_waiting_for_its_face_stands_still() {
    let mut s = shelf_with(3);
    run(&mut s, 3.0);
    let out = row(&s);
    assert!(
        out.iter().all(|d| matches!(d, Draw::Rect { .. })),
        "a cart with no face turned or grew a side"
    );
    let middle = out
        .iter()
        .find_map(|d| match *d {
            Draw::Rect { y, h, .. } if h == CART_H as f32 => Some(y),
            _ => None,
        })
        .expect("no cart at full size");
    assert_eq!(middle, rest_y(CART_H as f32), "a cart with no face floated");
}

#[test]
fn held_still_the_selected_cart_settles_square_and_level_and_sways_again_once_let_go() {
    let mut s = faced(3);
    run(&mut s, 3.0);
    s.hold_still(true);
    run(&mut s, 0.2);
    assert_eq!(s.pose(), Pose::REST, "the cart kept swaying while held");
    s.hold_still(false);
    run(&mut s, 2.0);
    assert_ne!(s.pose(), Pose::REST, "the cart never swayed again");
}

#[test]
fn the_side_shows_on_the_side_that_turns_towards_the_eye() {
    let (x, y, w, h) = (100.0, 50.0, 360.0, 211.0);
    for (yaw, near_left) in [(0.17, true), (-0.17, false)] {
        let mut out = Vec::new();
        push_cart_side(
            &mut out,
            (x, y, w, h),
            TexId::from_raw(1),
            0.0,
            yaw,
            1.0,
            1.0,
        );
        assert!(!out.is_empty(), "a cart turned {yaw} has no side");
        let mut last = f32::MAX;
        for d in &out {
            let Draw::Spun {
                depth,
                clip,
                colour,
                yaw: drawn,
                ..
            } = *d
            else {
                panic!("the side is drawn with something other than spun slices: {d:?}");
            };
            assert_eq!(drawn, yaw);
            assert!(
                depth > 0.0 && depth <= THICK * w + 0.001 && depth < last,
                "the slices do not step forward from the back of the cart: {depth} after {last}"
            );
            last = depth;
            assert!(colour[0] < 1.0, "the side is as bright as the face");
            let [left, _, wide, _] = clip.expect("an upright side is not kept to its band");
            let inset = OUTLINE_INSET * w + 2.0;
            match near_left {
                true => assert!(
                    left < x && left + wide <= x + inset,
                    "turned {yaw}, the side's band {left}+{wide} is not on the left edge"
                ),
                false => assert!(
                    left >= x + w - inset && left + wide > x + w,
                    "turned {yaw}, the side's band {left}+{wide} is not on the right edge"
                ),
            }
        }
    }
}

#[test]
fn a_cart_turned_in_the_canvas_draws_its_side_whole() {
    let mut out = Vec::new();
    push_cart_side(
        &mut out,
        (100.0, 50.0, 360.0, 211.0),
        TexId::from_raw(1),
        0.2,
        0.17,
        1.0,
        1.0,
    );
    assert!(!out.is_empty());
    assert!(
        out.iter()
            .all(|d| matches!(d, Draw::Spun { clip: None, .. })),
        "a side that is not upright was cut to an upright band"
    );
}

#[test]
fn a_side_too_thin_to_see_is_not_drawn() {
    let mut out = Vec::new();
    push_cart_side(
        &mut out,
        (100.0, 50.0, 360.0, 211.0),
        TexId::from_raw(1),
        0.0,
        0.005,
        1.0,
        1.0,
    );
    assert!(out.is_empty(), "a side under half a pixel was drawn");
}

#[test]
fn an_unspun_face_is_drawn_as_it_always_was() {
    let (rect, tex) = ((1.0, 2.0, 3.0, 4.0), TexId::from_raw(7));
    assert_eq!(
        posed_face(rect, tex, 0.5, 0.0, 0.0),
        Draw::Tex {
            x: 1.0,
            y: 2.0,
            w: 3.0,
            h: 4.0,
            tex,
            alpha: 0.5
        }
    );
    assert_eq!(
        posed_face(rect, tex, 0.5, 0.3, 0.0),
        Draw::Turned {
            x: 1.0,
            y: 2.0,
            w: 3.0,
            h: 4.0,
            tex,
            alpha: 0.5,
            turn: 0.3
        }
    );
    assert!(matches!(
        posed_face(rect, tex, 0.5, 0.0, 0.1),
        Draw::Spun {
            yaw: 0.1,
            depth: 0.0,
            ..
        }
    ));
}
