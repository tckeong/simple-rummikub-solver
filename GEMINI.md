# GEMINI.md - Project Context & Instructions

This file provides a high-level overview and development guidelines for the `rummy-app` project.

## Project Overview
`rummy-app` is a Rust-based Rummikub solver and game manager featuring an interactive Terminal User Interface (TUI). It allows users to track a Rummikub board, manage their hand, and find optimal tile combinations using a highly optimized search engine.

### Core Architecture
- **`game`**: Domain entities (`Tile`, `TileColor`), game state management (`Game`), and a regex-based command `Parser`.
- **`solver`**: A high-performance Depth-First Search (DFS) engine. It uses targeted candidate generation, lazy wildcard assignment, and `BTreeMap`-based state tracking to solve complex boards in sub-second time.
- **`views`**: An interactive TUI built with `crossterm`, implementing a page-based state machine (Main Menu, Rules, Game Board, Solver).

### Key Technologies
- **Rust (Edition 2021)**
- **crossterm**: For TUI rendering and raw mode terminal handling.
- **regex**: For parsing shorthand game commands.
- **itertools**: For efficient color combinations in group finding.

## Building and Running

### Development Commands
- **Run the App**: `cargo run`
- **Build**: `cargo build`
- **Run Unit Tests**: `cargo test`
- **Performance Benchmarking**: `cargo test --release solver::tests::test10` (Always use `--release` for solver performance testing).

## Development Conventions

### Coding Style
- **Efficiency**: Favor references (`&[Tile]`) or frequency maps (`BTreeMap<Tile, u8>`) over cloning large tile vectors.
- **Idiomatic Iterators**: Use `.count()`, `.is_empty()`, and other iterator adapters instead of manual loops or intermediate collections where possible.
- **Error Handling**: Commands and parsers should return `Result` types. Avoid `panic!` or `unwrap()` in the TUI loop to prevent terminal crashes.

### Solver Logic
- The solver is exhaustive. It picks the "first available" tile and attempts to form a valid set (Run or Group) around it.
- **Wildcards**: Wildcards should be treated as flexible placeholders. Do not permute wildcards through all 52 tile types upfront; instead, let the set-validation logic "consume" them as needed.
- **Memoization**: Always use `CacheKey` (a sorted representation of the tile pool) to avoid re-searching identical board states.

### TUI Management
- The TUI uses an alternate screen and raw mode. Ensure `terminal::disable_raw_mode()` and `terminal::LeaveAlternateScreen` are called on exit (handled in `TUI::run`).
- Page transitions are managed via the `Page` enum in `src/views/mod.rs`.
