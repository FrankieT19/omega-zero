use slot::bootlogo::{choose, dims, install, panel_size, valid, Outcome};

fn bmp(w: u32, h: u32, fill: u8) -> Vec<u8> {
    let stride = (w as usize * 3).next_multiple_of(4);
    let mut b = vec![0u8; 54];
    b[0..2].copy_from_slice(b"BM");
    b[2..6].copy_from_slice(&((54 + stride * h as usize) as u32).to_le_bytes());
    b[10..14].copy_from_slice(&54u32.to_le_bytes());
    b[14..18].copy_from_slice(&40u32.to_le_bytes());
    b[18..22].copy_from_slice(&w.to_le_bytes());
    b[22..26].copy_from_slice(&h.to_le_bytes());
    b[26..28].copy_from_slice(&1u16.to_le_bytes());
    b[28..30].copy_from_slice(&24u16.to_le_bytes());
    b.extend(std::iter::repeat_n(fill, stride * h as usize));
    b
}

fn wide(fill: u8) -> Vec<u8> {
    bmp(720, 480, fill)
}

fn partition_with(logo: &[u8]) -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("bootlogo.bmp"), logo).unwrap();
    d
}

#[test]
fn any_uncompressed_24_bit_bmp_is_valid_at_its_own_size() {
    assert_eq!(dims(&wide(0)), Some((720, 480)));
    assert_eq!(dims(&bmp(640, 480, 0)), Some((640, 480)));
    assert_eq!(dims(&bmp(720, 720, 0)), Some((720, 720)));
    assert_eq!(
        dims(&bmp(481, 3, 0)),
        Some((481, 3)),
        "rows pad to four bytes"
    );
    assert!(!valid(&wide(0)[..1000]));
    let mut deep = wide(0);
    deep[28..30].copy_from_slice(&32u16.to_le_bytes());
    assert!(!valid(&deep));
    let mut packed = wide(0);
    packed[30..34].copy_from_slice(&1u32.to_le_bytes());
    assert!(!valid(&packed));
    let mut png = wide(0);
    png[0..2].copy_from_slice(b"PN");
    assert!(!valid(&png));
    let mut lying = bmp(640, 480, 0);
    lying[18..22].copy_from_slice(&720u32.to_le_bytes());
    assert!(
        !valid(&lying),
        "a header that disagrees with the pixel data"
    );
    assert!(!valid(b"BM"));
}

#[test]
fn a_new_logo_replaces_the_old_and_keeps_the_original_once() {
    let d = partition_with(&wide(1));

    assert_eq!(install(&wide(2), d.path()).unwrap(), Outcome::Installed);
    assert_eq!(
        std::fs::read(d.path().join("bootlogo.bmp")).unwrap(),
        wide(2)
    );
    assert_eq!(
        std::fs::read(d.path().join("bootlogo.baseos.bmp")).unwrap(),
        wide(1)
    );

    assert_eq!(install(&wide(3), d.path()).unwrap(), Outcome::Installed);
    assert_eq!(
        std::fs::read(d.path().join("bootlogo.baseos.bmp")).unwrap(),
        wide(1),
        "the original was overwritten by a logo slot had installed"
    );
    assert!(!d.path().join("bootlogo.tmp").exists());
}

#[test]
fn the_same_logo_is_left_alone() {
    let d = partition_with(&wide(2));
    assert_eq!(install(&wide(2), d.path()).unwrap(), Outcome::Same);
    assert!(!d.path().join("bootlogo.baseos.bmp").exists());
}

#[test]
fn a_bad_logo_never_touches_the_partition() {
    let d = partition_with(&wide(1));
    assert_eq!(
        install(&wide(2)[..500], d.path()).unwrap(),
        Outcome::Invalid
    );
    assert_eq!(
        std::fs::read(d.path().join("bootlogo.bmp")).unwrap(),
        wide(1)
    );
    assert!(!d.path().join("bootlogo.baseos.bmp").exists());
}

#[test]
fn a_logo_the_wrong_size_for_the_panel_never_touches_the_partition() {
    let d = partition_with(&bmp(640, 480, 1));
    assert_eq!(install(&wide(2), d.path()).unwrap(), Outcome::WrongSize);
    assert_eq!(
        std::fs::read(d.path().join("bootlogo.bmp")).unwrap(),
        bmp(640, 480, 1)
    );
    assert!(!d.path().join("bootlogo.baseos.bmp").exists());

    assert_eq!(
        install(&bmp(640, 480, 2), d.path()).unwrap(),
        Outcome::Installed
    );
}

#[test]
fn the_panel_is_sized_by_the_original_before_whatever_slot_installed() {
    let d = partition_with(&bmp(640, 480, 1));
    assert_eq!(panel_size(d.path()), Some((640, 480)));
    std::fs::write(d.path().join("bootlogo.baseos.bmp"), bmp(720, 720, 1)).unwrap();
    assert_eq!(panel_size(d.path()), Some((720, 720)));
    assert_eq!(panel_size(tempfile::tempdir().unwrap().path()), None);
}

#[test]
fn an_empty_partition_takes_no_logo() {
    let d = tempfile::tempdir().unwrap();
    assert_eq!(install(&wide(2), d.path()).unwrap(), Outcome::WrongSize);
    assert!(!d.path().join("bootlogo.bmp").exists());
}

#[test]
fn the_card_logo_is_picked_by_the_panel_size() {
    let system = tempfile::tempdir().unwrap();
    let s = system.path();
    std::fs::write(s.join("bootlogo-640x480.bmp"), bmp(640, 480, 0)).unwrap();
    std::fs::write(s.join("bootlogo-720x480.bmp"), wide(0)).unwrap();

    assert_eq!(choose(s, (640, 480)), Some(s.join("bootlogo-640x480.bmp")));
    assert_eq!(choose(s, (720, 480)), Some(s.join("bootlogo-720x480.bmp")));
    assert_eq!(choose(s, (720, 720)), None);
}

#[test]
fn the_unsized_logo_from_before_is_used_only_where_it_fits() {
    let system = tempfile::tempdir().unwrap();
    let s = system.path();
    std::fs::write(s.join("bootlogo.bmp"), wide(0)).unwrap();

    assert_eq!(choose(s, (720, 480)), Some(s.join("bootlogo.bmp")));
    assert_eq!(choose(s, (640, 480)), None);
}
