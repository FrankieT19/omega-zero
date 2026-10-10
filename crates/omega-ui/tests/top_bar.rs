use omega_power::{Battery, Charge};
use omega_ui::{
    bolt_face, gauge_width, link_face, out_w, word_cap_top, word_face, word_tail, Draw, Link,
    LinkFaces, Links, Printed, TexId, TopBar, CELL_PX, GAUGE_H, LINK_OFF, LINK_ON, TOP_BAR_H,
};

const MARGIN: f32 = 24.0;

fn faces() -> LinkFaces {
    let pair = |base: usize| {
        [
            (TexId::from_raw(base), 20, 24),
            (TexId::from_raw(base + 1), 20, 24),
        ]
    };
    LinkFaces {
        wifi: pair(10),
        bluetooth: pair(20),
        ssh: pair(30),
    }
}

fn bar(links: Links, battery: Option<Battery>, bolt: Option<TexId>) -> Vec<Draw> {
    let faces = faces();
    let mut out = Vec::new();
    TopBar {
        links,
        faces: Some(&faces),
        clock: Printed::new(TexId::from_raw(1), 40),
        battery,
        percent: Printed::new(TexId::from_raw(2), 30),
        bolt,
    }
    .draw(&mut out);
    out
}

fn battery(charge: Charge) -> Option<Battery> {
    Some(Battery {
        percent: 68,
        charge,
    })
}

fn tex_x(out: &[Draw], id: usize) -> Option<(f32, f32)> {
    out.iter().find_map(|d| match *d {
        Draw::Tex { x, w, tex, .. } if tex == TexId::from_raw(id) => Some((x, x + w)),
        _ => None,
    })
}

fn drawn(out: &[Draw], id: usize) -> bool {
    tex_x(out, id).is_some()
}

#[test]
fn the_clock_sits_in_the_middle_and_the_battery_ends_at_the_right_margin() {
    let out = bar(Links::default(), battery(Charge::Discharging), None);
    let (a, b) = tex_x(&out, 1).expect("no clock");
    let ink_middle = (a + b - word_tail()) / 2.0;
    assert!(
        (ink_middle - out_w() as f32 / 2.0).abs() <= 0.5,
        "the clock's ink is centred on {ink_middle}"
    );
    let (_, percent_end) = tex_x(&out, 2).expect("no figure");
    assert_eq!(
        percent_end - word_tail(),
        out_w() as f32 - MARGIN,
        "the figure's ink does not end at the margin"
    );
    let leftmost_gauge = out
        .iter()
        .filter_map(|d| match *d {
            Draw::Rect { x, y, .. } if y > 0.0 && x > out_w() as f32 / 2.0 => Some(x),
            _ => None,
        })
        .fold(f32::MAX, f32::min);
    let figure = Printed::new(TexId::from_raw(2), 30);
    assert!(
        leftmost_gauge >= out_w() as f32 - MARGIN - gauge_width(figure),
        "the gauge starts at {leftmost_gauge}, left of the room it was given"
    );
}

#[test]
fn the_links_line_up_from_the_left_margin_wifi_then_bluetooth_then_ssh() {
    let out = bar(
        Links {
            wifi: true,
            bluetooth: Some(false),
            ssh: true,
        },
        None,
        None,
    );
    let wifi = tex_x(&out, 11).expect("Wi-Fi is not drawn on");
    let bluetooth = tex_x(&out, 20).expect("Bluetooth is not drawn off");
    let ssh = tex_x(&out, 31).expect("ssh is not drawn on");
    assert_eq!(wifi.0, MARGIN);
    assert!(wifi.1 < bluetooth.0 && bluetooth.1 < ssh.0, "out of order");
    for off in [10, 21, 30] {
        assert!(!drawn(&out, off), "face {off} is drawn in the wrong state");
    }
}

#[test]
fn every_link_shows_whether_it_is_on() {
    for (links, wanted) in [
        (Links::default(), [10, 30]),
        (
            Links {
                wifi: true,
                bluetooth: None,
                ssh: true,
            },
            [11, 31],
        ),
    ] {
        let out = bar(links, None, None);
        for id in wanted {
            assert!(drawn(&out, id), "{links:?} does not draw face {id}");
        }
    }
}

#[test]
fn a_handheld_without_bluetooth_shows_no_bluetooth_icon() {
    let out = bar(Links::default(), None, None);
    assert!(!drawn(&out, 20) && !drawn(&out, 21));
    let with = bar(
        Links {
            bluetooth: Some(false),
            ..Links::default()
        },
        None,
        None,
    );
    assert!(drawn(&with, 20), "an off Bluetooth radio is hidden");
}

#[test]
fn everything_on_the_bar_stays_inside_it() {
    let out = bar(
        Links {
            wifi: true,
            bluetooth: Some(true),
            ssh: true,
        },
        battery(Charge::Charging),
        Some(TexId::from_raw(3)),
    );
    for d in &out {
        if let Draw::Rect { y, h, .. } | Draw::Tex { y, h, .. } = *d {
            assert!(
                y >= 0.0 && y + h <= TOP_BAR_H,
                "{d:?} spills out of the bar"
            );
        }
    }
}

#[test]
fn charging_does_not_move_anything() {
    let idle = bar(Links::default(), battery(Charge::Discharging), None);
    let charging = bar(
        Links::default(),
        battery(Charge::Charging),
        Some(TexId::from_raw(3)),
    );
    for d in &idle {
        assert!(
            charging.contains(d),
            "{d:?} moved or vanished when charging started"
        );
    }
}

#[test]
fn a_bar_with_no_battery_still_has_its_clock() {
    let out = bar(Links::default(), None, None);
    assert!(drawn(&out, 1));
}

fn ink(face: &omega_ui::UndoFace) -> Option<[u8; 3]> {
    face.rgba
        .chunks(4)
        .find(|p| p[3] > 200)
        .map(|p| [p[0], p[1], p[2]])
}

#[test]
fn on_and_off_faces_are_the_same_size_and_tell_themselves_apart() {
    for link in Link::ALL {
        let (off, on) = (link_face(link, false), link_face(link, true));
        assert!(on.w > 0 && on.h > 0, "{link:?} has no face");
        assert_eq!(
            (on.w, on.h),
            (off.w, off.h),
            "{link:?} reflows when it changes"
        );
        assert_eq!(ink(&on), Some(LINK_ON), "{link:?} on is not lit");
        assert_eq!(ink(&off), Some(LINK_OFF), "{link:?} off is not dimmed");
    }
    for link in [Link::Wifi, Link::Bluetooth] {
        let shape =
            |on: bool| -> Vec<u8> { link_face(link, on).rgba.chunks(4).map(|p| p[3]).collect() };
        assert_eq!(
            shape(true),
            shape(false),
            "{link:?} changes shape, not only ink, when it goes off"
        );
    }
}

fn inked_rows(face: &omega_ui::UndoFace) -> Vec<u32> {
    (0..face.h)
        .filter(|&y| (0..face.w).any(|x| face.rgba[((y * face.w + x) * 4 + 3) as usize] > 0))
        .collect()
}

#[test]
fn every_icon_is_one_square_cell_of_pixel_art() {
    let cell = CELL_PX as u32;
    for face in [
        link_face(Link::Wifi, true),
        link_face(Link::Bluetooth, true),
        bolt_face(),
    ] {
        assert_eq!((face.w, face.h), (cell, cell), "an icon is not one cell");
        assert!(
            face.rgba.chunks(4).all(|p| p[3] == 0 || p[3] == 255),
            "an icon has soft edges, not whole pixels"
        );
        let rows = inked_rows(&face);
        assert_eq!(
            (rows[0], rows[rows.len() - 1]),
            (0, cell - 1),
            "an icon leaves rows of its cell unused"
        );
    }
    assert_eq!(GAUGE_H, CELL_PX, "the battery is not as tall as the icons");
}

#[test]
fn words_stand_exactly_as_tall_as_the_icons() {
    let ssh = link_face(Link::Ssh, true);
    assert_eq!(ssh.h, CELL_PX as u32, "ssh is not as tall as the icons");
    let figure = word_face("100%");
    let rows = inked_rows(&figure);
    assert_eq!(
        (rows[0] as f32, rows[rows.len() - 1] as f32 + 1.0),
        (word_cap_top(), word_cap_top() + CELL_PX),
        "the figure's ink does not fill one cell below its cap top"
    );
}

#[test]
fn every_item_stands_in_the_same_band() {
    let out = bar(
        Links {
            wifi: true,
            bluetooth: Some(true),
            ssh: true,
        },
        battery(Charge::Charging),
        Some(TexId::from_raw(3)),
    );
    let top = |d: &Draw| match *d {
        Draw::Tex { y, tex, .. } if [1, 2].map(TexId::from_raw).contains(&tex) => {
            Some(y + word_cap_top())
        }
        Draw::Tex { y, h, .. } => Some(y + (h - CELL_PX) / 2.0),
        _ => None,
    };
    let tops: Vec<f32> = out.iter().filter_map(top).collect();
    assert_eq!(tops.len(), 6, "not every item drew: {out:?}");
    assert!(
        tops.iter().all(|&t| t == tops[0]),
        "items stand at different heights: {tops:?}"
    );
    let band = tops[0];
    assert_eq!(
        band + CELL_PX / 2.0,
        (TOP_BAR_H - 2.0) / 2.0,
        "the band is not centred above the rule"
    );
    let capsule_top = out
        .iter()
        .filter_map(|d| match *d {
            Draw::Rect { x, y, .. } if x > out_w() as f32 / 2.0 && y > 0.0 => Some(y),
            _ => None,
        })
        .fold(f32::MAX, f32::min);
    assert_eq!(capsule_top, band, "the battery does not stand in the band");
}

#[test]
fn ssh_is_bolder_than_the_plain_face() {
    let face = link_face(Link::Ssh, true);
    let solid = face.rgba.chunks(4).filter(|p| p[3] > 200).count();
    assert!(solid > 60, "only {solid} solid pixels: ssh is not bold");
    assert!(face.h <= TOP_BAR_H as u32);
}
