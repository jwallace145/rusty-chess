use rusty_chess::board::{Board2, ChessMove, ChessMoveState, Color};
use rusty_chess::metrics::{AiMoveMetrics, GameRecorder, GameResult};
use rusty_chess::movegen::MoveGenerator;
use rusty_chess::search::{ChessEngine, SearchParams};
use std::io::{self, Write};

enum PlayerAction {
    Continue,
    Quit,
    Resign,
}

struct AiGame {
    board: Board2,
    move_history: Vec<ChessMoveState>,
    player_color: Color,
    search_params: SearchParams,
    engine: ChessEngine,
    game_recorder: GameRecorder,
    move_counter: u16,
}

impl AiGame {
    fn new(player_color: Color, ai_depth: u8) -> Self {
        // Create search parameters with time based on depth
        // Higher depths get more time: depth * 1000ms
        let min_search_time_ms = (ai_depth as u64) * 2000;
        let search_params = SearchParams {
            max_depth: ai_depth,
            min_search_time_ms,
        };

        Self {
            board: Board2::new_standard(),
            move_history: Vec::new(),
            player_color,
            search_params,
            engine: ChessEngine::with_opening_book("./opening_book.bin")
                .expect("Failed to load opening book"),
            game_recorder: GameRecorder::new(player_color, ai_depth),
            move_counter: 0,
        }
    }

    fn run(&mut self) {
        println!("Welcome to Rusty Chess - Play Against AI!");
        println!("You are playing as {:?}", self.player_color);
        println!("Enter moves in the format: e2,e4");
        println!("Type 'moves' to show all possible moves");
        println!("Type 'undo' to undo the last move (yours and AI's)");
        println!("Type 'resign' to resign the game");
        println!("Type 'quit' to exit the game\n");

        let mut game_result = GameResult::InProgress;
        let mut player_quit = false;

        loop {
            self.board.print();

            // Check for checkmate
            if self.is_checkmate() {
                let winner = self.board.side_to_move.opponent();
                println!("\nCheckmate! {:?} wins!", winner);

                game_result = if winner == self.player_color {
                    GameResult::PlayerWin
                } else {
                    GameResult::AIWin
                };
                break;
            }

            // Check for stalemate
            if self.is_stalemate() {
                println!("\nStalemate! The game is a draw.");
                game_result = GameResult::Draw;
                break;
            }

            // Determine if it's the player's turn or AI's turn
            if self.board.side_to_move == self.player_color {
                // Player's turn
                match self.handle_player_turn() {
                    PlayerAction::Continue => {}
                    PlayerAction::Quit => {
                        player_quit = true;
                        break;
                    }
                    PlayerAction::Resign => {
                        println!("\nYou have resigned. AI wins!");
                        game_result = GameResult::AIWin;
                        break;
                    }
                }
            } else {
                // AI's turn
                self.handle_ai_turn();
            }
        }

        // Save game recording
        if !player_quit {
            match self.game_recorder.finalize_and_save(game_result) {
                Ok(filename) => {
                    println!("\nGame recorded successfully: {}", filename);
                }
                Err(e) => {
                    eprintln!("Error saving game recording: {}", e);
                }
            }
        }
    }

    fn handle_player_turn(&mut self) -> PlayerAction {
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
                return PlayerAction::Quit;
            }
            "resign" => {
                return PlayerAction::Resign;
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

        PlayerAction::Continue
    }

    fn handle_ai_turn(&mut self) {
        println!(
            "{:?} to move (AI): Thinking (max depth: {}, min time: {}ms)...",
            self.board.side_to_move,
            self.search_params.max_depth,
            self.search_params.min_search_time_ms
        );

        let ai_color = self.board.side_to_move;

        match self
            .engine
            .find_best_move_iterative(&self.board, &self.search_params)
        {
            Some(best_move) => {
                let from_notation = square_to_notation(best_move.from);
                let to_notation = square_to_notation(best_move.to);
                let move_notation = format!("{}-{}", from_notation, to_notation);

                println!("AI plays: {},{}\n", from_notation, to_notation);

                // Capture AI metrics
                if let Some(search_metrics) = self.engine.get_last_search_metrics() {
                    self.move_counter += 1;

                    let nps = if search_metrics.search_time.as_secs_f64() > 0.0 {
                        (search_metrics.nodes_explored as f64
                            / search_metrics.search_time.as_secs_f64())
                            as u64
                    } else {
                        0
                    };

                    let beta_cutoff_percentage = if search_metrics.nodes_explored > 0 {
                        (search_metrics.beta_cutoffs as f64 / search_metrics.nodes_explored as f64)
                            * 100.0
                    } else {
                        0.0
                    };

                    let tt_hits = self.engine.get_tt_hits();
                    let tt_misses = self.engine.get_tt_misses();
                    let tt_hit_rate = if tt_hits + tt_misses > 0 {
                        (tt_hits as f64 / (tt_hits + tt_misses) as f64) * 100.0
                    } else {
                        0.0
                    };

                    let ai_metrics = AiMoveMetrics {
                        search_time_ms: search_metrics.search_time.as_millis(),
                        nodes_explored: search_metrics.nodes_explored,
                        nodes_per_second: nps,
                        beta_cutoffs: search_metrics.beta_cutoffs,
                        beta_cutoff_percentage,
                        max_depth_reached: search_metrics.max_depth_reached,
                        tt_size_bytes: self.engine.get_tt_size_bytes(),
                        tt_num_entries: self.engine.get_tt_num_entries(),
                        tt_hits,
                        tt_misses,
                        tt_hit_rate_percentage: tt_hit_rate,
                    };

                    self.game_recorder.record_ai_move(
                        self.move_counter,
                        ai_color,
                        move_notation,
                        ai_metrics,
                    );
                }

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

        // Capture player move notation
        let player_color = self.board.side_to_move;
        let from_notation = square_to_notation(chess_move.from);
        let to_notation = square_to_notation(chess_move.to);
        let move_notation = format!("{}-{}", from_notation, to_notation);

        self.move_counter += 1;
        self.game_recorder
            .record_player_move(self.move_counter, player_color, move_notation);

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
        let mut legal_moves = Vec::with_capacity(128);
        MoveGenerator::generate_legal_moves(&self.board, &mut legal_moves);
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
        MoveGenerator::is_checkmate(&self.board)
    }

    fn is_stalemate(&self) -> bool {
        MoveGenerator::is_stalemate(&self.board)
    }

    fn print_legal_moves(&self) {
        let mut legal_moves = Vec::with_capacity(128);
        MoveGenerator::generate_legal_moves(&self.board, &mut legal_moves);

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
