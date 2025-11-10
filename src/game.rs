use crate::board::{Board, ChessMove, ChessMoveState};
use std::io::{self, Write};

pub struct Game {
    board: Board,
    move_history: Vec<ChessMoveState>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            move_history: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        println!("Welcome to Rusty Chess!");
        println!("Enter moves in the format: e2,e4");
        println!("Type 'moves' to show all possible moves");
        println!("Type 'undo' to undo the last move");
        println!("Type 'quit' to exit the game\n");

        loop {
            self.board.print();

            // Check for checkmate
            if self.is_checkmate() {
                println!(
                    "\nCheckmate! {:?} wins!",
                    self.board.side_to_move.opponent()
                );
                break;
            }

            // Check for stalemate
            if self.is_stalemate() {
                println!("\nStalemate! The game is a draw.");
                break;
            }

            // Get user input
            print!("{:?} to move: ", self.board.side_to_move);
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");

            let input = input.trim();

            match input {
                "quit" => {
                    println!("Thanks for playing!");
                    break;
                }
                "undo" => {
                    if self.undo_move() {
                        println!("Move undone.");
                    } else {
                        println!("No moves to undo.");
                    }
                }
                "moves" => {
                    self.print_legal_moves();
                }
                _ => {
                    if let Err(e) = self.process_move(input) {
                        println!("Error: {}", e);
                    }
                }
            }
        }
    }

    fn process_move(&mut self, input: &str) -> Result<(), String> {
        // Parse the move
        let chess_move = self.parse_move(input)?;

        // Apply the move
        let state = self.board.apply_move(chess_move);
        self.move_history.push(state);

        Ok(())
    }

    fn parse_move(&self, input: &str) -> Result<ChessMove, String> {
        let parts: Vec<&str> = input.split(',').collect();
        if parts.len() != 2 {
            return Err("Invalid format. Use: e2,e4".to_string());
        }

        let from = parse_square(parts[0].trim())?;
        let to = parse_square(parts[1].trim())?;

        // Find the matching legal move
        let legal_moves = self.board.generate_legal_moves();
        legal_moves
            .into_iter()
            .find(|m| m.from == from && m.to == to)
            .ok_or_else(|| "Invalid or illegal move".to_string())
    }

    fn undo_move(&mut self) -> bool {
        if let Some(state) = self.move_history.pop() {
            self.board.undo_move(state);
            true
        } else {
            false
        }
    }

    fn is_checkmate(&self) -> bool {
        self.board.is_checkmate()
    }

    fn is_stalemate(&self) -> bool {
        self.board.is_stalemate()
    }

    fn print_legal_moves(&self) {
        let legal_moves = self.board.generate_legal_moves();

        if legal_moves.is_empty() {
            println!("No legal moves available.");
            return;
        }

        println!("\nPossible moves for {:?}:", self.board.side_to_move);

        // Convert moves to algebraic notation and group them
        let mut move_strings: Vec<String> = legal_moves
            .iter()
            .map(|m| {
                format!(
                    "{},{}",
                    square_to_notation(m.from),
                    square_to_notation(m.to)
                )
            })
            .collect();

        move_strings.sort();

        // Print in a nicely formatted grid
        let moves_per_line = 6;
        for (i, move_str) in move_strings.iter().enumerate() {
            print!("{:8}", move_str);
            if (i + 1) % moves_per_line == 0 {
                println!();
            }
        }

        // Add a newline if we didn't end on a complete line
        if move_strings.len() % moves_per_line != 0 {
            println!();
        }

        println!("Total: {} legal moves\n", legal_moves.len());
    }
}

fn parse_square(s: &str) -> Result<usize, String> {
    if s.len() != 2 {
        return Err(format!("Invalid square: {}", s));
    }

    let bytes = s.as_bytes();
    let file = bytes[0];
    let rank = bytes[1];

    if !(b'a'..=b'h').contains(&file) {
        return Err(format!("Invalid file: {}", file as char));
    }

    if !(b'1'..=b'8').contains(&rank) {
        return Err(format!("Invalid rank: {}", rank as char));
    }

    let file_idx = (file - b'a') as usize;
    let rank_idx = (rank - b'1') as usize;

    Ok(rank_idx * 8 + file_idx)
}

fn square_to_notation(square: usize) -> String {
    let file = (square % 8) as u8;
    let rank = (square / 8) as u8;

    let file_char = (b'a' + file) as char;
    let rank_char = (b'1' + rank) as char;

    format!("{}{}", file_char, rank_char)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_square() {
        assert_eq!(parse_square("a1"), Ok(0));
        assert_eq!(parse_square("h1"), Ok(7));
        assert_eq!(parse_square("a8"), Ok(56));
        assert_eq!(parse_square("h8"), Ok(63));
        assert_eq!(parse_square("e2"), Ok(12));
        assert_eq!(parse_square("e4"), Ok(28));
    }

    #[test]
    fn test_parse_square_invalid() {
        assert!(parse_square("i1").is_err());
        assert!(parse_square("a9").is_err());
        assert!(parse_square("a").is_err());
        assert!(parse_square("").is_err());
    }

    #[test]
    fn test_game_creation() {
        let game = Game::new();
        assert_eq!(game.move_history.len(), 0);
    }

    #[test]
    fn test_undo_empty() {
        let mut game = Game::new();
        assert!(!game.undo_move());
    }

    #[test]
    fn test_square_to_notation() {
        assert_eq!(square_to_notation(0), "a1");
        assert_eq!(square_to_notation(7), "h1");
        assert_eq!(square_to_notation(8), "a2");
        assert_eq!(square_to_notation(12), "e2");
        assert_eq!(square_to_notation(28), "e4");
        assert_eq!(square_to_notation(56), "a8");
        assert_eq!(square_to_notation(63), "h8");
    }
}
