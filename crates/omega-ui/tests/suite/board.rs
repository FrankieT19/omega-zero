use omega_store::{Cart, Core, Platform};
use omega_ui::{
    board_at, board_face, chip_face, chip_shadow_face, gba_shell_for, lid_at, lid_shadow_face,
    lift_of, on_board, padded, rom_marking, rom_marking_face, shelf_cart, slide_of, socket_face,
    CartFace, Placed, BOARD_H, BOARD_W, CART_H, CHIP_H, CHIP_U, CHIP_V, CHIP_W, DEFAULT_SHELL,
    LID_SHADOW_H, LID_SHADOW_W, LID_TURN, ROM_H, ROM_W, ROM_X, ROM_Y, SHADOW_H, SHADOW_W,
    SLIDE_SHARE, SLIDE_UP, SOCKET_H, SOCKET_U, SOCKET_V, SOCKET_W,
};
use omega_ui::{out_h, stage_x, stage_y, BAR_H, TOP_BAR_H};

fn cart(stem: &str, code: &str) -> Cart {
    Cart {
        platform: Platform::Gba,
        stem: stem.into(),
        rom: format!("Games/GBA/{stem}.gba").into(),
        label: None,
        code: code.into(),
        shell: None,
        title: stem.to_uppercase(),
    }
}

fn rgb(face: &CartFace, x: u32, y: u32) -> [u8; 3] {
    let i = ((y * face.w + x) * 4) as usize;
    [face.rgba[i], face.rgba[i + 1], face.rgba[i + 2]]
}

fn near(a: [u8; 3], b: [u8; 3]) -> bool {
    (0..3).all(|k| (a[k] as i32 - b[k] as i32).abs() <= 3)
}

const EMERALD: &str = "Pokemon - Emerald Version (USA, Europe)";

#[test]
fn the_marking_is_the_title_stacked() {
    assert_eq!(rom_marking(EMERALD), ["POKEMON", "EMERALD", "VERSION"]);
}

#[test]
fn a_long_title_folds_its_tail_into_the_third_line() {
    assert_eq!(
        rom_marking("Advance Wars 2 - Black Hole Rising"),
        ["ADVANCE", "WARS 2", "BLACK HOLE RISING"]
    );
}

#[test]
fn bracketed_tags_never_reach_the_chip() {
    assert_eq!(
        rom_marking("Pokemon - LeafGreen Version (USA, Europe) (Rev 1)"),
        ["POKEMON", "LEAFGREEN", "VERSION"]
    );
    assert_eq!(rom_marking("Metroid Fusion"), ["METROID", "FUSION"]);
}

#[test]
fn the_marking_is_drawn_and_stays_on_the_chip() {
    for stem in [
        EMERALD,
        "Metroid Fusion",
        "Advance Wars 2 - Black Hole Rising",
        "Pokemon - LeafGreen Version (USA, Europe) (Rev 1)",
    ] {
        let face = rom_marking_face(stem);
        assert_eq!((face.w, face.h), (ROM_W, ROM_H));
        let alpha = |x: u32, y: u32| face.rgba[((y * face.w + x) * 4 + 3) as usize];
        assert!(
            (0..face.h).any(|y| (0..face.w).any(|x| alpha(x, y) > 128)),
            "{stem} left no marking"
        );
        for y in 0..face.h {
            assert_eq!(alpha(0, y), 0, "{stem} ran off the left of the chip");
            assert_eq!(
                alpha(face.w - 1, y),
                0,
                "{stem} ran off the right of the chip"
            );
        }
    }
}

#[test]
fn the_cell_carries_no_print() {
    let face = board_face(&cart(EMERALD, "BPEE"));
    let dark = (24..37)
        .flat_map(|y| (146..159).map(move |x| (x, y)))
        .filter(|&(x, y)| (x as i32 - 152).pow(2) + (y as i32 - 30).pow(2) <= 30)
        .filter(|&(x, y)| rgb(&face, x, y)[0] < 150)
        .count();
    assert_eq!(dark, 0, "the cell has {dark} dark pixels of print");
}

#[test]
fn the_board_is_drawn_a_texel_to_an_art_pixel() {
    let face = board_face(&cart(EMERALD, "BPEE"));
    assert_eq!((face.w, face.h), (BOARD_W, BOARD_H));
    assert_eq!((BOARD_W, BOARD_H), (186, 104));
    let rest = board_at(1.0);
    assert_eq!(
        (rest.w, rest.h),
        (372.0, 208.0),
        "the board does not rest at 2x"
    );
}

#[test]
fn the_back_shell_is_the_carts_own_plastic() {
    let emerald = board_face(&cart(EMERALD, "BPEE"));
    assert!(
        near(rgb(&emerald, 5, 70), gba_shell_for("BPEE").colour),
        "Emerald's rim is {:?}",
        rgb(&emerald, 5, 70)
    );
    let unknown = board_face(&cart("Homebrew", ""));
    assert!(
        near(rgb(&unknown, 5, 70), DEFAULT_SHELL.colour),
        "an unknown cart's rim is {:?}",
        rgb(&unknown, 5, 70)
    );
}

#[test]
fn the_board_itself_is_drawn() {
    let face = board_face(&cart(EMERALD, "BPEE"));
    assert!(
        near(rgb(&face, 128, 45), [0x3a, 0x9a, 0x3c]),
        "the board is {:?}",
        rgb(&face, 128, 45)
    );
}

fn alpha(face: &CartFace, x: u32, y: u32) -> u8 {
    face.rgba[((y * face.w + x) * 4 + 3) as usize]
}

/// Every pixel of a pixel-art face is there or not: no edge is part covered.
fn assert_flat(face: &CartFace, what: &str) {
    for (i, px) in face.rgba.chunks_exact(4).enumerate() {
        assert!(
            px[3] == 0 || px[3] == 255,
            "{what} is part covered at {},{}: alpha {}",
            i as u32 % face.w,
            i as u32 / face.w,
            px[3]
        );
    }
}

#[test]
fn every_face_of_the_picker_is_drawn_without_soft_edges() {
    assert_flat(&board_face(&cart(EMERALD, "BPEE")), "the board");
    assert_flat(&rom_marking_face(EMERALD), "the marking");
    for core in Core::ALL {
        assert_flat(&socket_face(core), "a socket");
        assert_flat(&chip_face(Some(core)), "a chip");
    }
    assert_flat(&chip_face(None), "the blank chip");
    assert_flat(&chip_shadow_face(), "the chip's shadow");
    assert_flat(&lid_shadow_face(), "the lid's shadow");
}

#[test]
fn the_marking_is_printed_on_the_rom_chip() {
    let face = board_face(&cart(EMERALD, "BPEE"));
    let ink = [0xbd, 0xbd, 0xbd];
    let inked = (ROM_Y..ROM_Y + ROM_H)
        .flat_map(|y| (ROM_X..ROM_X + ROM_W).map(move |x| (x, y)))
        .filter(|&(x, y)| rgb(&face, x, y) == ink)
        .count();
    assert!(
        inked > 100,
        "the ROM chip carries {inked} pixels of marking"
    );
}

/// A point on the 720x480 mockup, where it lands on this canvas.
fn mock(x: f32, y: f32) -> (f32, f32) {
    (x + stage_x(), y + stage_y())
}

#[test]
fn the_open_cart_starts_as_the_shelf_cart_and_lands_where_the_mockup_has_it() {
    let shelf = shelf_cart();
    assert_eq!(
        (shelf.x, shelf.w, shelf.h),
        (mock(180.0, 0.0).0, 360.0, 212.0)
    );
    assert_eq!(
        shelf.y + shelf.h / 2.0,
        (stage_y() + TOP_BAR_H + out_h() as f32 - BAR_H) / 2.0,
        "not centred between the bar across the top and the bar across the bottom"
    );
    assert_eq!(board_at(0.0), shelf_cart());
    let (x, y) = mock(174.0, 140.0);
    assert_eq!(
        board_at(1.0),
        Placed {
            x,
            y,
            w: 372.0,
            h: 208.0
        }
    );
    assert_eq!(board_at(1.7), board_at(1.0), "an overshoot kept growing");
}

#[test]
fn the_lid_leaves_level_and_rests_turned() {
    assert_eq!(lid_at(0.0), (shelf_cart(), 0.0));
    let (rest, turn) = lid_at(1.0);
    let (x, y) = mock(288.0, 44.0);
    assert_eq!(
        rest,
        Placed {
            x,
            y,
            w: 144.0,
            h: 81.0
        }
    );
    assert_eq!(turn, LID_TURN);
    assert!(
        (LID_TURN.to_degrees() + 5.0).abs() < 1e-4,
        "{}",
        LID_TURN.to_degrees()
    );
}

#[test]
fn the_beats_split_one_progress() {
    assert_eq!((slide_of(0.0), lift_of(0.0)), (0.0, 0.0));
    assert_eq!((slide_of(SLIDE_SHARE), lift_of(SLIDE_SHARE)), (1.0, 0.0));
    assert_eq!((slide_of(1.0), lift_of(1.0)), (1.0, 1.0));
    assert!((slide_of(SLIDE_SHARE / 2.0) - 0.5).abs() < 1e-5);
}

#[test]
fn the_back_half_stays_on_the_shelf_while_the_front_slides() {
    for p in [0.0, 0.1, 0.25, SLIDE_SHARE] {
        assert_eq!(board_at(p), shelf_cart(), "the back moved at progress {p}");
    }
}

#[test]
fn the_front_slides_up_a_fifth_before_it_lifts() {
    assert_eq!(SLIDE_UP, CART_H as f32 / 5.0);
    assert!(
        shelf_cart().y - SLIDE_UP > lid_at(1.0).0.y,
        "the slide ends above where the lid rests, so the lift would carry it back down"
    );
    let shelf = shelf_cart();
    let (slid, turn) = lid_at(SLIDE_SHARE);
    assert_eq!(
        slid,
        Placed {
            y: shelf.y - SLIDE_UP,
            ..shelf
        }
    );
    assert_eq!(turn, 0.0);
    let (half, turn) = lid_at(SLIDE_SHARE / 2.0);
    assert!(half.y < shelf.y && half.y > slid.y && turn == 0.0 && half.w == shelf.w);
}

#[test]
fn the_lift_starts_where_the_slide_ends() {
    let (slid, _) = lid_at(SLIDE_SHARE);
    let (next, _) = lid_at(SLIDE_SHARE + 0.001);
    assert!((next.y - slid.y).abs() < 0.5 && (next.w - slid.w).abs() < 0.5);
}

#[test]
fn the_boards_art_pixels_land_on_the_panel_at_twice() {
    let board = board_at(1.0);
    assert_eq!(on_board(board, 0.0, 0.0), mock(174.0, 140.0));
    assert_eq!(on_board(board, 186.0, 104.0), mock(546.0, 348.0));
    let (x, y) = on_board(board, 1.0, 1.0);
    assert_eq!((x - board.x, y - board.y), (2.0, 2.0));
}

#[test]
fn the_sockets_and_the_chip_sit_on_whole_art_pixels_of_the_board() {
    let whole = |v: f32| v.fract() == 0.0;
    for i in 0..2 {
        assert!(
            whole(SOCKET_U[i]) && whole(CHIP_U[i]),
            "socket {i} is off the grid"
        );
        assert!(
            SOCKET_U[i] < CHIP_U[i] && CHIP_U[i] + CHIP_W as f32 <= SOCKET_U[i] + SOCKET_W as f32,
            "the chip overhangs socket {i}"
        );
        assert!(SOCKET_U[i] + SOCKET_W as f32 <= BOARD_W as f32);
    }
    assert!(whole(SOCKET_V) && whole(CHIP_V));
    assert!(SOCKET_V < CHIP_V && CHIP_V + CHIP_H as f32 <= SOCKET_V + SOCKET_H as f32);
    assert!(
        SOCKET_U[0] + SOCKET_W as f32 <= SOCKET_U[1],
        "the sockets overlap"
    );
}

#[test]
fn a_socket_prints_its_cores_name_in_flat_half_strength_ink() {
    // the silkscreen's colour halfway to the board's
    let ink = [0x94, 0xc7, 0x91];
    let printed = |core: Core| -> Vec<(u32, u32)> {
        let face = socket_face(core);
        assert_eq!((face.w, face.h), (SOCKET_W, SOCKET_H));
        (0..face.h)
            .flat_map(|y| (0..face.w).map(move |x| (x, y)))
            .filter(|&(x, y)| alpha(&face, x, y) == 255 && rgb(&face, x, y) == ink)
            .collect()
    };
    let (mgba, gpsp) = (printed(Core::Mgba), printed(Core::Gpsp));
    assert!(
        mgba.len() > 20 && gpsp.len() > 20,
        "a socket's name is barely there"
    );
    assert_ne!(mgba, gpsp, "both sockets print the same name");
}

#[test]
fn the_chip_is_level_and_needs_no_room_to_turn() {
    for core in [None, Some(Core::Mgba), Some(Core::Gpsp)] {
        let face = chip_face(core);
        assert_eq!((face.w, face.h), (CHIP_W, CHIP_H));
        assert_eq!(alpha(&face, 0, 0), 0, "{core:?}'s pins reach its corner");
        assert_eq!(
            alpha(&face, 0, CHIP_H / 2),
            255,
            "{core:?}'s body does not reach its side"
        );
    }
}

#[test]
fn a_seated_chip_wears_its_name_and_a_flying_one_is_blank() {
    let light = |face: &CartFace| {
        face.rgba
            .chunks_exact(4)
            .filter(|p| p[0] > 200 && p[1] > 200 && p[2] > 200 && p[3] > 200)
            .count()
    };
    assert!(
        light(&chip_face(Some(Core::Mgba))) > 20,
        "mGBA left no name"
    );
    assert_eq!(light(&chip_face(None)), 0, "the blank chip says something");
    assert_ne!(
        chip_face(Some(Core::Mgba)).rgba,
        chip_face(Some(Core::Gpsp)).rgba,
        "both cores wear the same mark"
    );
}

#[test]
fn padding_moves_nothing() {
    let face = chip_face(Some(Core::Gpsp));
    let more = padded(&face, 3);
    assert_eq!((more.w, more.h), (face.w + 6, face.h + 6));
    for (x, y) in [(0, 0), (10, 7), (face.w - 1, face.h - 1)] {
        let a = ((y * face.w + x) * 4) as usize;
        let b = (((y + 3) * more.w + x + 3) * 4) as usize;
        assert_eq!(face.rgba[a..a + 4], more.rgba[b..b + 4]);
    }
}

#[test]
fn a_shadow_is_a_checker_of_black_with_its_corners_clear() {
    for (s, (w, h)) in [
        (chip_shadow_face(), (SHADOW_W, SHADOW_H)),
        (lid_shadow_face(), (LID_SHADOW_W, LID_SHADOW_H)),
    ] {
        assert_eq!((s.w, s.h), (w, h));
        for y in 0..h {
            for x in 0..w {
                if alpha(&s, x, y) == 0 {
                    continue;
                }
                assert_eq!(rgb(&s, x, y), [0, 0, 0], "a shadow pixel is not black");
                assert_eq!((x + y) % 2, 0, "{x},{y} is off the checker");
            }
        }
        for (x, y) in [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)] {
            assert_eq!(alpha(&s, x, y), 0, "a shadow's corner {x},{y} is inked");
        }
        let middle = (0..w).filter(|&x| alpha(&s, x, h / 2) == 255).count() as u32;
        assert!(
            middle >= w / 2 - 1,
            "a shadow's middle row is a half shade: {middle} of {w}"
        );
    }
}

#[test]
fn the_seated_chips_name_is_printed_in_whole_pixels() {
    let red = |face: &CartFace, x: u32, y: u32| face.rgba[((y * face.w + x) * 4) as usize];
    for core in [Core::Mgba, Core::Gpsp] {
        let named = chip_face(Some(core));
        let blank = chip_face(None);
        let mut inks: Vec<u8> = (0..named.h)
            .flat_map(|y| (0..named.w).map(move |x| (x, y)))
            .filter(|&(x, y)| alpha(&named, x, y) == 255 && red(&named, x, y) != red(&blank, x, y))
            .map(|(x, y)| red(&named, x, y))
            .collect();
        assert!(inks.len() > 10, "{core:?}'s name is barely there");
        inks.sort_unstable();
        inks.dedup();
        assert_eq!(inks.len(), 1, "{core:?}'s name has soft edges: {inks:?}");
    }
}
