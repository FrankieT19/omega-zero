//! PlayStation content: discs, playlists and the serial a disc carries. The serial is a stub
//! until the PS1 work lands.

use std::path::{Path, PathBuf};

use crate::Region;

/// The disc images the game at `path` is made of, in order: one for a lone disc, every entry of
/// an `.m3u` playlist, each taken from the playlist's own folder unless it is a whole path. A
/// playlist that lists nothing stands for no disc.
pub fn discs(path: &Path) -> Vec<PathBuf> {
    let is_m3u = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("m3u"));
    if !is_m3u {
        return vec![path.to_path_buf()];
    }
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let dir = path.parent().unwrap_or(Path::new(""));
    text.lines()
        .map(|line| line.trim().trim_start_matches('\u{feff}'))
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| dir.join(line))
        .collect()
}

/// Where the label of one disc is kept: `Labels/PS1/<disc's own name>.png`, the way a lone
/// disc's label is kept under its own name.
pub fn disc_label(root: &Path, disc: &Path) -> PathBuf {
    let stem = disc
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    root.join("Labels").join("PS1").join(format!("{stem}.png"))
}

/// The serial printed on the disc at `path`, such as `SLUS-00594`, if it can be read.
pub fn serial(_path: &Path) -> Option<String> {
    None
}

/// Where the game at `path`, named `stem`, comes from: a region in its file name's tags wins
/// over its serial.
pub fn region(_path: &Path, stem: &str) -> Option<Region> {
    Region::from_tags(stem)
}
