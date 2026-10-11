//! omega-ui's tests, built as one binary. Built one per file, linking and starting them took
//! longer than running them. A test that needs a process to itself stays a file of its own in
//! tests/, beside this folder: theme sets the slot's palette, which a process sets only once.

mod backdrop;
mod barcode;
mod barcode_scan;
mod battery;
mod bezel;
mod bluetooth;
mod board;
mod cart;
mod chrome;
mod clock;
mod contrast;
mod font;
mod hud;
mod icon;
mod keyboard;
mod kit;
mod kit_hud;
mod kit_layout;
mod label;
mod letters;
mod link_art;
mod network;
mod palette;
mod plate;
mod polaroids;
mod quick_menu;
mod refusal;
mod render_about;
mod render_board;
mod render_gb_cart;
mod render_row_edges;
mod render_shelf_wrap;
mod render_sticker;
mod saves;
mod shelf;
mod shell;
mod spin;
mod sticker;
mod switcher;
mod toast;
mod top_bar;

/// A file in this folder that the list above leaves out is never compiled, let alone run.
#[test]
fn every_file_in_the_folder_is_in_the_suite() {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/suite");
    let listed = include_str!("main.rs");
    for entry in std::fs::read_dir(&folder).expect("the suite's folder") {
        let path = entry.expect("a file in the suite's folder").path();
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if path.extension().is_some_and(|e| e == "rs") && stem != "main" {
            assert!(
                listed.contains(&format!("\nmod {stem};")),
                "tests/suite/{stem}.rs is missing from tests/suite/main.rs"
            );
        }
    }
}
