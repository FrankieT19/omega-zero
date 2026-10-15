//! PlayStation content: discs, playlists and the serial a disc carries. Stub until the PS1 work
//! lands: every game is one disc and no serial is read yet.

use std::path::{Path, PathBuf};

use crate::Region;

/// The disc images the game at `path` is made of, in order: one for a lone disc, every entry of
/// an `.m3u` playlist.
pub fn discs(path: &Path) -> Vec<PathBuf> {
    vec![path.to_path_buf()]
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
