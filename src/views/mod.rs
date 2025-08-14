use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    style::{ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
    Command, ExecutableCommand,
};

use crate::game::{
    parser::{command_capture_to_tile_command, commands_capture_to_tile_commands, Parser},
    tile::Tile,
    Game, ToTiles,
};
use crate::solver::Solver;
use std::io::{stdout, Result as ioResult, Stdout, Write};

pub struct TUI {
    output: Stdout,
    buffer: String,
    y_pos: u16,
    page: Page,
    prev_page: Page,
    game: Game,
}

#[derive(Clone, Debug)]
enum Page {
    MainPage,
    GameRulePage,
    GameInitPage,
    GamePage,
    SolverPage,
    InvalidCommandPage { error_message: String },
}

impl TUI {
    pub fn new() -> Self {
        TUI {
            output: stdout(),
            buffer: String::new(),
            y_pos: 0,
            page: Page::MainPage,
            prev_page: Page::MainPage,
            game: Game::new(),
        }
    }

    fn reset(&mut self) {
        self.buffer.clear();
        self.y_pos = 0;
        self.game.reset();
    }

    pub fn run(&mut self) -> ioResult<()> {
        terminal::enable_raw_mode()?;
        self.execute(terminal::EnterAlternateScreen)?;

        loop {
            if self.render()? {
                break;
            }
        }

        self.execute(terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;

        Ok(())
    }

    // Returns true if the application should exit
    fn render(&mut self) -> ioResult<bool> {
        self.execute(terminal::Clear(ClearType::All))?;
        self.y_pos = 0;
        let page = self.page.clone();

        match page {
            Page::MainPage => {
                self.reset();
                self.render_main_page()?;

                self.flush()?;

                let mut should_exit = false;

                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => should_exit = true,
                        KeyCode::Char('s') => {
                            self.page = Page::GameRulePage;
                        }
                        _ => {}
                    }
                }

                Ok(should_exit)
            }

            Page::GameRulePage => {
                self.render_game_rule_page()?;

                self.flush()?;

                let mut should_exit = false;

                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => should_exit = true,
                        KeyCode::Char('m') => {
                            self.page = Page::MainPage;
                        }
                        KeyCode::Char('n') => {
                            self.page = Page::GameInitPage;
                        }
                        _ => {}
                    }
                };

                Ok(should_exit)
            }

            Page::GameInitPage => {
                self.render_game_init_page()?;

                self.flush()?;

                let mut should_exit = false;

                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => should_exit = true,
                        KeyCode::Char('m') => {
                            self.page = Page::MainPage;
                        }
                        KeyCode::Char(c) => {
                            self.buffer.push_str(&c.to_string());
                        }
                        KeyCode::Backspace => {
                            self.buffer.pop();
                        }
                        KeyCode::Enter => {
                            let command = self.buffer.trim();
                            let command = Parser::new().parse_init(command);

                            if let Some(commands) = command {
                                let tile_commands =
                                    commands_capture_to_tile_commands(commands, &self.game);

                                match tile_commands {
                                    Ok(tile_commands) => {
                                        let game_operations = tile_commands
                                            .into_iter()
                                            .map(|cmd| cmd.to_tiles())
                                            .collect::<Vec<_>>();

                                        for operation in game_operations {
                                            self.game.operate(operation);
                                        }

                                        self.page = Page::GamePage;
                                    }
                                    Err(e) => {
                                        self.display_error(&e.to_string());
                                    }
                                }
                            } else {
                                self.display_error("Invalid command format!");
                            }

                            self.buffer.clear();
                        }
                        _ => {}
                    }
                };

                Ok(should_exit)
            }

            Page::GamePage => {
                self.render_game_page()?;

                self.flush()?;

                let mut should_exit = false;

                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => should_exit = true,

                        KeyCode::Char('m') => {
                            self.page = Page::MainPage;
                        }

                        KeyCode::Char(c) => {
                            self.buffer.push_str(&c.to_string());
                        }

                        KeyCode::Backspace => {
                            self.buffer.pop();
                        }

                        KeyCode::Enter => {
                            let command = self.buffer.trim();

                            if command == "solve" {
                                self.page = Page::SolverPage;
                            } else {
                                let command = Parser::new().parse(command);

                                if let Some(cmd) = command {
                                    let tile_command =
                                        command_capture_to_tile_command(cmd, &self.game);

                                    match tile_command {
                                        Ok(tile_command) => {
                                            let game_operation = tile_command.to_tiles();
                                            self.game.operate(game_operation);
                                        }
                                        Err(e) => {
                                            self.display_error(&e.to_string());
                                        }
                                    }
                                } else {
                                    self.display_error("Invalid command format!");
                                }
                            }

                            self.buffer.clear();
                        }
                        _ => {}
                    }
                };

                Ok(should_exit)
            }

            Page::SolverPage => {
                self.render_solver_page()?;

                self.flush()?;

                let mut should_exit = false;

                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => should_exit = true,
                        KeyCode::Char('m') => {
                            self.page = Page::MainPage;
                        }
                        KeyCode::Char('c') => {
                            self.page = Page::GamePage;
                        }
                        _ => {}
                    }
                };

                Ok(should_exit)
            }

            Page::InvalidCommandPage { error_message } => {
                self.execute_move(0, 0)?;
                self.print_and_move(format!("Invalid command: {}", error_message).as_str(), 1)?;
                self.print_and_move("Press 'c' to continue the game.", 1)?;
                self.print_and_move(
                    "Press 'm' to return to the main menu, and restart new game.",
                    1,
                )?;
                self.print_and_move("Press 'q' to quit the game.", 1)?;
                self.flush()?;

                let mut should_exit = false;

                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('c') => {
                            self.page = self.prev_page.clone();
                        }
                        KeyCode::Char('m') => {
                            self.page = Page::MainPage;
                        }
                        KeyCode::Char('q') => {
                            should_exit = true;
                        }
                        _ => {}
                    }
                }

                Ok(should_exit)
            }
        }
    }

    fn render_main_page(&mut self) -> ioResult<()> {
        self.draw_box(50, 2, 0, true, "Rummy Solver App")?;
        self.execute_move(0, 1)?;
        print!("Press 'q' to quit or press 's' to start a game.");
        self.y_pos += 1;

        Ok(())
    }

    fn render_game_rule_page(&mut self) -> ioResult<()> {
        self.execute_move(0, 0)?;
        self.print_and_move("Game Page - Rules", 2)?;
        self.print_and_move("Available Commands: ", 1)?;
        self.print_and_move("    a - Add a tile to current tiles set.", 1)?;
        self.print_and_move("    d - Draw a tile from the deck.", 1)?;
        self.print_and_move("    p - put a set of tiles on the table.", 1)?;
        self.print_and_move("    r - Replace the wildcards.", 1)?;
        self.print_and_move("    solve - Solve the game.", 2)?;
        self.print_and_move(
            "[color] - The color of the tile can be 'b' - blue, 'r' - red, 'o' - orange, 'h' - black.",
            2,
        )?;
        self.print_and_move(
            "d([number])[color] - Draw a tile with a specific number and color.",
            1,
        )?;
        self.print_and_move("p[index]([number], *)[color] - Put a set of tiles with different numbers and same color.", 1)?;
        self.print_and_move("p[index]([color], *)[number] - Put a set of tiles with different colors and same number.", 1)?;
        self.print_and_move("r[index]([number], *)[color]([number^], *)[color^] - Replace the wildcards with the first set of tiles, 
        \r\tand then put the wildcards into the second sets (which has the ^ symbol).", 2)?;
        self.print_and_move("[command][index]([number], *)[color] - Execute a command with a set of tiles with different numbers and same color.", 1)?;
        self.print_and_move("[command][index]([color], *)[number] - Execute a command with a set of tiles with different colors and same number.", 2)?;
        self.print_and_move("Example: ", 1)?;
        self.print_and_move(
            "    a0(3)r - Add a red tile with number 3 to the tile set with index 0.",
            1,
        )?;
        self.print_and_move(
            "    a1(1,2)b - Add blue tiles with number 1, 2 to the tile set with index 1.",
            1,
        )?;
        self.print_and_move(
            "    a1(r,b)2 - Add red, blue tile with number 2 to the tile set with index 1.",
            1,
        )?;
        self.print_and_move(
            "    p(9,10,11)h - Put a set of tiles with number 9,10,11 with color Black on the table.",
            1,
        )?;
        self.print_and_move(
            "    p(r,b,h)10 - Put a set of tiles with color Red, Blue, Black with number 10 on the table.",
            1,
        )?;
        self.print_and_move(
            "    d(10)r - Draw a red tile with number 10 from the deck.",
            1,
        )?;
        self.print_and_move(
            "    r1(11,12)h(10,12)b - Replace the wildcards with black tiles with number 11 and 12 to the tile set with index 1.",   1)?;
        self.print_and_move("\tThen, put these wildcards to the blue tiles with number 10 and 12, and put the new tiles to the board.", 2)?;
        self.print_and_move("Rules End.", 2)?;

        self.print_and_move("Press 'n' to continue...", 1)?;

        Ok(())
    }

    fn render_game_init_page(&mut self) -> ioResult<()> {
        self.execute_move(0, 0)?;
        self.print_and_move("Please enter your initial tile set. ", 1)?;
        self.print_and_move("Format: [color]([number]) (e.g. r(3) for red 3)", 1)?;
        self.print_and_move("Press 'Enter' when done.", 2)?;

        self.print_and_move(
            format!("Your initial tile set: {}", self.buffer).as_str(),
            0,
        )?;
        self.execute_move(23 + self.buffer.len() as u16, 0)?;

        Ok(())
    }

    fn render_game_page(&mut self) -> ioResult<()> {
        self.execute_move(0, 0)?;
        let board = self.game.get_board();
        self.print_and_move("Initital Tile Set: ", 1)?;

        for (j, tile) in board[0].iter().enumerate() {
            self.draw_tile(tile.clone(), 2 + (j * 8) as u16)?;
        }

        self.y_pos += 4;

        self.print_and_move("Current Board: ", 2)?;
        let board = self.game.get_board();

        self.print_board(&board, true)?;
        self.y_pos += 4;

        self.execute_move(0, 0)?;
        self.print_and_move(format!("Your current command: {}", self.buffer).as_str(), 0)?;
        self.execute_move(22 + self.buffer.len() as u16, 0)?;

        Ok(())
    }

    fn render_solver_page(&mut self) -> ioResult<()> {
        let game = self.game.clone();
        let solver = Solver::new(game);

        self.print_and_move("Loading...", 0)?;

        let result = solver.solve();

        self.execute_move(0, 0)?;

        if let Some(board) = result {
            self.print_and_move("Game Solved!", 1)?;
            self.print_and_move("Press 'c' to continue...", 2)?;

            self.print_and_move("The solution board: ", 2)?;

            self.print_board(&board, false)?;
        } else {
            self.print_and_move("Game Not Solved!", 1)?;
            self.print_and_move("Press 'c' to continue...", 2)?;
        }

        self.execute_move(0, 0)?;

        Ok(())
    }

    fn print_board(&mut self, board: &Vec<Vec<Tile>>, skip: bool) -> ioResult<()> {
        let mut count = 0;

        for (i, row) in board.iter().enumerate() {
            if skip && i == 0 {
                continue;
            }

            let x_pos =
                (board[i - 1].len() * 1.min(count) * 8 + 11 * 1.min(count) + (12) * count) as u16;

            self.execute(cursor::MoveTo(x_pos, self.y_pos + 1))?;
            print!("Index {}: ", i);
            self.execute_move(0, 0)?;
            for (j, tile) in row.iter().enumerate() {
                self.draw_tile(tile.clone(), x_pos + 11 + (j * 8) as u16)?;
            }

            count += 1;

            if board[i - 1].len() > 5 || count > 1 {
                self.y_pos += 3;
                count = 0;
            }
        }

        self.y_pos += 4;

        Ok(())
    }

    fn print_and_move(&mut self, text: &str, offset: u16) -> ioResult<()> {
        self.execute_move(0, 0)?;
        print!("{}", text);
        self.execute_move(0, offset)?;
        Ok(())
    }

    fn execute_move(&mut self, x_pos: u16, y_offset: u16) -> ioResult<()> {
        self.y_pos += y_offset;
        self.execute(cursor::MoveTo(x_pos, self.y_pos))
    }

    fn execute(&mut self, command: impl Command) -> ioResult<()> {
        self.output.execute(command)?;

        Ok(())
    }

    fn flush(&mut self) -> ioResult<()> {
        self.output.flush()
    }

    fn display_error(self: &mut Self, error: &str) {
        self.prev_page = self.page.clone();
        self.page = Page::InvalidCommandPage {
            error_message: error.to_string(),
        };
        self.buffer.clear();
    }

    fn generate_text_middle(self: &Self, width: u16, text: &str) -> String {
        let padding = (width as usize - 2 - text.len()).max(0);
        let left_padding = padding / 2;
        let right_padding = padding - left_padding;
        format!(
            "│{}{}{}│",
            " ".repeat(left_padding),
            text,
            " ".repeat(right_padding)
        )
    }

    fn draw_box(
        self: &mut Self,
        width: u16,
        height: u16,
        x_pos: u16,
        modify_y: bool,
        text: &str,
    ) -> ioResult<()> {
        let top = format!("┌{}┐", "─".repeat((width - 2) as usize));
        let blank_middle = format!("│{}│", " ".repeat((width - 2) as usize));
        let text_middle = self.generate_text_middle(width, text);
        let bottom = format!("└{}┘", "─".repeat((width - 2) as usize));

        self.execute(cursor::MoveTo(x_pos, self.y_pos))?;

        let cursor_y = self.y_pos;

        print!("{}", top);
        for i in 1..=height {
            self.execute(cursor::MoveTo(x_pos, cursor_y + i))?;
            if i == (height >> 1) {
                print!("{}", text_middle);
            } else {
                print!("{}", blank_middle);
            }
        }
        self.execute(cursor::MoveTo(x_pos, cursor_y + height))?;
        print!("{}", bottom);

        self.flush()?;

        if modify_y {
            self.y_pos += height + 1;
        }

        Ok(())
    }

    fn draw_tile(self: &mut Self, tile: Tile, x_pos: u16) -> ioResult<()> {
        self.execute(SetForegroundColor(tile.color.as_color()))?;
        let text = if tile.is_wildcard {
            format!("w {}", tile.number)
        } else {
            format!("{}", tile.number)
        };
        self.draw_box(8, 2, x_pos, false, text.as_str())?;
        self.execute(ResetColor)?;

        Ok(())
    }
}
