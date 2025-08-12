use crate::game::{
    tile::{self, Tile},
    tile_color::TileColor,
    Game, TilesType,
};
use std::collections::{HashMap, HashSet};

pub struct Solver {
    game: Game,
}

impl Solver {
    pub fn new(game: Game) -> Self {
        Solver { game }
    }

    pub fn new_with_board(board: Vec<Vec<Tile>>) -> Self {
        Solver::new(Game::new_with_board(board))
    }

    fn tiles_to_string(tiles: Vec<Tile>) -> String {
        tiles
            .iter()
            .filter(|t| !t.is_wildcard)
            .fold(String::new(), |s, t| s + &t.to_string())
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

        if cache.contains(&Self::tiles_to_string(tiles.clone())) || tiles.len() <= 2 {
            return None;
        }

        let (available_pure_color_tiles, leave_tiles) =
            self.get_available_pure_color_tiles_sets(&tiles);

        if leave_tiles.is_empty() {
            let mut solution_set = solution_set.clone();
            solution_set.extend(available_pure_color_tiles);
            return Some(solution_set);
        }

        let (available_mixed_colors_tiles, leave_tiles) =
            self.get_available_mixed_colors_tiles_sets(&tiles);

        if leave_tiles.is_empty() {
            let mut solution_set = solution_set.clone();
            solution_set.extend(available_mixed_colors_tiles);
            return Some(solution_set);
        }

        let candidates = vec![available_pure_color_tiles, available_mixed_colors_tiles];
        let candidates = candidates.into_iter().flatten().collect::<Vec<_>>();

        let n = tiles.len();

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

            let remaining: Vec<_> = tiles
                .iter()
                .enumerate()
                .filter(|(i, _)| !removed.contains(i))
                .map(|(_, v)| v.clone())
                .collect();

            let mut next_solution = solution_set.clone();
            next_solution.push(candidate.clone());

            if let Some(solution) = self.search(remaining, next_solution.clone(), cache) {
                return Some(solution);
            }
        }

        cache.insert(Self::tiles_to_string(tiles));

        None
    }

    pub fn solve(&self) -> Option<Vec<Vec<Tile>>> {
        let board = self.game.get_board();
        let test_board = board.clone();
        let user_tiles = board[0].clone();
        let mut board = board[1..].to_vec();

        let (available_tiles, leave_tiles) = self.pick_available(&user_tiles);

        board.extend(available_tiles);

        println!(
            "Original Size: {}",
            test_board.iter().flatten().collect::<Vec<_>>().len()
        );

        println!(
            "Original wildcard count: {}",
            Game::wildcard_count(&test_board.iter().flatten().cloned().collect::<Vec<_>>())
        );

        let (mut tiles, completed_tiles) = self.pick_relevant_tiles(&board, &leave_tiles);

        tiles.push(leave_tiles);
        let tiles = tiles.iter().flatten().cloned().collect::<Vec<Tile>>();

        // println!("\n\nPut in tiles: {:?}\n\n", temp_tiles);

        // let solution = self.search(tiles, vec![], &mut HashSet::new());
        let test_tiles = test_board.into_iter().flatten().collect::<Vec<_>>();
        println!("Test tiles: {:?}", test_tiles);
        let solution = self.search(test_tiles, vec![], &mut HashSet::new());

        if let Some(solution) = solution {
            println!(
                "Solution: {}",
                solution.iter().flatten().collect::<Vec<_>>().len()
            );
            let mut temp_solution = solution.iter().flatten().cloned().collect::<Vec<_>>();
            temp_solution.sort_unstable();
            println!(
                "Solution tiles wildcard count: {}\n",
                Game::wildcard_count(&temp_solution)
            );
            println!("\n\nSolution tiles: {:?}\n\n", temp_solution);

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
                if (Game::get_tiles_type(&board[i]) == TilesType::MixedColor
                    && (board[i][0].number == tile.number || board[i].len() == 4))
                    || (Game::get_tiles_type(&board[i]) == TilesType::PureColor
                        && tile.number >= board[i][0].number
                        && tile.number <= board[i].last().unwrap().number)
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
                let mut temp_tiles_set = Vec::new();
                let mut temp_leave_tiles = Vec::new();
                let mut cur_tiles = vec![tiles[0].clone()];

                tiles.push(wildcard);
                let n = tiles.len();
                tiles.sort_unstable();

                for i in 1..n {
                    if tiles[i].color == tiles[i - 1].color
                        && tiles[i].number == tiles[i - 1].number + 1
                    {
                        cur_tiles.push(tiles[i].clone());
                    } else {
                        if cur_tiles.len() >= 3 {
                            temp_tiles_set.push(cur_tiles.clone());
                        } else {
                            temp_leave_tiles.extend(cur_tiles.clone());
                        }

                        cur_tiles = vec![tiles[i].clone()];
                    }
                }

                if cur_tiles.len() >= 3 {
                    temp_tiles_set.push(cur_tiles.clone());
                } else {
                    temp_leave_tiles.extend(cur_tiles.clone());
                }

                if temp_tiles_set.len() > tiles_set.len() {
                    tiles_set = temp_tiles_set;
                    leave_tiles = temp_leave_tiles;
                }
            }
        } else if wildcard_count == 2 {
            for w1 in Tile::iter() {
                for w2 in Tile::iter() {
                    let mut tiles = tiles.clone();
                    let mut temp_tiles_set = Vec::new();
                    let mut temp_leave_tiles = Vec::new();
                    let mut cur_tiles = vec![tiles[0].clone()];

                    tiles.push(w1.clone());
                    tiles.push(w2);
                    let n = tiles.len();
                    tiles.sort_unstable();

                    for i in 1..n {
                        if tiles[i].color == tiles[i - 1].color
                            && tiles[i].number == tiles[i - 1].number + 1
                        {
                            cur_tiles.push(tiles[i].clone());
                        } else {
                            if cur_tiles.len() >= 3 {
                                temp_tiles_set.push(cur_tiles.clone());
                            } else {
                                temp_leave_tiles.extend(cur_tiles.clone());
                            }

                            cur_tiles = vec![tiles[i].clone()];
                        }
                    }

                    if cur_tiles.len() >= 3 {
                        temp_tiles_set.push(cur_tiles.clone());
                    } else {
                        temp_leave_tiles.extend(cur_tiles.clone());
                    }

                    if temp_tiles_set.len() > tiles_set.len() {
                        tiles_set = temp_tiles_set;
                        leave_tiles = temp_leave_tiles;
                    }
                }
            }
        } else {
            let n = tiles.len();
            let mut cur_tiles = vec![tiles[0].clone()];

            for i in 1..n {
                if tiles[i].color == tiles[i - 1].color
                    && tiles[i].number == tiles[i - 1].number + 1
                {
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

                let mixed_colors_tiles_set = mixed_colors_tiles_set
                    .iter()
                    .filter(|t| t.len() > 0)
                    .cloned()
                    .collect::<Vec<_>>();

                let is_valid_mixed_colors_tiles = |t: &Vec<Tile>| -> bool {
                    (t.len() == 3 || t.len() == 4 || t.len() >= 6)
                        && ((Game::get_colors_count(t) == 3 && (t.len() == 3 || t.len() == 6))
                            || (Game::get_colors_count(t) == 4 && (t.len() == 4 || t.len() >= 6)))
                };

                let generate_mixed_colors_tiles_set = |tiles: &Vec<Tile>| -> Vec<Vec<Tile>> {
                    if tiles.len() < 6 {
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
                                if split_tiles1.len() < split_tiles2.len() {
                                    split_tiles1.push(c.1[0].clone());
                                } else {
                                    split_tiles2.push(c.1[0].clone());
                                }
                            }
                        }

                        vec![split_tiles1, split_tiles2]
                    }
                };

                let temp_tiles_set = mixed_colors_tiles_set
                    .iter()
                    .filter(|t| t.len() >= 3 && is_valid_mixed_colors_tiles(t))
                    .flat_map(|t| generate_mixed_colors_tiles_set(t))
                    .collect::<Vec<Vec<Tile>>>();

                let temp_leave_tiles = mixed_colors_tiles_set
                    .iter()
                    .filter(|t| t.len() < 3 || !is_valid_mixed_colors_tiles(t))
                    .flat_map(|t| t.clone())
                    .collect::<Vec<Tile>>();

                if temp_tiles_set.len() == 0 || temp_tiles_set.len() > tiles_set.len() {
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

                    let mixed_colors_tiles_set = mixed_colors_tiles_set
                        .iter()
                        .filter(|t| t.len() > 0)
                        .cloned()
                        .collect::<Vec<_>>();

                    let is_valid_mixed_colors_tiles = |t: &Vec<Tile>| -> bool {
                        (t.len() == 3 || t.len() == 4 || t.len() >= 6)
                            && ((Game::get_colors_count(t) == 3 && (t.len() == 3 || t.len() == 6))
                                || (Game::get_colors_count(t) == 4
                                    && (t.len() == 4 || t.len() > 6)))
                    };

                    let generate_mixed_colors_tiles_set = |tiles: &Vec<Tile>| -> Vec<Vec<Tile>> {
                        if tiles.len() < 6 {
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
                                    if split_tiles1.len() < split_tiles2.len() {
                                        split_tiles1.push(c.1[0].clone());
                                    } else {
                                        split_tiles2.push(c.1[0].clone());
                                    }
                                }
                            }

                            vec![split_tiles1, split_tiles2]
                        }
                    };

                    let temp_tiles_set = mixed_colors_tiles_set
                        .iter()
                        .filter(|t| t.len() >= 3 && is_valid_mixed_colors_tiles(t))
                        .flat_map(|t| generate_mixed_colors_tiles_set(t))
                        .collect::<Vec<Vec<Tile>>>();

                    let temp_leave_tiles = mixed_colors_tiles_set
                        .iter()
                        .filter(|t| t.len() < 3 || !is_valid_mixed_colors_tiles(t))
                        .flat_map(|t| t.clone())
                        .collect::<Vec<Tile>>();

                    if temp_tiles_set.len() == 0 || temp_tiles_set.len() > tiles_set.len() {
                        tiles_set = temp_tiles_set;
                        leave_tiles = temp_leave_tiles;
                    }
                }
            }
        } else {
            let mut mixed_colors_tiles_set = vec![vec![]; 14];

            for tile in tiles {
                if tile.is_wildcard {
                    continue;
                }

                let index = tile.number as usize;
                mixed_colors_tiles_set[index].push(tile.clone());
            }

            let is_valid_mixed_colors_tiles = |t: &Vec<Tile>| -> bool {
                (t.len() == 3 || t.len() == 4 || t.len() >= 6)
                    && ((Game::get_colors_count(t) == 3 && (t.len() == 3 || t.len() == 6))
                        || (Game::get_colors_count(t) == 4 && (t.len() == 4 || t.len() > 6)))
            };

            let generate_mixed_colors_tiles_set = |tiles: &Vec<Tile>| -> Vec<Vec<Tile>> {
                if tiles.len() < 6 {
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

            tiles_set = mixed_colors_tiles_set
                .iter()
                .filter(|t| t.len() >= 3 && is_valid_mixed_colors_tiles(t))
                .flat_map(|t| generate_mixed_colors_tiles_set(t))
                .collect::<Vec<Vec<Tile>>>();

            leave_tiles = mixed_colors_tiles_set
                .iter()
                .filter(|t| t.len() < 3 || !is_valid_mixed_colors_tiles(t))
                .flat_map(|t| t.clone())
                .collect::<Vec<Tile>>();
        }

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
            for color in TileColor::iter() {
                for number in 1..=13 {
                    let mut tiles = tiles.clone();
                    tiles.push(Tile::new(number, color, true));

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

        if wildcard_count == 2 {
            for c1 in TileColor::iter() {
                for c2 in TileColor::iter() {
                    for n1 in 1..=13 {
                        for n2 in 1..=13 {
                            let tile1 = Tile::new(n1, c1, true);
                            let tile2 = Tile::new(n2, c2, true);

                            let mut tiles = tiles.clone();
                            tiles.push(tile1);
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
            }
        }

        (vec![], tiles.clone())
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

    // load test
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
                Tile::new(3, TileColor::Orange, true),
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
            Tile::new(5, TileColor::Blue, false),
            Tile::new(9, TileColor::Blue, false),
            Tile::new(11, TileColor::Blue, false),
            Tile::new(2, TileColor::Orange, false),
            Tile::new(4, TileColor::Red, false),
            Tile::new(9, TileColor::Red, false),
            Tile::new(11, TileColor::Red, false),
        ];

        let mut board = vec![user_tiles];
        board.extend(tiles);

        let solver = Solver::new_with_board(board.clone());

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

        let solver = Solver::new_with_board(board.clone());

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
}
