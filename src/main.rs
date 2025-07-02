use gomoku::{Gomoku, Cell, BOARD_SIZE};
use std::io;

fn main() {
    let mut game = Gomoku::new();
    println!("Welcome to Gomoku! You are Black (X), AI is White (O).");
    println!("Enter moves as 'row col' (e.g., '7 7').");

    loop {
        game.print_board();
        if game.current_player() == Cell::Black {
            println!("Your turn (Black, X). Enter row and column (0-{}):", BOARD_SIZE - 1);
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Failed to read input");
            let coords: Vec<usize> = input
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if coords.len() != 2 {
                println!("Invalid input. Please enter two numbers (row col).");
                continue;
            }
            let (row, col) = (coords[0], coords[1]);
            if game.make_move(row, col).is_err() {
                println!("Invalid move");
                continue;
            }
        } else {
            println!("AI (White, O) is thinking...");
            let (row, col) = game.ai_move();
            println!("AI moves to ({}, {})", row, col);
            game.make_move(row, col).expect("AI made an invalid move");
        }

        if let Some(winner) = game.check_winner() {
            game.print_board();
            match winner {
                Cell::Black => println!("You win (Black, X)!"),
                Cell::White => println!("AI wins (White, O)!"),
                _ => unreachable!(),
            }
            break;
        }
        if game.is_board_full() {
            game.print_board();
            println!("Game is a draw!");
            break;
        }
        game.switch_player();
    }
}
