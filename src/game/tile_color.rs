use crossterm::style::Color;
use std::cmp::Ordering;

// h, b, o, r
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileColor {
    Blue,
    Red,
    Orange,
    Black,
}

impl PartialOrd for TileColor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.to_rank().partial_cmp(&other.to_rank())
    }
}

impl Ord for TileColor {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_rank().cmp(&other.to_rank())
    }
}

impl TileColor {
    pub fn as_color(&self) -> Color {
        match self {
            TileColor::Blue => Color::Blue,
            TileColor::Red => Color::Red,
            TileColor::Orange => Color::Rgb {
                r: 232,
                g: 118,
                b: 0,
            },
            TileColor::Black => Color::Black,
        }
    }

    pub fn str_to_tile_color(s: &str) -> Option<Self> {
        match s {
            "b" => Some(TileColor::Blue),
            "r" => Some(TileColor::Red),
            "o" => Some(TileColor::Orange),
            "h" => Some(TileColor::Black),
            _ => None,
        }
    }

    pub fn to_rank(&self) -> u8 {
        match self {
            TileColor::Black => 0,
            TileColor::Blue => 1,
            TileColor::Orange => 2,
            TileColor::Red => 3,
        }
    }
}
