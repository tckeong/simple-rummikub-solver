use std::io::Result as ioResult;
use views::TUI;

pub mod game;
pub mod solver;
pub mod views;

fn main() -> ioResult<()> {
    TUI::new().run()
}
