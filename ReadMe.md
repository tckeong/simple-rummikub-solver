# Rummikub Solver App

A high-performance Terminal User Interface (TUI) application for tracking and solving Rummikub games. This tool features a highly optimized Depth-First Search (DFS) engine that can find valid board combinations in sub-second time, even with multiple wildcards.

## Key Features

- **Interactive TUI**: Seamlessly manage your hand and the board using an intuitive terminal interface.
- **Optimized DFS Solver**: Features targeted candidate generation and state-based memoization using a `BTreeMap` for maximum performance.
- **Lazy Wildcard Assignment**: Handles wildcards as flexible placeholders, avoiding combinatorial explosion.
- **Regex Command Parser**: Powerful shorthand commands for quick board manipulation (e.g., `a0(3)r`, `p(r,b,h)10`).
- **High Performance**: Solves complex boards with multiple wildcards in milliseconds (ensure release mode for best results).

## Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (2021 Edition)

### Running the App
```bash
cargo run
```

### Running Tests
For performance-critical tests like the solver:
```bash
cargo test --release
```

## Usage & Commands

The app follows a simple shorthand for tile manipulation:
- **`a` (Add)**: Add tiles to your hand. E.g., `a0(3)r` (Add Red 3 to hand at index 0).
- **`p` (Put)**: Put a set on the board. E.g., `p(9,10,11)h` (Put Black 9, 10, 11).
- **`r` (Replace)**: Replace a wildcard on the board.
- **`solve`**: Attempt to solve the current board and hand.

### Color Shorthand
- `r`: Red
- `b`: Blue
- `o`: Orange
- `h`: Black

## Architecture

- **`src/game`**: Core entities and command parsing.
- **`src/solver`**: Optimized search engine and heuristics.
- **`src/views`**: TUI implementation and state management.

For more detailed development context, see [GEMINI.md](./GEMINI.md).
