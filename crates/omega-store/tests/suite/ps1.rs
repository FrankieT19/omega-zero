use omega_store::ps1::{disc_label, discs};
use std::path::{Path, PathBuf};
use tempfile::tempdir;

#[test]
fn a_lone_disc_is_its_own_only_disc() {
    let disc = Path::new("Games/PS1/Vagrant Story (USA).cue");
    assert_eq!(discs(disc), vec![disc.to_path_buf()]);
}

#[test]
fn a_playlist_is_its_discs_in_order_from_its_own_folder() {
    let d = tempdir().unwrap();
    let m3u = d.path().join("Final Fantasy VII (USA).m3u");
    std::fs::write(
        &m3u,
        "\u{feff}# made by hand\nFinal Fantasy VII (USA) (Disc 1).cue\r\n\nFinal Fantasy VII (USA) (Disc 2).cue\nFinal Fantasy VII (USA) (Disc 3).cue\n",
    )
    .unwrap();
    let names: Vec<PathBuf> = discs(&m3u)
        .iter()
        .map(|p| p.strip_prefix(d.path()).unwrap().to_path_buf())
        .collect();
    assert_eq!(
        names,
        vec![
            PathBuf::from("Final Fantasy VII (USA) (Disc 1).cue"),
            PathBuf::from("Final Fantasy VII (USA) (Disc 2).cue"),
            PathBuf::from("Final Fantasy VII (USA) (Disc 3).cue"),
        ]
    );
}

#[test]
fn a_playlist_that_cannot_be_read_has_no_discs() {
    assert!(discs(Path::new("/nowhere/Missing.m3u")).is_empty());
}

#[test]
fn each_disc_keeps_its_label_under_its_own_name() {
    assert_eq!(
        disc_label(
            Path::new("/sd"),
            Path::new("/sd/Games/PS1/Final Fantasy VII (USA) (Disc 2).cue")
        ),
        PathBuf::from("/sd/Labels/PS1/Final Fantasy VII (USA) (Disc 2).png")
    );
}
