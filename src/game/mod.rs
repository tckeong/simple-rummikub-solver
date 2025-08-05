pub(crate) mod parser;
pub(crate) mod tile;
pub(crate) mod tile_color;
pub(crate) mod tile_command;

use std::vec;

use tile::Tile;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Command {
    Add,
    Put,
    Draw,
}

#[derive(Debug)]
pub struct GameOperation {
    pub command: Command,
    pub index: usize,
    pub tiles: Vec<Tile>,
}

impl GameOperation {
    pub fn new(command: Command, index: usize, tiles: Vec<Tile>) -> Self {
        GameOperation {
            command,
            index,
            tiles,
        }
    }
}

pub trait ToTiles {
    fn to_tiles(&self) -> GameOperation;
}

pub(crate) struct Game {
    pub board: Vec<Vec<Tile>>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            board: vec![vec![]],
        }
    }

    pub fn validate_index(&self, idx: usize) -> bool {
        idx < self.board.len()
    }

    pub fn get_board(&self) -> Vec<Vec<Tile>> {
        self.board.clone()
    }

    pub(crate) fn operate(&mut self, mut operation: GameOperation) {
        match operation.command {
            Command::Put => {
                operation.tiles.sort_unstable();
                self.board.push(operation.tiles);
            }
            _ => {
                self.board[operation.index].extend(operation.tiles);
                self.board[operation.index].sort_unstable();
                self.check_and_split(operation.index);
            }
        }
    }

    fn check_and_split(&mut self, index: usize) {
        let tiles = &self.board[index];
        let n = tiles.len();

        if n < 6 {
            return;
        }

        let mut last_repeat = 0;

        for i in 1..n {
            if tiles[i] == tiles[i - 1] {
                last_repeat = i;
            }
        }

        let mut split_tiles1 = Vec::new();
        let mut split_tiles2 = Vec::new();

        split_tiles1.push(tiles[0].clone());

        for i in 1..n {
            if i >= last_repeat || tiles[i] == tiles[i - 1] {
                split_tiles2.push(tiles[i].clone());
            } else {
                split_tiles1.push(tiles[i].clone());
            }
        }

        if !split_tiles2.is_empty() {
            self.board[index] = split_tiles1;
            self.board.push(split_tiles2);
        }
    }

    pub fn reset(&mut self) {
        self.board = vec![vec![]];
    }
}
