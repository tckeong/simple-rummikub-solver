use crate::game::tile_color::TileColor;
use crate::game::{tile::Tile, Command, GameOperation};

use super::{Game, ToTiles};
use std::error::Error;
use std::fmt::{self, Display};

#[derive(Debug, Clone)]
pub(crate) enum TileCommandError {
    InvalidCommand,
    InvalidIndex,
    InvalidArgs,
    InvalidTail,
    Other(String),
}

impl Display for TileCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TileCommandError::InvalidCommand => write!(f, "Invalid command, please key in 'a'/'p'/'d' only!"),
            TileCommandError::InvalidIndex => write!(f, "Invalid index!"),
            TileCommandError::InvalidArgs => write!(f, "Invalid args, please key in only 1-13 or the color r/b/h/o only, and split by ',' !"),
            TileCommandError::InvalidTail => write!(f, "Invalid tail, please key in only 1-13, or the color r/b/h/o only!"),
            TileCommandError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for TileCommandError {}

#[derive(Debug)]
pub(crate) struct TileCommand {
    pub cmd: Command,
    pub idx: usize,
    pub args: Vec<String>,
    pub tail: String,
}

impl<'a> TileCommand {
    pub fn validate(&self, game: &Game) -> Result<Self, TileCommandError> {
        let idx = match self.cmd {
            Command::Add => game
                .validate_index(self.idx)
                .then_some(self.idx)
                .ok_or(TileCommandError::InvalidIndex),
            _ => Ok(self.idx),
        }?;

        let is_color = Ok(self.tail.chars().all(|ch| ch.is_ascii_alphabetic()))?;

        let args = if is_color {
            let test_args = self.args.clone();
            let n = test_args.len();

            let test_args = test_args
                .iter()
                .filter(|s| s == &&"w" || s.parse::<u8>().is_ok())
                .filter(|s| {
                    let d = if s == &&"w" {
                        251
                    } else {
                        s.parse::<u8>().unwrap_or(0)
                    };

                    d == 251 || (d >= 1 && d <= 13)
                })
                .collect::<Vec<&String>>();

            if test_args.len() == n {
                Ok(self.args.clone())
            } else {
                Err(TileCommandError::InvalidArgs)
            }
        } else {
            let test_args = self.args.clone();
            let n = test_args.len();

            let test_args = test_args
                .iter()
                .filter(|s| s == &&"w" || s == &&"r" || s == &&"b" || s == &&"h" || s == &&"o")
                .collect::<Vec<&String>>();

            if test_args.len() == n {
                Ok(self.args.clone())
            } else {
                Err(TileCommandError::InvalidArgs)
            }
        }?;

        Ok(TileCommand {
            cmd: self.cmd.clone(),
            idx,
            args,
            tail: self.tail.clone(),
        })
    }
}

impl ToTiles for TileCommand {
    fn to_tiles(&self) -> GameOperation {
        let mut tiles = Vec::new();
        let cmd = self.cmd.clone();
        let idx = self.idx;
        let args = self.args.clone();
        let tail = self.tail.clone();

        match tail {
            c if c.chars().all(|ch| ch.is_ascii_alphabetic()) => {
                let color = match c.chars().nth(0).unwrap() {
                    'r' => TileColor::Red,
                    'b' => TileColor::Blue,
                    'h' => TileColor::Black,
                    'o' => TileColor::Orange,
                    _ => panic!("Invalid color!"),
                };

                for arg in args {
                    let num = arg.parse::<u8>().unwrap_or_else(|_| {
                        (arg == "w").then_some(251).unwrap_or_else(|| {
                            panic!("Invalid number in args: {}", arg);
                        })
                    });

                    let is_wildcard = arg == "w";
                    let num = is_wildcard.then_some(251).unwrap_or(num);
                    let color = is_wildcard.then_some(TileColor::Red).unwrap_or(color);

                    tiles.push(Tile::new(num, color, is_wildcard));
                }
            }

            c if c.chars().all(|ch| ch.is_ascii_digit()) => {
                let num = c.parse::<u8>().unwrap_or_else(|_| {
                    panic!("Invalid digit in tail: {}", c);
                });

                for arg in args {
                    let color = match arg.as_str() {
                        "r" => TileColor::Red,
                        "b" => TileColor::Blue,
                        "h" => TileColor::Black,
                        "o" => TileColor::Orange,
                        "w" => TileColor::Red,
                        _ => panic!("Invalid color in args: {}", arg),
                    };

                    let is_wildcard = arg == "w";
                    let num = is_wildcard.then_some(251).unwrap_or(num);
                    let color = is_wildcard.then_some(TileColor::Red).unwrap_or(color);

                    tiles.push(Tile::new(num, color, is_wildcard));
                }
            }
            _ => {}
        }

        GameOperation::new(cmd, idx, tiles)
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test1() {
        let game = Game::new();
        let command = TileCommand {
            cmd: Command::Put,
            idx: usize::MAX,
            args: vec![String::from("10"), String::from("11"), String::from("12")],
            tail: "r".to_string(),
        };

        assert!(command.validate(&game).is_ok());
    }

    #[test]
    fn test2() {
        let game = Game::new();
        let command = TileCommand {
            cmd: Command::Add,
            idx: 0,
            args: vec![
                String::from("r"),
                String::from("b"),
                String::from("h"),
                String::from("o"),
            ],
            tail: "10".to_string(),
        };

        assert!(command.validate(&game).is_ok());
    }
}
