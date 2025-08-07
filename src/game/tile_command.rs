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
            TileCommandError::InvalidCommand => write!(f, "Invalid command, please key in 'a'/'p'/'d'/'r' only!"),
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
    pub replace_args: Option<Vec<String>>,
    pub replace_tail: Option<String>,
    pub tail: String,
}

impl<'a> TileCommand {
    pub fn validate(&self, game: &Game) -> Result<Self, TileCommandError> {
        let idx = match self.cmd {
            Command::Add | Command::Replace => game
                .validate_index(self.idx)
                .then_some(self.idx)
                .ok_or(TileCommandError::InvalidIndex),
            _ => Ok(self.idx),
        }?;

        let is_color = Ok(self.tail.chars().all(|ch| ch.is_ascii_alphabetic()))?;

        let args = if is_color {
            self.validate_numbers(self.args.clone())
        } else {
            self.validate_colors(self.args.clone())
        }?;

        if self.cmd == Command::Replace {
            let replace_tail = self.replace_tail.clone().ok_or(TileCommandError::Other(
                "Invalid replace command format!".to_string(),
            ))?;

            let replace_is_color = Ok(replace_tail.chars().all(|ch| ch.is_ascii_alphabetic()))?;

            let replace_args = self.replace_args.clone().ok_or(TileCommandError::Other(
                "Invalid replace command format!".to_string(),
            ))?;

            let replace_args = if replace_is_color {
                self.validate_numbers(replace_args)
            } else {
                self.validate_colors(replace_args)
            }?;

            Ok(TileCommand {
                cmd: self.cmd.clone(),
                idx,
                replace_args: Some(replace_args),
                replace_tail: Some(replace_tail),
                args,
                tail: self.tail.clone(),
            })
        } else {
            Ok(TileCommand {
                cmd: self.cmd.clone(),
                idx,
                replace_args: self.replace_args.clone(),
                replace_tail: self.replace_tail.clone(),
                args,
                tail: self.tail.clone(),
            })
        }
    }

    fn validate_numbers(&self, args: Vec<String>) -> Result<Vec<String>, TileCommandError> {
        let test_args = args.clone();
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

        (test_args.len() == n)
            .then_some(self.args.clone())
            .ok_or(TileCommandError::InvalidArgs)
    }

    fn validate_colors(&self, args: Vec<String>) -> Result<Vec<String>, TileCommandError> {
        let test_args = args.clone();
        let n = test_args.len();

        let test_args = test_args
            .iter()
            .filter(|s| s == &&"w" || s == &&"r" || s == &&"b" || s == &&"h" || s == &&"o")
            .collect::<Vec<&String>>();

        (test_args.len() == n)
            .then_some(self.args.clone())
            .ok_or(TileCommandError::InvalidArgs)
    }
}

impl ToTiles for TileCommand {
    fn to_tiles(&self) -> GameOperation {
        let cmd = self.cmd.clone();
        let idx = self.idx;
        let args = self.args.clone();
        let tail = self.tail.clone();
        let tiles_generator = TilesGenerator::new(args);

        let replace_tiles = if cmd == Command::Replace {
            let replace_tail = self.replace_tail.clone().unwrap();
            let replace_args = self.replace_args.clone().unwrap();
            let replace_tiles_generator = TilesGenerator::new(replace_args);

            match replace_tail {
                c if c.chars().all(|ch| ch.is_ascii_alphabetic()) => {
                    let color = match c.chars().nth(0).unwrap() {
                        'r' => TileColor::Red,
                        'b' => TileColor::Blue,
                        'h' => TileColor::Black,
                        'o' => TileColor::Orange,
                        _ => panic!("Invalid color!"),
                    };

                    Some(replace_tiles_generator.generate_numbers_tiles(color))
                }

                c if c.chars().all(|ch| ch.is_ascii_digit()) => {
                    let num = c.parse::<u8>().unwrap_or_else(|_| {
                        panic!("Invalid digit in replace tail: {}", c);
                    });

                    Some(replace_tiles_generator.generate_colors_tiles(num))
                }

                _ => Some(Vec::new()),
            }
        } else {
            None
        };

        let tiles = match tail {
            c if c.chars().all(|ch| ch.is_ascii_alphabetic()) => {
                let color = match c.chars().nth(0).unwrap() {
                    'r' => TileColor::Red,
                    'b' => TileColor::Blue,
                    'h' => TileColor::Black,
                    'o' => TileColor::Orange,
                    _ => panic!("Invalid color!"),
                };

                tiles_generator.generate_numbers_tiles(color)
            }

            c if c.chars().all(|ch| ch.is_ascii_digit()) => {
                let num = c.parse::<u8>().unwrap_or_else(|_| {
                    panic!("Invalid digit in tail: {}", c);
                });

                tiles_generator.generate_colors_tiles(num)
            }

            _ => Vec::new(),
        };

        GameOperation::new(cmd, idx, tiles, replace_tiles)
    }
}

struct TilesGenerator {
    args: Vec<String>,
}

impl TilesGenerator {
    fn new(args: Vec<String>) -> Self {
        TilesGenerator { args }
    }

    fn generate_numbers_tiles(&self, color: TileColor) -> Vec<Tile> {
        let mut tiles = Vec::new();

        for arg in &self.args {
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

        tiles
    }

    fn generate_colors_tiles(&self, num: u8) -> Vec<Tile> {
        let mut tiles = Vec::new();

        for arg in &self.args {
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

        tiles
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
            replace_args: None,
            replace_tail: None,
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
            replace_args: None,
            replace_tail: None,
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

    #[test]
    fn test3() {
        let game = Game::new();
        let command = TileCommand {
            cmd: Command::Replace,
            idx: 0,
            replace_args: Some(vec!["1".to_string()]),
            replace_tail: Some("h".to_string()),
            args: vec!["h".to_string()],
            tail: "10".to_string(),
        };

        assert!(command.validate(&game).is_ok());
    }

    #[test]
    fn test4() {
        let game = Game::new();
        let command = TileCommand {
            cmd: Command::Replace,
            idx: 0,
            replace_args: None,
            replace_tail: None,
            args: vec!["h".to_string()],
            tail: "10".to_string(),
        };

        assert!(command.validate(&game).is_err());
    }
}
