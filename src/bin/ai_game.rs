use rusty_chess::board::{Board, ChessMove, ChessMoveState, Color};
use rusty_chess::search::Minimax;
use std::io::{self, Write};

struct AiGame {
    board: Board,
    move_history: Vec<ChessMoveState>,
    player_color: Color,
    ai_depth: u8,
}

impl AiGame {
    fn new(player_color: Color, ai_depth: u8) -> Self {
        Self {
            board: Board::new(),
            move_history: Vec::new(),
            player_color,
            ai_depth,
        }
    }

    fn run(&mut self) {
        println!("Welcome to Rusty Chess - Play Against AI!");
        println!("You are playing as {:?}", self.player_color);
        println!("Enter moves in the format: e2,e4");
        println!("Type 'moves' to show all possible moves");
        println!("Type 'undo' to undo the last move (yours and AI's)");
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

            // Determine if it's the player's turn or AI's turn
            if self.board.side_to_move == self.player_color {
                // Player's turn
                if !self.handle_player_turn() {
                    break; // Player chose to quit
                }
            } else {
                // AI's turn
                self.handle_ai_turn();
            }
        }
    }

    fn handle_player_turn(&mut self) -> bool {
        print!("{:?} to move (You): ", self.board.side_to_move);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        let input = input.trim();

        match input {
            "quit" => {
                println!("Thanks for playing!");
                return false;
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

        true
    }

    fn handle_ai_turn(&mut self) {
        println!("{:?} to move (AI): Thinking...", self.board.side_to_move);

        match Minimax::find_best_move(&self.board, self.ai_depth) {
            Some(best_move) => {
                let from_notation = square_to_notation(best_move.from);
                let to_notation = square_to_notation(best_move.to);
                println!("AI plays: {},{}\n", from_notation, to_notation);

                let state = self.board.apply_move(best_move);
                self.move_history.push(state);
            }
            None => {
                println!("AI has no legal moves!");
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
        // Check if we have at least 2 moves (one player move + one AI move)
        if self.move_history.len() < 2 {
            return false;
        }

        // Undo the last move (AI's move)
        if let Some(ai_state) = self.move_history.pop() {
            self.board.undo_move(ai_state);

            // Also undo the player's move before that
            if let Some(player_state) = self.move_history.pop() {
                self.board.undo_move(player_state);
                return true;
            }
        }

        false
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
        if !move_strings.len().is_multiple_of(moves_per_line) {
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

fn get_player_color() -> Color {
    loop {
        print!("Choose your color (w/b): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        match input.trim().to_lowercase().as_str() {
            "w" | "white" => return Color::White,
            "b" | "black" => return Color::Black,
            _ => println!("Invalid choice. Please enter 'w' for white or 'b' for black."),
        }
    }
}

fn get_ai_depth() -> u8 {
    loop {
        print!("Choose AI difficulty (depth 1-10, recommended 5): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        match input.trim().parse::<u8>() {
            Ok(depth) if (1..=10).contains(&depth) => return depth,
            _ => println!("Invalid choice. Please enter a number between 1 and 10."),
        }
    }
}

fn main() {
    let player_color = get_player_color();
    let ai_depth = get_ai_depth();

    let mut game = AiGame::new(player_color, ai_depth);

    // If player chose black, AI makes the first move
    if player_color == Color::Black {
        println!("\nAI will make the first move as White.\n");
    }

    game.run();
}
