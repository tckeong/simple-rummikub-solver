pub(crate) mod parser;
pub(crate) mod tile;
pub(crate) mod tile_color;
pub(crate) mod tile_command;

use std::collections::{HashMap, HashSet};
use std::vec;

use tile::Tile;

use crate::game::tile_color::TileColor;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Command {
    Add,
    Put,
    Draw,
    Replace,
}

#[derive(Debug)]
pub struct GameOperation {
    pub command: Command,
    pub index: usize,
    pub replace_tiles: Option<Vec<Tile>>,
    pub tiles: Vec<Tile>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TilesType {
    PureColor,
    MixedColor,
}

impl GameOperation {
    pub fn new(
        command: Command,
        index: usize,
        tiles: Vec<Tile>,
        replace_tiles: Option<Vec<Tile>>,
    ) -> Self {
        GameOperation {
            command,
            index,
            replace_tiles,
            tiles,
        }
    }
}

pub trait ToTiles {
    fn to_tiles(&self) -> GameOperation;
}

#[derive(Debug, Clone)]
pub struct TileSetInfo {
    pub tiles_type: TilesType,
    pub start: u8,
    pub end: u8,
    pub color: Option<TileColor>,
}

impl TileSetInfo {
    fn new(tiles_type: TilesType, start: u8, end: u8, color: Option<TileColor>) -> Self {
        TileSetInfo {
            tiles_type,
            start,
            end,
            color,
        }
    }
}

pub(crate) struct Game {
    pub board: Vec<Vec<Tile>>,
    pub tiles_set_info: Vec<TileSetInfo>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            board: vec![vec![]],
            tiles_set_info: vec![TileSetInfo::new(TilesType::MixedColor, 1, 13, None)],
        }
    }

    pub fn validate_index(&self, idx: usize) -> bool {
        idx < self.board.len()
    }

    pub fn get_board(&self) -> Vec<Vec<Tile>> {
        self.board.clone()
    }

    pub fn get_tiles_set_info(&self) -> Vec<TileSetInfo> {
        self.tiles_set_info.clone()
    }

    pub(crate) fn operate(&mut self, mut operation: GameOperation) {
        match operation.command {
            Command::Put => {
                operation.tiles.sort_unstable();

                self.push_tiles_set_info(&operation.tiles);
                self.board.push(operation.tiles);
            }

            Command::Replace => {
                let wildcard_count = Self::wildcard_count(&self.board[operation.index]);
                let replace_tiles = operation.replace_tiles.unwrap();

                if wildcard_count != replace_tiles.len() {
                    return;
                }

                let tiles_to_replace = self.board[operation.index].clone();
                self.board[operation.index] =
                    self.replace_wildcards(tiles_to_replace, replace_tiles);
                self.set_tiles_set_info(operation.index);

                let mut tiles = operation.tiles;
                tiles.extend(vec![Tile::new(251, TileColor::Red, true); wildcard_count]);

                let tiles_set = self.wildcard_to_tiles(tiles);

                for tiles in tiles_set {
                    self.push_tiles_set_info(&tiles);
                    self.board.push(tiles);
                }
            }

            _ => {
                self.board[operation.index].extend(operation.tiles);
                self.board[operation.index].sort_unstable();

                if operation.index != 0 {
                    let tiles = self.board[operation.index].clone();
                    let tiles = self.wildcard_to_tiles(tiles);

                    self.board[operation.index] = tiles[0].clone();
                    self.set_tiles_set_info(operation.index);

                    if tiles.len() > 1 {
                        self.push_tiles_set_info(&tiles[1]);
                        self.board.push(tiles[1].clone());
                    }
                }
            }
        }
    }

    fn check_and_split(&self, tiles: Vec<Tile>) -> Vec<Vec<Tile>> {
        let n = tiles.len();

        if n < 6 {
            return vec![tiles];
        }

        let tiles_type = Self::get_tiles_type(&tiles);

        match tiles_type {
            TilesType::PureColor => self.split_pure_color_tiles(tiles),
            TilesType::MixedColor => self.split_mixed_colors_tiles(tiles),
        }
    }

    fn split_pure_color_tiles(&self, tiles: Vec<Tile>) -> Vec<Vec<Tile>> {
        let n = tiles.len();
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

        vec![split_tiles1, split_tiles2]
    }

    fn split_mixed_colors_tiles(&self, tiles: Vec<Tile>) -> Vec<Vec<Tile>> {
        let n = tiles.len();
        let mut colors_map = HashMap::new();
        let mut numbers_map = HashMap::new();

        for i in 0..n {
            let tile = &tiles[i];
            let color = tile.color;
            let number = tile.number;

            colors_map.entry(color).or_insert(vec![]).push(i);
            numbers_map.entry(number).or_insert(vec![]).push(i);
        }

        let max_color_count = colors_map
            .iter()
            .max_by_key(|(_, v)| v.len())
            .unwrap()
            .1
            .len();

        if max_color_count <= 2 {
            // case multiple sets of same number mixed colors tiles
            let mut split_tiles1 = Vec::new();
            let mut split_tiles2 = Vec::new();

            // assume user provide tiles are correct, all tiles are provide in same number
            for (_, v) in colors_map {
                if v.len() > 1 {
                    split_tiles1.push(tiles[v[0]].clone());
                    split_tiles2.push(tiles[v[1]].clone());
                } else {
                    if split_tiles1.len() < split_tiles2.len() {
                        split_tiles1.push(tiles[v[0]].clone());
                    } else {
                        split_tiles2.push(tiles[v[0]].clone());
                    }
                }
            }

            vec![split_tiles1, split_tiles2]
        } else {
            // case multiple sets of different number pure color tiles + mixed colors tiles
            let mixed_colors_number = numbers_map.iter().max_by_key(|(_, v)| v.len()).unwrap().0;

            let split_tiles1 = numbers_map
                .get(mixed_colors_number)
                .unwrap()
                .iter()
                .map(|&i| tiles[i].clone())
                .collect::<Vec<_>>();

            let mut split_tiles2 = Vec::new();

            for tile in tiles {
                if tile.number != *mixed_colors_number {
                    split_tiles2.push(tile);
                }
            }

            vec![split_tiles1, split_tiles2]
        }
    }

    pub fn reset(&mut self) {
        self.board = vec![vec![]];
    }

    pub fn wildcard_count(tiles: &Vec<Tile>) -> usize {
        let mut count = 0;

        for tile in tiles {
            if tile.is_wildcard {
                count += 1;
            }
        }

        count
    }

    fn wildcard_to_tiles(&mut self, tiles: Vec<Tile>) -> Vec<Vec<Tile>> {
        let wildcard_count = Self::wildcard_count(&tiles);

        if wildcard_count == 0 {
            return self.check_and_split(tiles);
        }

        let tiles = tiles
            .clone()
            .into_iter()
            .filter(|tile| !tile.is_wildcard)
            .collect::<Vec<_>>();
        let tiles_type = Self::get_tiles_type(&tiles);

        match tiles_type {
            TilesType::PureColor => {
                let color = tiles[0].color;

                let low = tiles.iter().min_by_key(|tile| tile.number).unwrap().number;
                let high = tiles.iter().max_by_key(|tile| tile.number).unwrap().number;

                let low = 1.max(low - 1);
                let high = 13.min(high + 1);

                let mut tiles_set = Vec::new();

                // max 2 wildcards
                if wildcard_count == 1 {
                    for num in low..=high {
                        let mut tiles = tiles.clone();
                        let tile = Tile::new(num, color, true);
                        tiles.push(tile);
                        tiles.sort_unstable();

                        tiles_set = self.check_and_split(tiles);

                        if self.is_valid_pure_color_tiles(&tiles_set) {
                            break;
                        }
                    }
                } else if wildcard_count == 2 {
                    for num1 in low..=high {
                        for num2 in (low - 1)..=(high + 1) {
                            let mut tiles = tiles.clone();
                            let tile1 = Tile::new(num1, color, true);
                            let tile2 = Tile::new(num2, color, true);
                            tiles.push(tile1);
                            tiles.push(tile2);
                            tiles.sort_unstable();

                            tiles_set = self.check_and_split(tiles);

                            if self.is_valid_pure_color_tiles(&tiles_set) {
                                break;
                            }
                        }
                    }
                }

                tiles_set
            }

            TilesType::MixedColor => {
                let number = tiles[0].number;

                let mut tiles_set = Vec::new();

                // max 2 wildcards
                if wildcard_count == 1 {
                    for color in TileColor::iter() {
                        let mut tiles = tiles.clone();
                        let tile = Tile::new(number, color, true);
                        tiles.push(tile);
                        tiles.sort_unstable();

                        tiles_set = self.check_and_split(tiles);

                        if self.is_valid_mixed_color_tiles(&tiles_set) {
                            break;
                        }
                    }
                } else if wildcard_count == 2 {
                    for color1 in TileColor::iter() {
                        for color2 in TileColor::iter() {
                            let mut tiles = tiles.clone();

                            let tile1 = Tile::new(number, color1, true);
                            let tile2 = Tile::new(number, color2, true);
                            tiles.push(tile1);
                            tiles.push(tile2);
                            tiles.sort_unstable();

                            tiles_set = self.check_and_split(tiles);

                            if self.is_valid_mixed_color_tiles(&tiles_set) {
                                break;
                            }
                        }
                    }
                }

                tiles_set
            }
        }
    }

    fn replace_wildcards(&self, tiles: Vec<Tile>, replace_tiles: Vec<Tile>) -> Vec<Tile> {
        let mut tiles = tiles;

        for replace_tile in replace_tiles {
            for i in 0..tiles.len() {
                let tile = &mut tiles[i];

                if tile.is_wildcard {
                    tile.is_wildcard = false;
                    tile.number = replace_tile.number;
                    tile.color = replace_tile.color;
                    break;
                }
            }
        }

        tiles.sort_unstable();
        tiles
    }

    pub fn get_colors_count(tiles: &Vec<Tile>) -> usize {
        let mut colors = HashSet::new();

        for tile in tiles {
            if tile.is_wildcard {
                continue;
            }

            colors.insert(tile.color);
        }

        colors.len()
    }

    pub fn get_tiles_type(tiles: &Vec<Tile>) -> TilesType {
        if Self::get_colors_count(tiles) == 1 {
            TilesType::PureColor
        } else {
            TilesType::MixedColor
        }
    }

    fn get_range(&self, tiles: &Vec<Tile>) -> (u8, u8) {
        let mut start = u8::MAX;
        let mut end = u8::MIN;

        for tile in tiles {
            if tile.is_wildcard {
                continue;
            }

            start = start.min(tile.number);
            end = end.max(tile.number);
        }

        (start, end)
    }

    fn set_tiles_set_info(&mut self, index: usize) {
        let tiles = &self.board[index];
        let tiles_type = Self::get_tiles_type(tiles);
        let (start, end) = self.get_range(tiles);
        let color = (tiles_type == TilesType::PureColor).then_some(tiles[0].color);

        self.tiles_set_info[index] = TileSetInfo::new(tiles_type, start, end, color);
    }

    fn push_tiles_set_info(&mut self, tiles: &Vec<Tile>) {
        let tiles_type = Self::get_tiles_type(tiles);
        let (start, end) = self.get_range(tiles);
        let color = (tiles_type == TilesType::PureColor).then_some(tiles[0].color);

        self.tiles_set_info
            .push(TileSetInfo::new(tiles_type, start, end, color));
    }

    fn is_valid_pure_color_tiles(&self, tiles_set: &Vec<Vec<Tile>>) -> bool {
        let mut is_valid = true;

        for tiles in tiles_set {
            let mut value = tiles.iter().min_by_key(|tile| tile.number).unwrap().number;

            if tiles.len() < 3 {
                is_valid = false;
                break;
            }

            for tile in tiles {
                if tile.number == value {
                    value += 1;
                } else {
                    is_valid = false;
                    break;
                }
            }
        }

        is_valid
    }

    fn is_valid_mixed_color_tiles(&self, tiles_set: &Vec<Vec<Tile>>) -> bool {
        let mut is_valid = true;

        for tiles in tiles_set {
            let mut colors = HashSet::new();

            for tile in tiles {
                colors.insert(tile.color);
            }

            if colors.len() < 3 {
                is_valid = false;
                break;
            }
        }

        is_valid
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test1() {
        let tile1 = Tile::new(1, TileColor::Red, true);
        let tile2 = Tile::new(1, TileColor::Red, false);

        assert!(tile1 == tile2);
    }

    #[test]
    fn test2() {
        let game = Game::new();
        let tiles1 = vec![
            Tile::new(1, TileColor::Red, false),
            Tile::new(2, TileColor::Red, true),
            Tile::new(3, TileColor::Red, false),
        ];
        let tiles2 = vec![Tile::new(2, TileColor::Red, false)];

        let tiles = game.replace_wildcards(tiles1, tiles2);

        println!("{:?}", tiles);

        assert_eq!(
            tiles,
            vec![
                Tile::new(1, TileColor::Red, false),
                Tile::new(2, TileColor::Red, false),
                Tile::new(3, TileColor::Red, false),
            ]
        );
    }

    #[test]
    fn test3() {
        let mut game = Game::new();
        game.operate(GameOperation::new(
            Command::Add,
            0,
            vec![Tile::new(10, TileColor::Red, false)],
            None,
        ));

        let tiles1 = vec![
            Tile::new(10, TileColor::Black, false),
            Tile::new(11, TileColor::Black, false),
            Tile::new(12, TileColor::Black, false),
        ];

        let tiles2 = vec![
            Tile::new(9, TileColor::Black, false),
            Tile::new(251, TileColor::Red, true),
            Tile::new(13, TileColor::Black, false),
        ];

        let tiles3 = vec![
            Tile::new(1, TileColor::Red, false),
            Tile::new(1, TileColor::Blue, false),
        ];

        let replace_tiles = vec![Tile::new(11, TileColor::Black, false)];

        game.operate(GameOperation::new(Command::Put, usize::MAX, tiles1, None));

        game.operate(GameOperation::new(Command::Add, 1, tiles2, None));

        game.operate(GameOperation::new(
            Command::Replace,
            2,
            tiles3,
            Some(replace_tiles),
        ));

        let board = game.get_board();
        let tiles_info = game.get_tiles_set_info();

        println!("{:?}", board);
        println!("{:?}", tiles_info);

        assert_eq!(board.len(), tiles_info.len())
    }
}
