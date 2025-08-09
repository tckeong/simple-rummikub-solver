use crate::game::{tile::Tile, Game, TilesType};
use std::collections::HashMap;

pub struct Solver {
    game: Game,
}

impl Solver {
    pub fn new(game: Game) -> Self {
        Solver { game }
    }

    fn search(&self, tiles: Vec<Tile>, solution_set: Vec<Vec<Tile>>) -> Option<Vec<Vec<Tile>>> {
        if tiles.is_empty() {
            return Some(solution_set);
        }

        let available_pure_color_tiles_sets = self.get_available_pure_color_tiles_sets(&tiles).0;
        let available_mixed_color_tiles_sets = self.get_available_mixed_color_tiles_sets(&tiles).0;

        let available_tiles_sets = available_pure_color_tiles_sets
            .iter()
            .chain(available_mixed_color_tiles_sets.iter())
            .cloned()
            .collect::<Vec<Vec<Tile>>>();

        for available_tiles in available_tiles_sets {
            let mut tiles = tiles.clone();
            let n = tiles.len();

            for available_tile in &available_tiles {
                let mut index = 0;

                for i in 0..n {
                    if tiles[i] == *available_tile {
                        index = i;
                        break;
                    }
                }

                tiles.remove(index);
            }

            let mut solution_set = solution_set.clone();
            solution_set.push(available_tiles);

            if let Some(solution) = self.search(tiles, solution_set.clone()) {
                return Some(solution);
            }
        }

        None
    }

    pub fn solve(&self) -> Option<Vec<Vec<Tile>>> {
        let tiles = self
            .game
            .get_board()
            .iter()
            .flat_map(|v| v.iter())
            .cloned()
            .collect::<Vec<_>>();

        self.search(tiles, vec![])
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
            if tiles[i].color == tiles[i - 1].color && tiles[i].number == tiles[i - 1].number + 1 {
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
            if tile.is_wildcard {
                continue;
            }

            let index = tile.number as usize;
            mixed_colors_tiles_set[index].push(tile.clone());
        }

        let is_valid_mixed_colors_tiles = |t: &Vec<Tile>| -> bool {
            ((t.len() % 3 == 0) && Game::get_colors_count(t) == 3)
                || (t.len() >= 4 && Game::get_colors_count(t) == 4)
        };

        let generate_mixed_colors_tiles_set = |tiles: &Vec<Tile>| -> Vec<Vec<Tile>> {
            if tiles.len() <= 3 {
                vec![tiles.clone()]
            } else {
                let number = tiles[0].number;
                let mut colors_map = HashMap::new();

                for tile in tiles {
                    let color = tile.color;

                    *colors_map.entry(color).or_insert(0) += 1;
                }

                let mut split_tiles1 = Vec::new();
                let mut split_tiles2 = Vec::new();

                for c in colors_map {
                    if c.1 > 1 {
                        split_tiles1.push(Tile::new(number, c.0, false));
                        split_tiles2.push(Tile::new(number, c.0, false));
                    } else {
                        if split_tiles1.len() < split_tiles2.len() {
                            split_tiles1.push(Tile::new(number, c.0, false));
                        } else {
                            split_tiles2.push(Tile::new(number, c.0, false));
                        }
                    }
                }

                vec![split_tiles1, split_tiles2]
            }
        };

        let tiles_set = mixed_colors_tiles_set
            .iter()
            .filter(|t| t.len() >= 3 && is_valid_mixed_colors_tiles(t))
            .flat_map(|t| generate_mixed_colors_tiles_set(t))
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

        if let Some(solution) = solver.solve() {
            println!("{:?}", solution);
        }

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

    #[test]
    fn test3() {
        let game = Game::new();
        let solver = Solver::new(game);

        let tiles = vec![
            Tile::new(10, TileColor::Red, false),
            Tile::new(11, TileColor::Red, false),
            Tile::new(12, TileColor::Red, false),
            Tile::new(1, TileColor::Blue, false),
            Tile::new(1, TileColor::Red, false),
            Tile::new(1, TileColor::Orange, false),
            Tile::new(4, TileColor::Orange, false),
            Tile::new(5, TileColor::Orange, false),
            Tile::new(6, TileColor::Orange, false),
        ];

        println!("{:?}", solver.get_available_pure_color_tiles_sets(&tiles));
        println!("{:?}", solver.get_available_mixed_color_tiles_sets(&tiles));
    }

    #[test]
    fn test4() {
        let game = Game::new();
        let solver = Solver::new(game);

        let tiles = vec![
            Tile::new(1, TileColor::Red, false),
            Tile::new(2, TileColor::Red, false),
            Tile::new(3, TileColor::Red, false),
            Tile::new(3, TileColor::Red, false),
            Tile::new(4, TileColor::Red, false),
            Tile::new(5, TileColor::Red, false),
            Tile::new(6, TileColor::Red, false),
        ];

        println!("{:?}", solver.get_available_pure_color_tiles_sets(&tiles));
        println!("{:?}", solver.get_available_mixed_color_tiles_sets(&tiles));
    }

    #[test]
    fn test5() {
        let game = Game::new();
        let solver = Solver::new(game);

        let tiles = vec![
            Tile::new(1, TileColor::Red, false),
            Tile::new(1, TileColor::Orange, false),
            Tile::new(1, TileColor::Blue, false),
            Tile::new(1, TileColor::Black, false),
            Tile::new(1, TileColor::Red, false),
            Tile::new(1, TileColor::Blue, false),
        ];

        println!(
            "Available pure color tiles sets: {:?}\n",
            solver.get_available_pure_color_tiles_sets(&tiles)
        );
        println!(
            "Available mixed color tiles sets: {:?}\n",
            solver.get_available_mixed_color_tiles_sets(&tiles)
        );
    }

    #[test]
    fn test6() {
        let tiles = vec![
            Tile::new(10, TileColor::Red, false),
            Tile::new(11, TileColor::Red, false),
            Tile::new(12, TileColor::Red, false),
            Tile::new(1, TileColor::Blue, false),
            Tile::new(1, TileColor::Red, false),
            Tile::new(1, TileColor::Orange, false),
            Tile::new(4, TileColor::Orange, false),
            Tile::new(5, TileColor::Orange, false),
            Tile::new(6, TileColor::Orange, false),
        ];

        let game = Game::new();
        let solver = Solver::new(game);

        let solution = solver.search(tiles, Vec::new());

        println!("{:?}", solution);
        assert!(solution.is_some());
    }

    #[test]
    fn test7() {
        let tiles = vec![
            Tile::new(10, TileColor::Red, false),
            Tile::new(11, TileColor::Red, false),
            Tile::new(12, TileColor::Red, false),
            Tile::new(1, TileColor::Blue, false),
            Tile::new(1, TileColor::Red, false),
            Tile::new(1, TileColor::Orange, false),
            Tile::new(4, TileColor::Red, false),
            Tile::new(5, TileColor::Blue, false),
            Tile::new(6, TileColor::Orange, false),
        ];

        let game = Game::new();
        let solver = Solver::new(game);

        let solution = solver.search(tiles, Vec::new());

        println!("{:?}", solution);
        assert!(solution.is_none());
    }
}
