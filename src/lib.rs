use std::cmp::{max, min};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use js_sys;

pub const BOARD_SIZE: usize = 15;
pub const WIN_LENGTH: usize = 5;
const MAX_DEPTH: i32 = 3; // Limit depth for performance

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn board_size() -> usize {
    BOARD_SIZE
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Cell {
    Empty,
    Black,
    White,
}

#[derive(Clone)]
pub struct Gomoku {
    board: [[Cell; BOARD_SIZE]; BOARD_SIZE],
    current_player: Cell,
}

impl Gomoku {
    pub fn new() -> Self {
        Gomoku {
            board: [[Cell::Empty; BOARD_SIZE]; BOARD_SIZE],
            current_player: Cell::Black,
        }
    }

    pub fn print_board(&self) {
        print!("  ");
        for i in 0..BOARD_SIZE {
            print!("{:2} ", i);
        }
        println!();

        for (i, row) in self.board.iter().enumerate() {
            print!("{:2} ", i);
            for &cell in row.iter() {
                match cell {
                    Cell::Empty => print!(".  "),
                    Cell::Black => print!("X  "),
                    Cell::White => print!("O  "),
                }
            }
            println!();
        }
        println!();
    }

    pub fn make_move(&mut self, row: usize, col: usize) -> Result<(), &'static str> {
        if row >= BOARD_SIZE || col >= BOARD_SIZE {
            return Err("Move out of bounds");
        }
        if self.board[row][col] != Cell::Empty {
            return Err("Cell already occupied");
        }
        self.board[row][col] = self.current_player;
        Ok(())
    }

    pub fn switch_player(&mut self) {
        self.current_player = match self.current_player {
            Cell::Black => Cell::White,
            Cell::White => Cell::Black,
            _ => Cell::Black,
        };
    }

    pub fn current_player(&self) -> Cell {
        self.current_player
    }

    pub fn check_winner(&self) -> Option<Cell> {
        let directions = [
            (0, 1),  // horizontal
            (1, 0),  // vertical
            (1, 1),  // diagonal down-right
            (1, -1), // diagonal down-left
        ];

        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                if self.board[row][col] == Cell::Empty {
                    continue;
                }
                let player = self.board[row][col];

                for &(dr, dc) in directions.iter() {
                    let mut count = 1;
                    for step in 1..WIN_LENGTH {
                        let r = row as i32 + dr * step as i32;
                        let c = col as i32 + dc * step as i32;

                        if r < 0 || r >= BOARD_SIZE as i32 || c < 0 || c >= BOARD_SIZE as i32 {
                            break;
                        }

                        if self.board[r as usize][c as usize] == player {
                            count += 1;
                        } else {
                            break;
                        }
                    }

                    if count >= WIN_LENGTH {
                        return Some(player);
                    }
                }
            }
        }
        None
    }

    pub fn is_board_full(&self) -> bool {
        self.board
            .iter()
            .all(|row| row.iter().all(|&cell| cell != Cell::Empty))
    }

    fn get_valid_moves(&self) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();
        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                if self.board[row][col] == Cell::Empty {
                    moves.push((row, col));
                }
            }
        }
        moves
    }

    fn evaluate(&self) -> i32 {
        let mut score = 0;
        let directions = [(0, 1), (1, 0), (1, 1), (1, -1)];

        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                if self.board[row][col] == Cell::Empty {
                    continue;
                }
                let player = self.board[row][col];
                let player_score = if player == Cell::White { 1 } else { -1 };

                for &(dr, dc) in directions.iter() {
                    let mut count = 1;
                    let mut open_ends = 0;

                    // Check forward
                    for step in 1..WIN_LENGTH {
                        let r = row as i32 + dr * step as i32;
                        let c = col as i32 + dc * step as i32;
                        if r < 0 || r >= BOARD_SIZE as i32 || c < 0 || c >= BOARD_SIZE as i32 {
                            break;
                        }
                        if self.board[r as usize][c as usize] == player {
                            count += 1;
                        } else if self.board[r as usize][c as usize] == Cell::Empty {
                            open_ends += 1;
                            break;
                        } else {
                            break;
                        }
                    }

                    // Check backward
                    for step in 1..WIN_LENGTH {
                        let r = row as i32 - dr * step as i32;
                        let c = col as i32 - dc * step as i32;
                        if r < 0 || r >= BOARD_SIZE as i32 || c < 0 || c >= BOARD_SIZE as i32 {
                            break;
                        }
                        if self.board[r as usize][c as usize] == player {
                            count += 1;
                        } else if self.board[r as usize][c as usize] == Cell::Empty {
                            open_ends += 1;
                            break;
                        } else {
                            break;
                        }
                    }

                    if count >= WIN_LENGTH {
                        score += player_score * 100000; // Winning position
                    } else if count == 4 && open_ends >= 1 {
                        score += player_score * 1000; // Four in a row, one open end
                    } else if count == 3 && open_ends == 2 {
                        score += player_score * 100; // Three in a row, two open ends
                    } else if count == 2 && open_ends == 2 {
                        score += player_score * 10; // Two in a row, two open ends
                    }
                }
            }
        }
        score
    }

    fn minimax(
        &self,
        depth: i32,
        alpha: i32,
        beta: i32,
        maximizing: bool,
    ) -> (i32, Option<(usize, usize)>) {
        if depth == 0 || self.check_winner().is_some() || self.is_board_full() {
            return (self.evaluate(), None);
        }

        let valid_moves = self.get_valid_moves();
        if valid_moves.is_empty() {
            return (self.evaluate(), None);
        }

        let mut best_move = None;
        let mut alpha = alpha;
        let mut beta = beta;

        if maximizing {
            let mut max_eval = i32::MIN;
            for &(row, col) in valid_moves.iter() {
                let mut new_game = self.clone();
                new_game.board[row][col] = Cell::White;
                let (eval, _) = new_game.minimax(depth - 1, alpha, beta, false);
                if eval > max_eval {
                    max_eval = eval;
                    best_move = Some((row, col));
                }
                alpha = max(alpha, eval);
                if beta <= alpha {
                    break; // Alpha-beta pruning
                }
            }
            (max_eval, best_move)
        } else {
            let mut min_eval = i32::MAX;
            for &(row, col) in valid_moves.iter() {
                let mut new_game = self.clone();
                new_game.board[row][col] = Cell::Black;
                let (eval, _) = new_game.minimax(depth - 1, alpha, beta, true);
                if eval < min_eval {
                    min_eval = eval;
                    best_move = Some((row, col));
                }
                beta = min(beta, eval);
                if beta <= alpha {
                    break; // Alpha-beta pruning
                }
            }
            (min_eval, best_move)
        }
    }

    pub fn ai_move(&mut self) -> (usize, usize) {
        let (_, best_move) = self.minimax(MAX_DEPTH, i32::MIN, i32::MAX, true);
        best_move.unwrap_or((7, 7)) // Default to center if no move found
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmGomoku {
    inner: Gomoku,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmGomoku {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmGomoku {
        WasmGomoku { inner: Gomoku::new() }
    }

    pub fn board(&self) -> Vec<u8> {
        self.inner
            .board
            .iter()
            .flat_map(|row| row.iter())
            .map(|cell| match cell {
                Cell::Empty => 0,
                Cell::Black => 1,
                Cell::White => 2,
            })
            .collect()
    }

    pub fn current_player(&self) -> u8 {
        match self.inner.current_player {
            Cell::Black => 1,
            Cell::White => 2,
            _ => 0,
        }
    }

    pub fn make_move(&mut self, row: usize, col: usize) -> bool {
        self.inner.make_move(row, col).is_ok()
    }

    pub fn ai_move(&mut self) -> js_sys::Array {
        let (r, c) = self.inner.ai_move();
        let arr = js_sys::Array::new();
        arr.push(&JsValue::from_f64(r as f64));
        arr.push(&JsValue::from_f64(c as f64));
        arr
    }

    pub fn check_winner(&self) -> u8 {
        match self.inner.check_winner() {
            Some(Cell::Black) => 1,
            Some(Cell::White) => 2,
            _ => 0,
        }
    }

    pub fn is_board_full(&self) -> bool {
        self.inner.is_board_full()
    }

    pub fn switch_player(&mut self) {
        self.inner.switch_player();
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_board_is_empty() {
        let game = Gomoku::new();
        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                assert_eq!(game.board[row][col], Cell::Empty);
            }
        }
        assert_eq!(game.current_player, Cell::Black);
    }

    #[test]
    fn make_move_and_switch_player() {
        let mut game = Gomoku::new();
        game.make_move(0, 0).unwrap();
        assert_eq!(game.board[0][0], Cell::Black);
        game.switch_player();
        assert_eq!(game.current_player, Cell::White);
    }

    #[test]
    fn invalid_move_out_of_bounds() {
        let mut game = Gomoku::new();
        assert!(game.make_move(BOARD_SIZE, BOARD_SIZE).is_err());
    }

    #[test]
    fn detect_horizontal_win() {
        let mut game = Gomoku::new();
        for col in 0..WIN_LENGTH {
            game.make_move(0, col).unwrap();
        }
        assert_eq!(game.check_winner(), Some(Cell::Black));
    }

    #[test]
    fn detect_diagonal_win() {
        let mut game = Gomoku::new();
        for i in 0..WIN_LENGTH {
            game.make_move(i, i).unwrap();
        }
        assert_eq!(game.check_winner(), Some(Cell::Black));
    }

    #[test]
    fn board_full_detection() {
        let mut game = Gomoku::new();
        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                game.make_move(row, col).unwrap();
                if row != BOARD_SIZE - 1 || col != BOARD_SIZE - 1 {
                    game.switch_player();
                }
            }
        }
        assert!(game.is_board_full());
    }
}
