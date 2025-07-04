use gomoku::{Gomoku, Cell, BOARD_SIZE};
use std::io;

/// Entry point for the console version of the game.
///
/// Handles the game loop, user input and AI moves while printing the
/// board after each turn.
fn main() {
    let mut game = Gomoku::new();
    println!("Welcome to Gomoku!");
    println!("Do you want to move first? (y/n)");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    let human_first = input.trim().eq_ignore_ascii_case("y");

    let (human_color, ai_color) = if human_first {
        (Cell::Black, Cell::White)
    } else {
        (Cell::White, Cell::Black)
    };

    println!(
        "You are {:?} ({}), AI is {:?} ({})",
        human_color,
        if human_color == Cell::Black { 'X' } else { 'O' },
        ai_color,
        if ai_color == Cell::Black { 'X' } else { 'O' }
    );
    println!("Enter moves as 'row col' (e.g., '7 7').");

    if !human_first {
        println!("AI ({:?}) is thinking...", ai_color);
        let (row, col) = game.ai_move();
        println!("AI moves to ({}, {})", row, col);
        game.make_move(row, col).expect("AI made an invalid move");
        if let Some(winner) = game.check_winner() {
            game.print_board();
            println!("AI wins ({:?})!", winner);
            return;
        }
        if game.is_board_full() {
            game.print_board();
            println!("Game is a draw!");
            return;
        }
        game.switch_player();
    }

    loop {
        game.print_board();
        if game.current_player() == human_color {
            println!("Your turn ({:?}). Enter row and column (0-{}):", human_color, BOARD_SIZE - 1);
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
            println!("AI ({:?}) is thinking...", ai_color);
            let (row, col) = game.ai_move();
            println!("AI moves to ({}, {})", row, col);
            game.make_move(row, col).expect("AI made an invalid move");
        }

        if let Some(winner) = game.check_winner() {
            game.print_board();
            match winner {
                w if w == human_color => println!("You win ({:?})!", human_color),
                w if w == ai_color => println!("AI wins ({:?})!", ai_color),
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
