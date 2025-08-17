use super::tile_color::TileColor;
use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone, Hash)]
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

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let is_wildcard = if self.is_wildcard { "w" } else { "n" };

        write!(f, "{}({}){}", self.color, self.number, is_wildcard)
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

    pub fn iter() -> impl Iterator<Item = Tile> {
        let mut tiles = vec![];

        for color in TileColor::iter() {
            for number in 1..=13 {
                tiles.push(Tile::new(number, color, true));
            }
        }

        tiles.into_iter()
    }
}
