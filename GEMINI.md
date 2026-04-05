# GEMINI.md - Project Context & Instructions

This file provides the foundational context, architectural patterns, and development mandates for the `rummy-app` project.

## Project Essence
`rummy-app` is a high-performance Rummikub utility that bridges a custom Terminal User Interface (TUI) with a highly optimized Depth-First Search (DFS) engine. Its core objective is to reorganize a set of Rummikub tiles into valid runs and groups in sub-second time, even when considering multiple wildcards.

## Architectural Abstract

### 1. Data Transformation Pipeline
The system operates as a linear transformation pipeline:
- **Input Layer (`src/views`)**: Captures raw terminal keystrokes into a string buffer.
- **Parsing Layer (`src/game/parser.rs`)**: Uses Regex to decompose strings into `TileCommand` primitives.
- **Operation Layer (`src/game/mod.rs`)**: Translates commands into `GameOperation`s that mutate the `Game` state (the `board`).
- **Optimization Layer (`src/solver/mod.rs`)**: Flattens the board into a frequency map (`BTreeMap<Tile, u8>`) and executes an exhaustive DFS to find a valid `solution_set`.

### 2. Search & Solver Philosophy
The solver avoids the "combinatorial explosion" common in Rummikub solvers through several abstract strategies:
- **Targeted Candidate Generation**: Instead of finding all possible sets in the entire pool, the solver picks the "first available" tile and only generates sets that *must* include that tile.
- **Lazy Wildcard Assignment**: Wildcards are treated as abstract "jokers" during set generation. Their concrete values are only assigned when they are needed to fill a gap in a run or a group, rather than permuting them upfront.
- **State Memoization**: Every unique tile pool state is hashed/keyed. If the solver returns to a state it has already explored and failed, it backtracks immediately.

### 3. State & Memory Management
- **Frequency Mapping**: The board is represented as a `BTreeMap<Tile, u8>` during search. This provides $O(\log n)$ access and ensures that the "remaining tiles" state is compact and stable for caching.
- **Clone Minimization**: The search algorithm minimizes cloning by passing references where possible and using "decrement-then-recurse-then-increment" patterns for backtracking.

## Technical Mandates

### Building and Running
- **Primary Build**: `cargo build`
- **TUI Execution**: `cargo run`
- **Solver Performance**: Always test the solver with the `--release` flag (`cargo test --release`). Debug mode performance is significantly slower due to the deep recursion and heavy iterator usage.

### Development Conventions
- **Entity Equality**: The `Tile` struct's `PartialEq` implementation **must** include `is_wildcard`. This is critical for the solver to correctly distinguish between a joker and a regular tile with the same value.
- **Iterator Idioms**: Prefer `.count()` and `.is_empty()` over `.collect().len()`. Avoid unnecessary allocations in the hot loops of the solver.
- **TUI Safety**: Never use `panic!` or `unwrap()` in code paths triggered by the TUI. All errors must be propagated as `Result` and displayed on the `InvalidCommandPage`.

### Page State Machine
The TUI is a finite state machine driven by the `Page` enum. New screens or modal states must be added as variants to this enum to maintain clean navigation and "Previous Page" logic.
