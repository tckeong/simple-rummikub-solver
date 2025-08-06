use super::tile_color::TileColor;
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct Tile {
    pub number: u8,
    pub color: TileColor,
    pub is_wildcard: bool,
}

impl PartialEq for Tile {
    fn eq(&self, other: &Self) -> bool {
        self.number == other.number && self.color == other.color
    }
}

impl Eq for Tile {}

impl Ord for Tile {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.color.cmp(&other.color) {
            Ordering::Equal => self.number.cmp(&other.number),
            ord => ord,
        }
    }
}

impl PartialOrd for Tile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Tile {
    pub(crate) fn new(number: u8, color: TileColor, is_wildcard: bool) -> Self {
        Tile {
            number,
            color,
            is_wildcard,
        }
    }
}
