# Technical Overview: Structs and Attributes

This document provides a detailed breakdown of the key structs used in the `rummy-app` project and the purpose of their attributes.

## 1. Module: `game`

### `Tile` (in `src/game/tile.rs`)
The fundamental unit of the game.
- **`number: u8`**: The face value of the tile (1-13). For wildcards, this stores the "assigned" number.
- **`color: TileColor`**: The color of the tile (Black, Blue, Orange, Red).
- **`is_wildcard: bool`**: A flag indicating if this tile is a joker/wildcard. 
  *Note: `PartialEq` now includes this to distinguish a physical wildcard from a regular tile with the same value.*

### `Game` (in `src/game/mod.rs`)
The central state manager for the board and player hand.
- **`board: Vec<Vec<Tile>>`**: A collection of tile sets. 
  - `board[0]` is conventionally the **player's hand**.
  - `board[1..]` are the **sets currently on the table**.

### `GameOperation` (in `src/game/mod.rs`)
A data structure representing an intended change to the game state.
- **`command: Command`**: The action type (`Add`, `Put`, `Draw`, `Replace`).
- **`index: usize`**: The target row in the `board` for the operation.
- **`tiles: Vec<Tile>`**: The tiles being added or moved.
- **`replace_tiles: Option<Vec<Tile>>`**: Specifically for the `Replace` command, these are the tiles that will physically take the place of wildcards on the board.

### `TileCommand` (in `src/game/tile_command.rs`)
The intermediate result of parsing a user's string input.
- **`cmd`, `idx`, `args`, `tail`**: Raw parts of a command like `a0(3)r`.
- **`replace_args`, `replace_tail`**: Parts used specifically for the complex `Replace` syntax.

---

## 2. Module: `solver`

### `Solver` (in `src/solver/mod.rs`)
The engine responsible for finding valid board configurations.
- **`game: Game`**: A snapshot of the current game state to be solved.

### `CacheKey` (Type Alias: `Vec<(u8, u8, bool)>`)
Used for memoization in the DFS search.
- **Attributes**: Represents a sorted "snapshot" of all tiles currently being processed. If the solver encounters the same `CacheKey` twice, it knows it has already failed that branch and can backtrack immediately.

---

## 3. Module: `views`

### `TUI` (in `src/views/mod.rs`)
The Terminal User Interface controller.
- **`output: Stdout`**: The handle to the terminal screen.
- **`buffer: String`**: Stores the user's current keystrokes before they hit `Enter`.
- **`y_pos: u16`**: Tracks the vertical cursor position for dynamic rendering.
- **`page: Page`**: The current active screen (e.g., `MainPage`, `GamePage`, `SolverPage`).
- **`prev_page: Page`**: Used to return to the correct screen after an error message.
- **`game: Game`**: The live game instance being manipulated by the user.

## Data Flow Summary
1. **Input**: User types a string into the `TUI.buffer`.
2. **Parsing**: `Parser` converts the string into a `TileCommand`.
3. **Conversion**: `TileCommand` is validated and converted into a `GameOperation`.
4. **Execution**: `Game.operate(operation)` updates the `board`.
5. **Solving**: `Solver` reads the `board`, converts it into a frequency map (`BTreeMap<Tile, u8>`), and runs a DFS search to find a valid `solution_set`.
