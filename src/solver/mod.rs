use crate::game::Game;

struct Solver {
    game: Game,
}

impl Solver {
    pub fn new(game: Game) -> Self {
        Solver { game }
    }

    pub fn solve(&mut self) -> Result<bool, String> {
        let board = self.game.get_board();

        let user_tiles = board[0].clone();
        let mut board = board[1..].iter().cloned().collect::<Vec<_>>();

        Ok(false)
    }
}
