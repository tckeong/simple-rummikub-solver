use crate::game::{tile::Tile, Game, TilesType};
use std::collections::{HashMap, HashSet};

pub struct Solver {
    game: Game,
}

impl Solver {
    pub fn new(game: Game) -> Self {
        Solver { game }
    }

    fn tiles_to_string(mut tiles: Vec<Tile>) -> String {
        tiles.sort_unstable();
        tiles
            .iter()
            .filter(|t| !t.is_wildcard)
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join("")
    }

    fn search(
        &self,
        tiles: Vec<Tile>,
        solution_set: Vec<Vec<Tile>>,
        cache: &mut HashSet<String>,
    ) -> Option<Vec<Vec<Tile>>> {
        if tiles.is_empty() {
            return Some(solution_set);
        }

        let key = Self::tiles_to_string(tiles.clone());
        if cache.contains(&key) || tiles.len() <= 2 {
            return None;
        }

        let (available_pure_color_tiles, leave_tiles) =
            self.get_available_pure_color_tiles_sets(&tiles);

        if !available_pure_color_tiles.is_empty() && leave_tiles.is_empty() {
            let mut solution_set = solution_set.clone();
            solution_set.extend(available_pure_color_tiles);
            return Some(solution_set);
        }

        let (available_mixed_colors_tiles, leave_tiles) =
            self.get_available_mixed_colors_tiles_sets(&tiles);

        if !available_mixed_colors_tiles.is_empty() && leave_tiles.is_empty() {
            let mut solution_set = solution_set.clone();
            solution_set.extend(available_mixed_colors_tiles);
            return Some(solution_set);
        }

        let n = tiles.len();
        let candidates = available_mixed_colors_tiles
            .into_iter()
            .chain(available_pure_color_tiles);

        for candidate in candidates {
            let mut removed = Vec::new();

            for c in &candidate {
                for i in 0..n {
                    if removed.contains(&i) {
                        continue;
                    }

                    let tile = &tiles[i];

                    if c == tile || (c.is_wildcard && tile.is_wildcard) {
                        removed.push(i);
                        break;
                    }
                }
            }

            let remaining = tiles
                .iter()
                .enumerate()
                .filter(|(i, _)| !removed.contains(i))
                .map(|(_, v)| v.clone())
                .collect::<Vec<_>>();

            let mut next_solution = solution_set.clone();
            next_solution.push(candidate);

            if let Some(solution) = self.search(remaining, next_solution, cache) {
                return Some(solution);
            }
        }

        cache.insert(key);

        None
    }

    pub fn solve(&self) -> Option<Vec<Vec<Tile>>> {
        let board = self.game.get_board();
        let user_tiles = board[0].clone();
        let mut board = board[1..].to_vec();

        let (available_tiles, leave_tiles) = self.pick_available(&user_tiles);

        board.extend(available_tiles);

        let (mut tiles, completed_tiles) = self.pick_relevant_tiles(&board, &leave_tiles);

        tiles.push(leave_tiles);
        let tiles = tiles.iter().flatten().cloned().collect::<Vec<Tile>>();

        if tiles.is_empty() {
            return Some(board);
        }

        let solution = self.search(tiles, Vec::new(), &mut HashSet::new());

        if let Some(mut solution) = solution {
            solution.extend(completed_tiles);
            Some(solution)
        } else {
            None
        }
    }

    fn pick_relevant_tiles(
        &self,
        board: &Vec<Vec<Tile>>,
        tiles: &Vec<Tile>,
    ) -> (Vec<Vec<Tile>>, Vec<Vec<Tile>>) {
        let n = board.len();
        let mut relevant_index = Vec::new();

        for tile in tiles {
            for i in 0..n {
                if relevant_index.contains(&i) {
                    continue;
                }

                if board[i].len() >= 4
                    || (Game::get_tiles_type(&board[i]) == TilesType::MixedColor
                        && board[i][0].number == tile.number)
                    || (Game::get_tiles_type(&board[i]) == TilesType::PureColor
                        && tile.number >= board[i][0].number
                        && tile.number <= board[i].last().unwrap().number
                        && board[i].len() >= 4)
                    || Game::wildcard_count(&board[i]) > 0
                {
                    relevant_index.push(i);
                }
            }
        }

        let tiles = board
            .iter()
            .enumerate()
            .filter(|(i, _)| relevant_index.contains(i))
            .map(|(_, v)| v.clone())
            .collect::<Vec<Vec<Tile>>>();

        let completed_tiles = board
            .iter()
            .enumerate()
            .filter(|(i, _)| !relevant_index.contains(i))
            .map(|(_, v)| v.clone())
            .collect::<Vec<Vec<Tile>>>();

        (tiles, completed_tiles)
    }

    fn pick_available(&self, user_tiles: &Vec<Tile>) -> (Vec<Vec<Tile>>, Vec<Tile>) {
        let mut tiles = user_tiles
            .clone()
            .into_iter()
            .filter(|t| !t.is_wildcard)
            .collect::<Vec<_>>();
        let wildcard_count = Game::wildcard_count(user_tiles);
        tiles.sort_unstable();

        let (available_pure_color_tiles_sets, leave_tiles) =
            self.get_available_pure_color_tiles_sets(&tiles);

        let (available_mixed_color_tiles_sets, leave_tiles) =
            self.get_available_mixed_colors_tiles_sets(&leave_tiles);

        let (available_tiles_with_wildcards, leave_tiles) =
            self.get_available_tiles_with_wildcards(&leave_tiles, wildcard_count);

        let available_tiles = vec![
            available_pure_color_tiles_sets,
            available_mixed_color_tiles_sets,
            available_tiles_with_wildcards,
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
        let wildcard_count = Game::wildcard_count(tiles);
        let mut tiles = tiles
            .iter()
            .filter(|t| !t.is_wildcard)
            .cloned()
            .collect::<Vec<_>>();
        tiles.sort_unstable();

        if wildcard_count == 1 {
            for wildcard in Tile::iter() {
                let mut tiles = tiles.clone();
                tiles.push(wildcard);
                tiles.sort_unstable();

                let (temp_tiles_set, temp_leave_tiles) =
                    self.generate_available_pure_color_tiles_sets(&tiles);

                if tiles_set.len() == 0 || temp_tiles_set.len() > tiles_set.len() {
                    tiles_set = temp_tiles_set;
                    leave_tiles = temp_leave_tiles;
                }
            }
        } else if wildcard_count == 2 {
            for w1 in Tile::iter() {
                for w2 in Tile::iter() {
                    let mut tiles = tiles.clone();
                    tiles.push(w1.clone());
                    tiles.push(w2);
                    tiles.sort_unstable();

                    let (temp_tiles_set, temp_leave_tiles) =
                        self.generate_available_pure_color_tiles_sets(&tiles);

                    if tiles_set.len() == 0 || temp_tiles_set.len() > tiles_set.len() {
                        tiles_set = temp_tiles_set;
                        leave_tiles = temp_leave_tiles;
                    }
                }
            }
        } else {
            (tiles_set, leave_tiles) = self.generate_available_pure_color_tiles_sets(&tiles);
        }

        (tiles_set, leave_tiles)
    }

    fn generate_available_pure_color_tiles_sets(
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

    fn get_available_mixed_colors_tiles_sets(
        &self,
        tiles: &Vec<Tile>,
    ) -> (Vec<Vec<Tile>>, Vec<Tile>) {
        let mut tiles_set = Vec::new();
        let mut leave_tiles = Vec::new();
        let wildcard_count = Game::wildcard_count(tiles);
        let tiles = tiles
            .iter()
            .filter(|t| !t.is_wildcard)
            .cloned()
            .collect::<Vec<_>>();

        if wildcard_count == 1 {
            for wildcard in Tile::iter() {
                let mut tiles = tiles.clone();
                tiles.push(wildcard.clone());

                let mut mixed_colors_tiles_set = vec![vec![]; 14];
                let mut skip = false;

                for tile in tiles {
                    let index = tile.number as usize;
                    mixed_colors_tiles_set[index].push(tile.clone());

                    if mixed_colors_tiles_set[index]
                        .iter()
                        .filter(|t| t.color == tile.color)
                        .collect::<Vec<_>>()
                        .len()
                        > 2
                    {
                        skip = true;
                        break;
                    }
                }

                if skip {
                    continue;
                }

                let (temp_tiles_set, temp_leave_tiles) =
                    self.generate_available_mixed_colors_tiles_sets(&mixed_colors_tiles_set);

                if tiles_set.len() == 0 || temp_tiles_set.len() > tiles_set.len() {
                    tiles_set = temp_tiles_set;
                    leave_tiles = temp_leave_tiles;
                }
            }
        } else if wildcard_count == 2 {
            for w1 in Tile::iter() {
                for w2 in Tile::iter() {
                    let mut tiles = tiles.clone();
                    tiles.push(w1.clone());
                    tiles.push(w2);

                    let mut mixed_colors_tiles_set = vec![vec![]; 14];
                    let mut skip = false;

                    for tile in tiles {
                        let index = tile.number as usize;
                        mixed_colors_tiles_set[index].push(tile.clone());

                        if mixed_colors_tiles_set[index]
                            .iter()
                            .filter(|t| t.color == tile.color)
                            .collect::<Vec<_>>()
                            .len()
                            > 2
                        {
                            skip = true;
                            break;
                        }
                    }

                    if skip {
                        continue;
                    }

                    let (temp_tiles_set, temp_leave_tiles) =
                        self.generate_available_mixed_colors_tiles_sets(&mixed_colors_tiles_set);

                    if tiles_set.len() == 0 || temp_tiles_set.len() > tiles_set.len() {
                        tiles_set = temp_tiles_set;
                        leave_tiles = temp_leave_tiles;
                    }
                }
            }
        } else {
            let mut mixed_colors_tiles_set = vec![vec![]; 14];

            for tile in tiles {
                let index = tile.number as usize;
                mixed_colors_tiles_set[index].push(tile.clone());
            }

            (tiles_set, leave_tiles) =
                self.generate_available_mixed_colors_tiles_sets(&mixed_colors_tiles_set);
        }

        (tiles_set, leave_tiles)
    }

    fn generate_available_mixed_colors_tiles_sets(
        &self,
        mixed_colors_tiles_set: &Vec<Vec<Tile>>,
    ) -> (Vec<Vec<Tile>>, Vec<Tile>) {
        let mixed_colors_tiles_set = mixed_colors_tiles_set
            .iter()
            .filter(|t| t.len() > 0)
            .cloned()
            .collect::<Vec<_>>();

        let is_valid_mixed_colors_tiles =
            |t: &Vec<Tile>| -> bool { t.len() >= 3 && Game::get_colors_count(t) >= 3 };

        let generate_mixed_colors_tiles_set = |tiles: &Vec<Tile>| -> Vec<Vec<Tile>> {
            if tiles.len() < 5 {
                vec![tiles.clone()]
            } else {
                let mut colors_map = HashMap::new();

                for tile in tiles {
                    let color = tile.color;

                    colors_map.entry(color).or_insert(vec![]).push(tile.clone());
                }

                let mut split_tiles1 = Vec::new();
                let mut split_tiles2 = Vec::new();

                for c in colors_map {
                    if c.1.len() > 1 {
                        split_tiles1.push(c.1[0].clone());
                        split_tiles2.push(c.1[1].clone());
                    } else {
                        if split_tiles1.len() < 3 || split_tiles1.len() < split_tiles2.len() {
                            split_tiles1.push(c.1[0].clone());
                        } else {
                            split_tiles2.push(c.1[0].clone());
                        }
                    }
                }

                split_tiles1.sort_unstable();
                split_tiles2.sort_unstable();

                vec![split_tiles1, split_tiles2]
            }
        };

        let tiles_set = mixed_colors_tiles_set
            .iter()
            .filter(|t| t.len() >= 3 && is_valid_mixed_colors_tiles(t))
            .flat_map(|t| generate_mixed_colors_tiles_set(t))
            .collect::<Vec<Vec<Tile>>>();

        let invalid_tiles = tiles_set
            .iter()
            .filter(|t| t.len() < 3)
            .cloned()
            .flatten()
            .collect::<Vec<_>>();

        let mut leave_tiles = mixed_colors_tiles_set
            .iter()
            .filter(|t| t.len() < 3 || !is_valid_mixed_colors_tiles(t))
            .flat_map(|t| t.clone())
            .collect::<Vec<Tile>>();

        leave_tiles.extend(invalid_tiles);

        let tiles_set = tiles_set
            .iter()
            .filter(|t| t.len() >= 3)
            .cloned()
            .collect::<Vec<_>>();

        (tiles_set, leave_tiles)
    }

    fn get_available_tiles_with_wildcards(
        &self,
        tiles: &Vec<Tile>,
        wildcard_count: usize,
    ) -> (Vec<Vec<Tile>>, Vec<Tile>) {
        let tiles = tiles
            .iter()
            .filter(|t| !t.is_wildcard)
            .cloned()
            .collect::<Vec<_>>();

        if wildcard_count == 1 {
            for tile in Tile::iter() {
                let mut tiles = tiles.clone();
                tiles.push(tile);

                let (available_tiles, leave_tiles) =
                    self.get_available_pure_color_tiles_sets(&tiles);

                if !available_tiles.is_empty() {
                    return (available_tiles, leave_tiles);
                }

                let (available_tiles, leave_tiles) =
                    self.get_available_mixed_colors_tiles_sets(&tiles);

                if !available_tiles.is_empty() {
                    return (available_tiles, leave_tiles);
                }
            }
        }

        if wildcard_count == 2 {
            for tile1 in Tile::iter() {
                for tile2 in Tile::iter() {
                    let mut tiles = tiles.clone();
                    tiles.push(tile1.clone());
                    tiles.push(tile2);

                    let (available_tiles, leave_tiles) =
                        self.get_available_pure_color_tiles_sets(&tiles);

                    if !available_tiles.is_empty() {
                        return (available_tiles, leave_tiles);
                    }

                    let (available_tiles, leave_tiles) =
                        self.get_available_mixed_colors_tiles_sets(&tiles);

                    if !available_tiles.is_empty() {
                        return (available_tiles, leave_tiles);
                    }
                }
            }
        }

        (vec![], tiles.clone())
    }
}

#[cfg(test)]
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
        let solution = solver.solve();

        assert!(solution.is_none());
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
        println!("{:?}", solver.get_available_mixed_colors_tiles_sets(&tiles));
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
        println!("{:?}", solver.get_available_mixed_colors_tiles_sets(&tiles));
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
            solver.get_available_mixed_colors_tiles_sets(&tiles)
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

        let solution = solver.search(tiles, Vec::new(), &mut HashSet::new());

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

        let solution = solver.search(tiles, Vec::new(), &mut HashSet::new());

        println!("{:?}", solution);
        assert!(solution.is_none());
    }

    #[test]
    fn test8() {
        let tiles = vec![
            vec![
                Tile::new(7, TileColor::Black, false),
                Tile::new(7, TileColor::Blue, false),
                Tile::new(7, TileColor::Orange, false),
                Tile::new(7, TileColor::Red, false),
            ],
            vec![
                Tile::new(6, TileColor::Orange, false),
                Tile::new(7, TileColor::Orange, false),
                Tile::new(8, TileColor::Orange, false),
                Tile::new(9, TileColor::Orange, false),
            ],
            vec![
                Tile::new(6, TileColor::Blue, false),
                Tile::new(6, TileColor::Orange, false),
                Tile::new(6, TileColor::Red, false),
            ],
            vec![
                Tile::new(1, TileColor::Blue, false),
                Tile::new(1, TileColor::Orange, false),
                Tile::new(1, TileColor::Red, false),
            ],
            vec![
                Tile::new(13, TileColor::Black, false),
                Tile::new(13, TileColor::Blue, false),
                Tile::new(13, TileColor::Red, false),
            ],
            vec![
                Tile::new(3, TileColor::Black, false),
                Tile::new(3, TileColor::Blue, false),
                Tile::new(3, TileColor::Orange, false),
                Tile::new(3, TileColor::Red, false),
            ],
            vec![
                Tile::new(5, TileColor::Black, false),
                Tile::new(6, TileColor::Black, false),
                Tile::new(7, TileColor::Black, false),
            ],
            vec![
                Tile::new(9, TileColor::Orange, false),
                Tile::new(10, TileColor::Orange, false),
                Tile::new(11, TileColor::Orange, false),
            ],
            vec![
                Tile::new(10, TileColor::Black, false),
                Tile::new(11, TileColor::Black, false),
                Tile::new(12, TileColor::Black, false),
            ],
            vec![
                Tile::new(1, TileColor::Orange, false),
                Tile::new(2, TileColor::Orange, false),
                Tile::new(3, TileColor::Orange, false),
                Tile::new(4, TileColor::Orange, false),
            ],
            vec![
                Tile::new(12, TileColor::Blue, false),
                Tile::new(12, TileColor::Orange, false),
                Tile::new(12, TileColor::Red, false),
            ],
            vec![
                Tile::new(10, TileColor::Red, false),
                Tile::new(11, TileColor::Red, false),
                Tile::new(12, TileColor::Red, false),
            ],
            vec![
                Tile::new(8, TileColor::Black, false),
                Tile::new(8, TileColor::Orange, false),
                Tile::new(8, TileColor::Red, false),
            ],
        ];

        let user_tiles = vec![
            Tile::new(2, TileColor::Black, false),
            Tile::new(4, TileColor::Black, false),
            Tile::new(2, TileColor::Blue, false),
            Tile::new(9, TileColor::Blue, false),
            Tile::new(2, TileColor::Orange, false),
            Tile::new(4, TileColor::Red, false),
            Tile::new(9, TileColor::Red, false),
        ];

        let mut board = vec![user_tiles];
        board.extend(tiles);

        let solver = Solver::new(Game::new_with_board(board.clone()));

        let solution = solver.solve();

        if let Some(solution) = solution.clone() {
            println!(
                "Solution Length: {}",
                solution.iter().flatten().collect::<Vec<_>>().len()
            );
            println!("Solution: {:?}", solution);
        }

        assert!(solution.is_some());
    }

    #[test]
    fn test9() {
        let board = vec![
            vec![
                Tile::new(11, TileColor::Orange, false),
                Tile::new(8, TileColor::Red, false),
                Tile::new(10, TileColor::Red, false),
            ],
            vec![
                Tile::new(10, TileColor::Blue, false),
                Tile::new(11, TileColor::Blue, false),
                Tile::new(12, TileColor::Blue, false),
            ],
            vec![
                Tile::new(10, TileColor::Orange, false),
                Tile::new(11, TileColor::Orange, true),
                Tile::new(12, TileColor::Orange, false),
                Tile::new(13, TileColor::Orange, false),
            ],
        ];

        println!(
            "Board Length: {}",
            board.iter().flatten().collect::<Vec<_>>().len()
        );

        let solver = Solver::new(Game::new_with_board(board.clone()));

        let solution = solver.solve();

        println!(
            "Solution Length: {}",
            solution
                .clone()
                .unwrap()
                .iter()
                .flatten()
                .collect::<Vec<_>>()
                .len()
        );

        println!("{:?}", solution);
        assert!(solution.is_some());
    }

    // load test
    #[test]
    fn test10() {
        let game_board = vec![
            vec![
                Tile::new(7, TileColor::Blue, false),
                Tile::new(8, TileColor::Blue, false),
                Tile::new(9, TileColor::Blue, false),
            ],
            vec![
                Tile::new(6, TileColor::Blue, false),
                Tile::new(6, TileColor::Orange, false),
                Tile::new(6, TileColor::Red, false),
            ],
            vec![
                Tile::new(7, TileColor::Red, false),
                Tile::new(8, TileColor::Red, false),
                Tile::new(9, TileColor::Red, false),
                Tile::new(10, TileColor::Red, false),
                Tile::new(11, TileColor::Red, false),
            ],
            vec![
                Tile::new(2, TileColor::Black, false),
                Tile::new(3, TileColor::Black, false),
                Tile::new(4, TileColor::Black, false),
                Tile::new(5, TileColor::Black, false),
                Tile::new(6, TileColor::Black, false),
            ],
            vec![
                Tile::new(1, TileColor::Black, false),
                Tile::new(1, TileColor::Blue, false),
                Tile::new(1, TileColor::Red, false),
            ],
            vec![
                Tile::new(10, TileColor::Black, false),
                Tile::new(10, TileColor::Red, false),
                Tile::new(10, TileColor::Blue, false),
                Tile::new(10, TileColor::Orange, false),
            ],
            vec![
                Tile::new(8, TileColor::Blue, false),
                Tile::new(9, TileColor::Blue, false),
                Tile::new(10, TileColor::Blue, false),
                Tile::new(11, TileColor::Blue, false),
                Tile::new(12, TileColor::Blue, false),
            ],
            vec![
                Tile::new(2, TileColor::Blue, false),
                Tile::new(3, TileColor::Blue, false),
                Tile::new(4, TileColor::Blue, false),
            ],
            vec![
                Tile::new(13, TileColor::Black, false),
                Tile::new(13, TileColor::Blue, false),
                Tile::new(13, TileColor::Orange, false),
            ],
            vec![
                Tile::new(12, TileColor::Black, false),
                Tile::new(12, TileColor::Red, false),
                Tile::new(12, TileColor::Blue, false),
                Tile::new(12, TileColor::Orange, false),
            ],
            vec![
                Tile::new(7, TileColor::Blue, false),
                Tile::new(7, TileColor::Orange, false),
                Tile::new(7, TileColor::Red, false),
            ],
            vec![
                Tile::new(13, TileColor::Black, false),
                Tile::new(13, TileColor::Orange, false),
                Tile::new(13, TileColor::Red, false),
            ],
            vec![
                Tile::new(1, TileColor::Orange, false),
                Tile::new(2, TileColor::Orange, false),
                Tile::new(3, TileColor::Orange, false),
                Tile::new(4, TileColor::Orange, false),
                Tile::new(5, TileColor::Orange, false),
            ],
            vec![
                Tile::new(5, TileColor::Orange, false),
                Tile::new(6, TileColor::Orange, false),
                Tile::new(7, TileColor::Orange, false),
            ],
            vec![
                Tile::new(6, TileColor::Blue, false),
                Tile::new(6, TileColor::Black, false),
                Tile::new(6, TileColor::Red, false),
            ],
            vec![
                Tile::new(1, TileColor::Black, false),
                Tile::new(2, TileColor::Black, false),
                Tile::new(3, TileColor::Black, false),
                Tile::new(4, TileColor::Black, false),
                Tile::new(5, TileColor::Black, false),
            ],
            vec![
                Tile::new(8, TileColor::Black, false),
                Tile::new(9, TileColor::Black, false),
                Tile::new(10, TileColor::Black, false),
                Tile::new(11, TileColor::Black, false),
                Tile::new(12, TileColor::Black, false),
            ],
            vec![
                Tile::new(8, TileColor::Black, false),
                Tile::new(8, TileColor::Orange, false),
                Tile::new(8, TileColor::Red, false),
            ],
            vec![
                Tile::new(9, TileColor::Black, false),
                Tile::new(9, TileColor::Orange, false),
                Tile::new(9, TileColor::Red, false),
            ],
            vec![
                Tile::new(11, TileColor::Black, false),
                Tile::new(11, TileColor::Orange, false),
                Tile::new(11, TileColor::Red, false),
            ],
            vec![
                Tile::new(4, TileColor::Blue, false),
                Tile::new(4, TileColor::Orange, false),
                Tile::new(4, TileColor::Red, false),
            ],
        ];

        let user_tiles = vec![
            Tile::new(2, TileColor::Red, false),
            Tile::new(3, TileColor::Red, true),
            Tile::new(4, TileColor::Red, false),
            Tile::new(13, TileColor::Red, false),
            Tile::new(10, TileColor::Orange, false),
            Tile::new(12, TileColor::Orange, false),
        ];

        let mut board = vec![user_tiles];
        board.extend(game_board.clone());

        let solver = Solver::new(Game::new_with_board(board));

        let solution = solver.solve();

        if let Some(solution) = solution.clone() {
            println!("Solution: {:?}", solution);
        }

        assert!(solution.is_some());
    }
}
