# Gomoku Game in Rust

This is a console-based Gomoku game implemented in Rust. Gomoku is a two-player strategy game on a 15x15 grid where players alternate placing stones to form five in a row (horizontally, vertically, or diagonally). The game pits a human player (Black, X) against an AI opponent (White, O) powered by the Minimax algorithm with alpha-beta pruning.

## Features

- **Console Interface**: Text-based gameplay with a clear board display.
- **Human vs. AI**: Play as Black (X) against an AI as White (O).
- **Minimax AI**: AI uses Minimax with alpha-beta pruning (depth 3) for strategic moves.
- **Win/Draw Detection**: Detects wins (five in a row) or draws (full board).
- **Input Validation**: Ensures valid moves with error messages for invalid inputs.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable version recommended)
- Cargo (included with Rust)

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/<your-username>/gomoku.git
   cd gomoku
