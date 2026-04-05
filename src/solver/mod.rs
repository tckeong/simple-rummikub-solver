use crate::game::{tile::Tile, Game};
use std::collections::{BTreeMap, HashSet};

type CacheKey = Vec<(u8, u8, bool)>;

pub struct Solver {
    game: Game,
}

impl Solver {
    pub fn new(game: Game) -> Self {
        Solver { game }
    }

    fn get_cache_key(tiles: &BTreeMap<Tile, u8>) -> CacheKey {
        let mut key = Vec::new();
        for (tile, &count) in tiles {
            for _ in 0..count {
                key.push((tile.number, tile.color.to_rank() as u8, tile.is_wildcard));
            }
        }
        // key is already somewhat sorted because BTreeMap is sorted by Tile
        key
    }

    fn search(
        &self,
        mut tiles: BTreeMap<Tile, u8>,
        solution_set: Vec<Vec<Tile>>,
        cache: &mut HashSet<CacheKey>,
    ) -> Option<Vec<Vec<Tile>>> {
        if tiles.values().all(|&c| c == 0) {
            return Some(solution_set);
        }

        let key = Self::get_cache_key(&tiles);
        if cache.contains(&key) {
            return None;
        }

        // Pick the first tile that has count > 0
        let first_tile = tiles
            .iter()
            .find(|(_, &count)| count > 0)
            .map(|(t, _)| t.clone())
            .unwrap();

        // Temporarily remove the first tile to generate candidates that MUST include it
        *tiles.get_mut(&first_tile).unwrap() -= 1;

        let wildcard_count = tiles
            .iter()
            .filter(|(t, _)| t.is_wildcard)
            .map(|(_, &c)| c)
            .sum::<u8>() as usize;

        let candidates = self.find_candidates(&first_tile, &tiles, wildcard_count);

        for candidate in candidates {
            let mut next_tiles = tiles.clone();
            let mut possible = true;

            // The first_tile is already removed from 'tiles' (the count was decremented)
            // But 'candidate' includes 'first_tile'.
            // So we only need to remove the OTHER tiles in the candidate.
            for (i, c_tile) in candidate.iter().enumerate() {
                if i == 0 && c_tile == &first_tile {
                    // This is our 'first_tile', already accounted for.
                    continue;
                }
                
                // If it's a wildcard we generated, we need to find A wildcard in next_tiles
                let target = if c_tile.is_wildcard {
                    next_tiles.keys().find(|t| t.is_wildcard && next_tiles[t] > 0).cloned()
                } else {
                    (next_tiles.get(c_tile).copied().unwrap_or(0) > 0).then(|| c_tile.clone())
                };

                if let Some(t) = target {
                    *next_tiles.get_mut(&t).unwrap() -= 1;
                } else {
                    possible = false;
                    break;
                }
            }

            if possible {
                let mut next_solution = solution_set.clone();
                next_solution.push(candidate);

                if let Some(solution) = self.search(next_tiles, next_solution, cache) {
                    return Some(solution);
                }
            }
        }

        // Backtrack: put the first tile back before caching the failure for THIS state
        *tiles.get_mut(&first_tile).unwrap() += 1;
        cache.insert(key);
        None
    }

    fn find_candidates(
        &self,
        tile: &Tile,
        others: &BTreeMap<Tile, u8>,
        wildcard_count: usize,
    ) -> Vec<Vec<Tile>> {
        let mut candidates = Vec::new();

        if tile.is_wildcard {
            return Vec::new();
        }

        candidates.extend(self.find_runs_for_tile(tile, others, wildcard_count));
        candidates.extend(self.find_groups_for_tile(tile, others, wildcard_count));

        candidates
    }

    fn find_runs_for_tile(
        &self,
        tile: &Tile,
        others: &BTreeMap<Tile, u8>,
        total_wildcards: usize,
    ) -> Vec<Vec<Tile>> {
        let mut runs = Vec::new();
        
        let same_color_tiles: Vec<Tile> = others
            .iter()
            .filter(|(t, &c)| t.color == tile.color && !t.is_wildcard && c > 0)
            .map(|(t, _)| t.clone())
            .collect();

        for len in 3..=13 {
            for p in 0..len {
                let start_num = tile.number as i16 - p as i16;
                let end_num = start_num + len as i16 - 1;

                if start_num < 1 || end_num > 13 {
                    continue;
                }

                let mut current_run = Vec::new();
                let mut wildcards_used = 0;
                let mut possible = true;

                for n in start_num..=end_num {
                    let num = n as u8;
                    if num == tile.number {
                        current_run.push(tile.clone());
                    } else if let Some(t) = same_color_tiles.iter().find(|t| t.number == num) {
                        current_run.push(t.clone());
                    } else if wildcards_used < total_wildcards {
                        wildcards_used += 1;
                        current_run.push(Tile::new(num, tile.color, true));
                    } else {
                        possible = false;
                        break;
                    }
                }

                if possible {
                    current_run.sort_unstable();
                    runs.push(current_run);
                }
            }
        }
        runs
    }

    fn find_groups_for_tile(
        &self,
        tile: &Tile,
        others: &BTreeMap<Tile, u8>,
        total_wildcards: usize,
    ) -> Vec<Vec<Tile>> {
        let mut groups = Vec::new();
        let same_number_tiles: Vec<Tile> = others
            .iter()
            .filter(|(t, &c)| t.number == tile.number && !t.is_wildcard && c > 0)
            .map(|(t, _)| t.clone())
            .collect();

        for len in 3..=4 {
            let other_colors: Vec<_> = crate::game::tile_color::TileColor::iter()
                .filter(|&c| c != tile.color)
                .collect();

            use itertools::Itertools;
            for colors in other_colors.into_iter().combinations(len - 1) {
                let mut current_group = vec![tile.clone()];
                let mut wildcards_used = 0;
                let mut possible = true;

                for &color in &colors {
                    if let Some(t) = same_number_tiles.iter().find(|t| t.color == color) {
                        current_group.push(t.clone());
                    } else if wildcards_used < total_wildcards {
                        wildcards_used += 1;
                        current_group.push(Tile::new(tile.number, color, true));
                    } else {
                        possible = false;
                        break;
                    }
                }

                if possible {
                    current_group.sort_unstable();
                    groups.push(current_group);
                }
            }
        }

        groups
    }

    pub fn solve(&self) -> Option<Vec<Vec<Tile>>> {
        let board = self.game.get_board();
        let mut tiles_map = BTreeMap::new();
        for tile in board.into_iter().flatten() {
            *tiles_map.entry(tile).or_insert(0) += 1;
        }

        if tiles_map.is_empty() {
            return Some(Vec::new());
        }

        self.search(tiles_map, Vec::new(), &mut HashSet::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{tile_color::TileColor, Command, GameOperation};

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
        assert!(solver.solve().is_some());
    }
}
