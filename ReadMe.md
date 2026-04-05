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

The app follows a specific shorthand for manipulating tiles. Commands generally follow the format: `[cmd][idx]([args])[tail]`.

### Command Reference

| Command | Action | Example | Description |
| :--- | :--- | :--- | :--- |
| **`a`** | **Add** | `a0(3,4,5)r` | Adds Red 3, 4, and 5 to the tile set at index 0 (usually your hand). |
| **`p`** | **Put** | `p(9,10,11)h` | Puts a **Run** of Black 9, 10, 11 on the board. |
| | | `p(r,b,h)10` | Puts a **Group** of Red, Blue, and Black 10s on the board. |
| **`d`** | **Draw** | `d(10)r` | Draws a Red 10 from the deck and adds it to your hand. |
| **`r`** | **Replace** | `r1(11)h(10,12)b` | **Replace** a wildcard in set 1 with Black 11. Then, put that wildcard into a new set of Blue 10 and 12, and put it on the board. |
| **`solve`** | **Solve** | `solve` | Triggers the DFS solver to reorganize the board and hand into valid sets. |

### Argument Rules
- **Numbers as Args**: Use numbers inside `()` and a color code as the `tail` to create a **Run**.
- **Colors as Args**: Use color codes (`r,b,o,h`) inside `()` and a number as the `tail` to create a **Group**.
- **Wildcards**: Use `w` inside the `()` to represent a wildcard. E.g., `a0(11,w,13)b`.

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
