use crate::game::{tile::Tile, Game, TilesType};

pub struct Solver {
    game: Game,
}

impl Solver {
    pub fn new(game: Game) -> Self {
        Solver { game }
    }

    pub fn solve(&self) -> Option<Vec<Vec<Tile>>> {
        let board = self.game.get_board();

        let user_tiles = board[0].clone();
        let mut board = board[1..].iter().cloned().collect::<Vec<_>>();

        let (available_tiles_set, user_tiles) = self.pick_available(&user_tiles);

        for available_tiles in available_tiles_set {
            board.push(available_tiles);
        }

        for tile in user_tiles {
            let index = self.try_fill_board(&board, tile.clone());

            if let Some(index) = index {
                let mut tiles = board[index].clone();
                tiles.push(tile);
                tiles.sort_unstable();

                let tiles_set = Game::check_and_split(tiles);

                board[index] = tiles_set[0].clone();

                if tiles_set.len() > 1 {
                    board.push(tiles_set[1].clone());
                }
            } else {
                return None;
            }
        }

        Some(board)
    }

    fn pick_available(&self, user_tiles: &Vec<Tile>) -> (Vec<Vec<Tile>>, Vec<Tile>) {
        let mut tiles = user_tiles.clone();
        tiles.sort_unstable();

        let (available_pure_color_tiles_sets, leave_tiles) =
            self.get_available_pure_color_tiles_sets(&tiles);
        let (available_mixed_color_tiles_sets, leave_tiles) =
            self.get_available_mixed_color_tiles_sets(&leave_tiles);

        let available_tiles = vec![
            available_pure_color_tiles_sets,
            available_mixed_color_tiles_sets,
        ];

        let available_tiles = available_tiles
            .iter()
            .flat_map(|v| v.iter())
            .cloned()
            .collect();

        (available_tiles, leave_tiles)
    }

    fn get_available_pure_color_tiles_sets(
        &self,
        tiles: &Vec<Tile>,
    ) -> (Vec<Vec<Tile>>, Vec<Tile>) {
        let mut tiles_set = Vec::new();
        let mut leave_tiles = Vec::new();
        let n = tiles.len();
        let mut cur_tiles = vec![tiles[0].clone()];

        for i in 1..n {
            if tiles[i].number == tiles[i - 1].number + 1 {
                cur_tiles.push(tiles[i].clone());
            } else {
                if cur_tiles.len() >= 3 {
                    tiles_set.push(cur_tiles.clone());
                } else {
                    leave_tiles.extend(cur_tiles.clone());
                }

                cur_tiles = vec![tiles[i].clone()];
            }
        }

        if cur_tiles.len() >= 3 {
            tiles_set.push(cur_tiles.clone());
        } else {
            leave_tiles.extend(cur_tiles.clone());
        }

        (tiles_set, leave_tiles)
    }

    fn get_available_mixed_color_tiles_sets(
        &self,
        tiles: &Vec<Tile>,
    ) -> (Vec<Vec<Tile>>, Vec<Tile>) {
        let mut mixed_colors_tiles_set = vec![vec![]; 14];

        for tile in tiles {
            let index = tile.number as usize;
            mixed_colors_tiles_set[index].push(tile.clone());
        }

        let is_valid_mixed_colors_tiles = |t: &Vec<Tile>| -> bool {
            t.len() % 3 == Game::get_colors_count(t) || t.len() % 4 == Game::get_colors_count(t)
        };

        let tiles_set = mixed_colors_tiles_set
            .iter()
            .filter(|t| t.len() >= 3 && is_valid_mixed_colors_tiles(t))
            .cloned()
            .collect::<Vec<Vec<Tile>>>();

        let leave_tiles = mixed_colors_tiles_set
            .iter()
            .filter(|t| t.len() < 3 || !is_valid_mixed_colors_tiles(t))
            .flat_map(|t| t.clone())
            .collect::<Vec<Tile>>();

        (tiles_set, leave_tiles)
    }

    fn try_fill_board(&self, board: &Vec<Vec<Tile>>, tile: Tile) -> Option<usize> {
        let n = board.len();

        let same_color = |tiles: &Vec<Tile>, tile: &Tile| -> bool {
            Game::get_tiles_type(tiles) == TilesType::PureColor && tiles[0].color == tile.color
        };

        let same_number = |tiles: &Vec<Tile>, tile: &Tile| -> bool {
            Game::get_tiles_type(tiles) == TilesType::MixedColor && tiles[0].number == tile.number
        };

        for i in 0..n {
            let tiles = &board[i];
            if same_color(tiles, &tile) {
                if self.check_pure_color_valid(tiles, &tile) {
                    return Some(i);
                }
            }

            if same_number(tiles, &tile) {
                if self.check_mixed_color_valid(tiles, &tile) {
                    return Some(i);
                }
            }
        }

        None
    }

    fn check_pure_color_valid(&self, tiles: &Vec<Tile>, tile: &Tile) -> bool {
        let mut tiles = tiles.clone();
        tiles.push(tile.clone());
        tiles.sort_unstable();

        Game::is_valid_pure_color_tiles(&vec![tiles])
    }

    fn check_mixed_color_valid(&self, tiles: &Vec<Tile>, tile: &Tile) -> bool {
        let mut tiles = tiles.clone();
        tiles.push(tile.clone());
        tiles.sort_unstable();

        Game::is_valid_mixed_color_tiles(&vec![tiles])
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[allow(unused_imports)]
    use crate::game::{tile_color::TileColor, Command, Game, GameOperation};

    #[test]
    fn test1() {
        let mut game = Game::new();

        game.operate(GameOperation::new(
            Command::Add,
            0,
            vec![
                Tile::new(10, TileColor::Red, false),
                Tile::new(11, TileColor::Red, false),
                Tile::new(12, TileColor::Red, false),
                Tile::new(1, TileColor::Blue, false),
                Tile::new(1, TileColor::Red, false),
                Tile::new(1, TileColor::Orange, false),
                Tile::new(4, TileColor::Blue, false),
                Tile::new(5, TileColor::Black, false),
                Tile::new(6, TileColor::Orange, false),
            ],
            None,
        ));

        let solver = Solver::new(game);
        assert!(solver.solve().is_none());
    }

    #[test]
    fn test2() {
        let mut game = Game::new();

        game.operate(GameOperation::new(
            Command::Add,
            0,
            vec![
                Tile::new(10, TileColor::Red, false),
                Tile::new(11, TileColor::Red, false),
                Tile::new(12, TileColor::Red, false),
                Tile::new(1, TileColor::Blue, false),
                Tile::new(1, TileColor::Red, false),
                Tile::new(1, TileColor::Orange, false),
                Tile::new(4, TileColor::Orange, false),
                Tile::new(5, TileColor::Orange, false),
                Tile::new(6, TileColor::Orange, false),
            ],
            None,
        ));

        let solver = Solver::new(game);
        assert!(solver.solve().is_some());
    }
}
