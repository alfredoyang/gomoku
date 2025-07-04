use std::cmp::{max, min};

#[cfg(target_arch = "wasm32")]
use js_sys;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub const BOARD_SIZE: usize = 15;
pub const WIN_LENGTH: usize = 5;
const MAX_DEPTH: i32 = 3; // Limit depth for performance

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
/// Return the board size constant for the WebAssembly bindings.
///
/// This helper exposes the compile-time board dimension so the
/// JavaScript side can allocate buffers of the correct length.
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
    /// Create a new game with an empty board and the Black player to move.
    pub fn new() -> Self {
        Gomoku {
            board: [[Cell::Empty; BOARD_SIZE]; BOARD_SIZE],
            current_player: Cell::Black,
        }
    }

    /// Display the board state to the console using ASCII characters.
    ///
    /// Empty cells are shown with `.` while black and white stones are
    /// displayed as `X` and `O` respectively. The method also prints row
    /// and column indices for easier interaction in the console version.
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

    /// Place a stone for the current player.
    ///
    /// Returns an error if the coordinates are outside the board or the
    /// cell is already occupied. On success the stone is placed but the
    /// player is not automatically switched.
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

    /// Toggle the current player between Black and White.
    pub fn switch_player(&mut self) {
        self.current_player = match self.current_player {
            Cell::Black => Cell::White,
            Cell::White => Cell::Black,
            _ => Cell::Black,
        };
    }

    /// Get the player whose turn it is to move.
    pub fn current_player(&self) -> Cell {
        self.current_player
    }

    /// Determine if either player has achieved five in a row.
    ///
    /// The method scans the board in all four directions starting from each
    /// occupied cell. If a sequence of `WIN_LENGTH` stones belonging to the
    /// same player is found, that player is returned.
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

    /// Check if there are no empty cells remaining on the board.
    pub fn is_board_full(&self) -> bool {
        self.board
            .iter()
            .all(|row| row.iter().all(|&cell| cell != Cell::Empty))
    }

    /// Collect all empty board positions.
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

    /// Heuristic evaluation of the board from the given player's
    /// perspective.
    ///
    /// Positive scores favour the supplied player while negative scores
    /// favour the opponent. The function looks for runs of stones with
    /// open ends and assigns increasingly large scores as sequences grow
    /// longer.
    fn evaluate(&self, perspective: Cell) -> i32 {
        let mut score = 0;
        let directions = [(0, 1), (1, 0), (1, 1), (1, -1)];

        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                if self.board[row][col] == Cell::Empty {
                    continue;
                }
                let player = self.board[row][col];
                let player_score = if player == perspective { 1 } else { -1 };

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

    /// Minimax search with alpha-beta pruning.
    ///
    /// * `depth` limits the recursive search depth.
    /// * `alpha` and `beta` are the current bounds for pruning.
    /// * `player` indicates whose turn it is at this node.
    /// * `ai_player` is the color the AI is playing.
    fn minimax(
        &self,
        depth: i32,
        alpha: i32,
        beta: i32,
        player: Cell,
        ai_player: Cell,
    ) -> (i32, Option<(usize, usize)>) {
        if depth == 0 || self.check_winner().is_some() || self.is_board_full() {
            return (self.evaluate(ai_player), None);
        }

        let valid_moves = self.get_valid_moves();
        if valid_moves.is_empty() {
            return (self.evaluate(ai_player), None);
        }

        let mut best_move = None;
        let mut alpha = alpha;
        let mut beta = beta;

        let maximizing = player == ai_player;
        if maximizing {
            let mut max_eval = i32::MIN;
            for &(row, col) in valid_moves.iter() {
                let mut new_game = self.clone();
                new_game.board[row][col] = player;
                let (eval, _) = new_game.minimax(
                    depth - 1,
                    alpha,
                    beta,
                    match player {
                        Cell::White => Cell::Black,
                        Cell::Black => Cell::White,
                        Cell::Empty => Cell::Empty,
                    },
                    ai_player,
                );
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
                new_game.board[row][col] = player;
                let (eval, _) = new_game.minimax(
                    depth - 1,
                    alpha,
                    beta,
                    match player {
                        Cell::White => Cell::Black,
                        Cell::Black => Cell::White,
                        Cell::Empty => Cell::Empty,
                    },
                    ai_player,
                );
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

    /// Choose an optimal move for the AI using minimax.
    ///
    /// Returns the board coordinates of the best move. If no move is
    /// found (which should not happen in normal play) the center of the
    /// board is returned as a fallback.
    pub fn ai_move(&mut self) -> (usize, usize) {
        let player = self.current_player;
        let (_, best_move) = self.minimax(MAX_DEPTH, i32::MIN, i32::MAX, player, player);
        best_move.unwrap_or((BOARD_SIZE / 2, BOARD_SIZE / 2)) // Default to center if no move found
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
    /// Create a new `WasmGomoku` wrapping the core game logic.
    pub fn new() -> WasmGomoku {
        WasmGomoku {
            inner: Gomoku::new(),
        }
    }

    /// Flatten the internal board to a simple array for JavaScript.
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

    /// Return the active player as a numeric value used by the JS side.
    pub fn current_player(&self) -> u8 {
        match self.inner.current_player {
            Cell::Black => 1,
            Cell::White => 2,
            _ => 0,
        }
    }

    /// Wrapper around [`Gomoku::make_move`] that exposes a boolean result.
    pub fn make_move(&mut self, row: usize, col: usize) -> bool {
        self.inner.make_move(row, col).is_ok()
    }

    /// Compute the AI's move and return it as a two-element JS array.
    pub fn ai_move(&mut self) -> js_sys::Array {
        let (r, c) = self.inner.ai_move();
        let arr = js_sys::Array::new();
        arr.push(&JsValue::from_f64(r as f64));
        arr.push(&JsValue::from_f64(c as f64));
        arr
    }

    /// Translate the winner check into a numeric value for JavaScript.
    pub fn check_winner(&self) -> u8 {
        match self.inner.check_winner() {
            Some(Cell::Black) => 1,
            Some(Cell::White) => 2,
            _ => 0,
        }
    }

    /// Expose whether the board is completely filled.
    pub fn is_board_full(&self) -> bool {
        self.inner.is_board_full()
    }

    /// Switch the active player.
    pub fn switch_player(&mut self) {
        self.inner.switch_player();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Ensure a newly created board contains only empty cells and that
    /// the starting player is Black.
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
    /// Verify that placing a stone succeeds and that `switch_player`
    /// correctly toggles the active color.
    fn make_move_and_switch_player() {
        let mut game = Gomoku::new();
        game.make_move(0, 0).unwrap();
        assert_eq!(game.board[0][0], Cell::Black);
        game.switch_player();
        assert_eq!(game.current_player, Cell::White);
    }

    #[test]
    /// Attempt to play outside the board bounds should return an error.
    fn invalid_move_out_of_bounds() {
        let mut game = Gomoku::new();
        assert!(game.make_move(BOARD_SIZE, BOARD_SIZE).is_err());
    }

    #[test]
    /// Confirm horizontal sequences are detected as wins.
    fn detect_horizontal_win() {
        let mut game = Gomoku::new();
        for col in 0..WIN_LENGTH {
            game.make_move(0, col).unwrap();
        }
        assert_eq!(game.check_winner(), Some(Cell::Black));
    }

    #[test]
    /// Confirm diagonal sequences are detected as wins.
    fn detect_diagonal_win() {
        let mut game = Gomoku::new();
        for i in 0..WIN_LENGTH {
            game.make_move(i, i).unwrap();
        }
        assert_eq!(game.check_winner(), Some(Cell::Black));
    }

    #[test]
    /// Fill the board to verify draw detection when no moves remain.
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

    #[test]
    /// Scores should favor the supplied player.
    fn evaluation_respects_perspective() {
        let mut game = Gomoku::new();
        game.board[7][5] = Cell::White;
        game.board[7][6] = Cell::White;

        let white_score = game.evaluate(Cell::White);
        let black_score = game.evaluate(Cell::Black);
        assert!(white_score > 0);
        assert_eq!(white_score, -black_score);
    }

    #[test]
    /// Winning evaluations should outrank non-winning positions.
    fn win_scores_highest() {
        let mut four = Gomoku::new();
        for col in 0..4 {
            four.board[0][col] = Cell::Black;
        }
        let four_score = four.evaluate(Cell::Black);

        let mut five = Gomoku::new();
        for col in 0..5 {
            five.board[0][col] = Cell::Black;
        }
        let win_score = five.evaluate(Cell::Black);

        assert!(win_score > four_score);
        assert!(win_score >= 100000);
    }

    #[test]
    /// AI should select the immediate winning move when available.
    fn ai_makes_winning_move() {
        let mut game = Gomoku::new();
        for col in 0..4 {
            game.board[0][col] = Cell::Black;
        }

        let (row, col) = game.ai_move();
        assert_eq!((row, col), (0, 4));
    }

    #[test]
    /// Evaluations must account for diagonal lines of stones.
    fn evaluate_diagonal_sequences() {
        let mut game = Gomoku::new();
        for i in 0..3 {
            game.board[3 + i][3 + i] = Cell::Black;
        }

        let black_score = game.evaluate(Cell::Black);
        let white_score = game.evaluate(Cell::White);
        let (row, col) = game.ai_move();

        assert!(
            (row, col) == (7, 7) || (row, col) == (2, 2),
            "Expected (row, col) to be (7, 7) or (2, 2), but got {:?}",
            (row, col)
        );
        assert!(black_score > 0);
        assert_eq!(black_score, -white_score);
        assert_eq!(black_score, 300);
    }

    #[test]
    /// Confirm counter diagonal sequences are detected as wins.
    fn detect_counter_diagonal_win() {
        let mut game = Gomoku::new();
        for i in 0..WIN_LENGTH {
            game.make_move(i, WIN_LENGTH - 1 - i).unwrap();
        }
        assert_eq!(game.check_winner(), Some(Cell::Black));
    }

    #[test]
    /// Confirm vertical sequences are detected as wins.
    fn detect_vertical_win() {
        let mut game = Gomoku::new();
        for row in 0..WIN_LENGTH {
            game.make_move(row, 0).unwrap();
        }
        assert_eq!(game.check_winner(), Some(Cell::Black));
    }
}
