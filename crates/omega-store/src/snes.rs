//! A SNES ROM's internal header. Stub until the SNES work lands: no header is read yet.

use std::path::Path;

use crate::Region;

/// What a SNES ROM's header says about it.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Header {
    pub title: String,
    pub region: Option<Region>,
}

/// The header of the ROM at `path`, if it has one that reads.
pub fn header(_path: &Path) -> Option<Header> {
    None
}

/// Where the game at `path`, named `stem`, comes from: a region in its file name's tags wins
/// over its header.
pub fn region(path: &Path, stem: &str) -> Option<Region> {
    Region::from_tags(stem).or_else(|| header(path).and_then(|h| h.region))
}
