use rusty_chess::board::{Board, ChessMove, ChessMoveState, Color, print_board};
use rusty_chess::eval::Evaluator;
use rusty_chess::metrics::{AiMoveMetrics, GameRecorder, GameResult};
use rusty_chess::movegen::MoveGenerator;
use rusty_chess::search::{ChessEngine, SearchParams};
use std::env;
use std::io::{self, Write};

#[derive(Clone, Default)]
struct DisplaySettings {
    show_search_stats: bool,
    show_tt_info: bool,
    show_eval: bool,
    show_move_analysis: bool,
}

impl DisplaySettings {
    fn from_args() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut settings = Self::default();

        for arg in &args[1..] {
            match arg.as_str() {
                "--stats" | "-s" => settings.show_search_stats = true,
                "--tt" | "-t" => settings.show_tt_info = true,
                "--eval" | "-e" => settings.show_eval = true,
                "--analysis" | "-a" => settings.show_move_analysis = true,
                "--verbose" | "-v" => {
                    settings.show_search_stats = true;
                    settings.show_tt_info = true;
                    settings.show_eval = true;
                    settings.show_move_analysis = true;
                }
                "--help" | "-h" => {
                    print_usage();
                    std::process::exit(0);
                }
                _ => {}
            }
        }

        settings
    }

    fn any_enabled(&self) -> bool {
        self.show_search_stats || self.show_tt_info || self.show_eval || self.show_move_analysis
    }
}

fn print_usage() {
    println!(
        r#"
Rusty Chess - A terminal chess engine written in Rust

USAGE:
    rusty-chess [OPTIONS]

OPTIONS:
    -s, --stats      Show search statistics (time, nodes, depth)
    -t, --tt         Show transposition table information
    -e, --eval       Show position evaluation before/after moves
    -a, --analysis   Show move analysis (position change, material delta)
    -v, --verbose    Enable all display options
    -h, --help       Print this help message

EXAMPLES:
    rusty-chess                  Run with minimal output (default)
    rusty-chess --verbose        Run with all performance insights enabled
    rusty-chess -s -e            Show search stats and evaluation only

IN-GAME COMMANDS:
    You can also toggle these options during gameplay using:
    stats, tt, eval, analysis, verbose
"#
    );
}

enum PlayerAction {
    Continue,
    Quit,
    Resign,
}

struct ChessEngineSettings {
    player_color: Color,
    search_depth: u8,
    starting_position: Board,
}

struct AiGame {
    board: Board,
    move_history: Vec<ChessMoveState>,
    player_color: Color,
    search_params: SearchParams,
    engine: ChessEngine,
    evaluator: Evaluator,
    game_recorder: GameRecorder,
    move_counter: u16,
    display: DisplaySettings,
}

impl AiGame {
    fn new(
        player_color: Color,
        ai_depth: u8,
        starting_board: Board,
        display: DisplaySettings,
    ) -> Self {
        // Create search parameters with time based on depth
        // Higher depths get more time: depth * 1000ms
        let min_search_time_ms: u64 = (ai_depth as u64) * 2000;
        let search_params: SearchParams = SearchParams {
            max_depth: ai_depth,
            min_search_time_ms,
        };

        Self {
            board: starting_board,
            move_history: Vec::new(),
            player_color,
            search_params,
            engine: ChessEngine::with_london_system(),
            evaluator: Evaluator::new(),
            game_recorder: GameRecorder::new(player_color, ai_depth),
            move_counter: 0,
            display,
        }
    }

    fn run(&mut self) {
        let mut game_result = GameResult::InProgress;
        let mut player_quit = false;

        loop {
            print_board(&self.board);

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
            "fen" => {
                println!("FEN: {}", self.board.to_fen());
            }
            "eval" => {
                self.print_evaluation();
            }
            "stats" => {
                self.display.show_search_stats = !self.display.show_search_stats;
                println!(
                    "Search statistics: {}",
                    if self.display.show_search_stats {
                        "ON"
                    } else {
                        "OFF"
                    }
                );
            }
            "tt" => {
                self.display.show_tt_info = !self.display.show_tt_info;
                println!(
                    "Transposition table info: {}",
                    if self.display.show_tt_info {
                        "ON"
                    } else {
                        "OFF"
                    }
                );
            }
            "evalon" => {
                self.display.show_eval = !self.display.show_eval;
                println!(
                    "Evaluation display: {}",
                    if self.display.show_eval { "ON" } else { "OFF" }
                );
            }
            "analysis" => {
                self.display.show_move_analysis = !self.display.show_move_analysis;
                println!(
                    "Move analysis: {}",
                    if self.display.show_move_analysis {
                        "ON"
                    } else {
                        "OFF"
                    }
                );
            }
            "verbose" => {
                let enable = !self.display.any_enabled();
                self.display.show_search_stats = enable;
                self.display.show_tt_info = enable;
                self.display.show_eval = enable;
                self.display.show_move_analysis = enable;
                println!("All display options: {}", if enable { "ON" } else { "OFF" });
            }
            "display" => {
                self.print_display_status();
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

        // Capture evaluation before the move
        let before_eval = if self.display.show_eval || self.display.show_move_analysis {
            Some(self.evaluator.evaluate_detailed(&self.board))
        } else {
            None
        };

        match self
            .engine
            .find_best_move_iterative(&self.board, &self.search_params)
        {
            Some(best_move) => {
                let from_notation = square_to_notation(best_move.from);
                let to_notation = square_to_notation(best_move.to);
                let move_notation = format!("{}-{}", from_notation, to_notation);

                println!("AI plays: {},{}", from_notation, to_notation);
                if best_move.capture {
                    println!("  (capture)");
                }
                println!();

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

                    // Display search statistics
                    if self.display.show_search_stats {
                        println!("=== Search Statistics ===");
                        println!("  Time: {:.2}s", search_metrics.search_time.as_secs_f64());
                        println!("  Nodes: {} ({} n/s)", search_metrics.nodes_explored, nps);
                        println!("  Depth reached: {}", search_metrics.max_depth_reached);
                        println!();
                    }

                    // Display transposition table info
                    if self.display.show_tt_info {
                        println!("=== Transposition Table ===");
                        println!("  Entries: {}", self.engine.get_tt_num_entries());
                        println!("  Size: {} KB", self.engine.get_tt_size_bytes() / 1024);
                        println!("  Hits: {} | Misses: {}", tt_hits, tt_misses);
                        println!("  Hit rate: {:.1}%", tt_hit_rate);
                        println!();
                    }
                }

                // Make the move on a copy first to get after evaluation
                let mut board_after = self.board;
                board_after.make_move(best_move);

                // Show evaluation breakdown if enabled
                if let Some(ref before) = before_eval {
                    let after_eval = self.evaluator.evaluate_detailed(&board_after);

                    if self.display.show_eval {
                        println!(
                            "=== Before {},{} Evaluation ===",
                            from_notation, to_notation
                        );
                        println!("{}", before);
                        println!();

                        println!("=== After {},{} Evaluation ===", from_notation, to_notation);
                        println!("{}", after_eval);
                        println!();
                    }

                    // Show move analysis if enabled
                    if self.display.show_move_analysis {
                        let delta = after_eval.total - before.total;
                        let improvement = match ai_color {
                            Color::White => delta,
                            Color::Black => -delta,
                        };
                        println!("=== Move Analysis ===");
                        println!(
                            "  Position change: {:+} cp (for {:?})",
                            improvement, ai_color
                        );

                        // Highlight significant changes
                        let mat_delta = after_eval.material - before.material;
                        if mat_delta != 0 {
                            println!("  Material change: {:+} cp", mat_delta);
                        }
                        let threat_delta = after_eval.threat - before.threat;
                        if threat_delta.abs() >= 30 {
                            println!("  Threat change:   {:+} cp", threat_delta);
                        }
                        let mobility_delta = after_eval.mobility - before.mobility;
                        if mobility_delta.abs() >= 20 {
                            println!("  Mobility change: {:+} cp", mobility_delta);
                        }
                        let king_safety_delta = after_eval.king_safety - before.king_safety;
                        if king_safety_delta.abs() >= 30 {
                            println!("  King safety:     {:+} cp", king_safety_delta);
                        }
                        let forcing_delta = after_eval.forcing_moves - before.forcing_moves;
                        if forcing_delta.abs() >= 30 {
                            println!("  Forcing moves:   {:+} cp", forcing_delta);
                        }
                        println!();
                    }
                }

                let state = self.board.make_move(best_move);
                self.move_history.push(state);
                println!("FEN: {}", self.board.to_fen());
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
        let state = self.board.make_move(chess_move);
        self.move_history.push(state);
        println!("FEN: {}", self.board.to_fen());

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
            self.board.unmake_move(ai_state);

            // Also undo the player's move before that
            if let Some(player_state) = self.move_history.pop() {
                self.board.unmake_move(player_state);
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

    fn print_evaluation(&self) {
        println!("\n=== Position Evaluation ===");
        let breakdown = self.evaluator.evaluate_detailed(&self.board);
        println!("{}", breakdown);

        // Interpret the score
        let interpretation = match breakdown.total {
            t if t > 300 => "White has a decisive advantage",
            t if t > 150 => "White has a clear advantage",
            t if t > 50 => "White has a slight advantage",
            t if t > -50 => "Position is roughly equal",
            t if t > -150 => "Black has a slight advantage",
            t if t > -300 => "Black has a clear advantage",
            _ => "Black has a decisive advantage",
        };
        println!("\n  Assessment: {}", interpretation);
        println!();
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

    fn print_display_status(&self) {
        println!("\n=== Display Settings ===");
        println!(
            "  stats    - Search statistics:      {}",
            if self.display.show_search_stats {
                "ON"
            } else {
                "OFF"
            }
        );
        println!(
            "  tt       - Transposition table:    {}",
            if self.display.show_tt_info {
                "ON"
            } else {
                "OFF"
            }
        );
        println!(
            "  evalon   - Evaluation display:     {}",
            if self.display.show_eval { "ON" } else { "OFF" }
        );
        println!(
            "  analysis - Move analysis:          {}",
            if self.display.show_move_analysis {
                "ON"
            } else {
                "OFF"
            }
        );
        println!("  verbose  - Toggle all on/off");
        println!();
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

fn display_introduction() {
    println!(
        r#"
    ╔═══════════════════════════════════════════════════════════════╗
    ║                                                               ║
    ║   ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜      ____  _   _ ____ _______   __          ║
    ║   ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟     |  _ \| | | / ___|_   _\ \ / /          ║
    ║   . . . . . . . .     | |_) | | | \___ \ | |  \ V /           ║
    ║   . . . . . . . .     |  _ <| |_| |___) || |   | |            ║
    ║   . . . . . . . .     |_| \_\\___/|____/ |_|   |_|            ║
    ║   . . . . . . . .                                             ║
    ║   ♙ ♙ ♙ ♙ ♙ ♙ ♙ ♙       ____ _   _ _____ ____ ____            ║
    ║   ♖ ♘ ♗ ♕ ♔ ♗ ♘ ♖      / ___| | | | ____/ ___/ ___|           ║
    ║                       | |   | |_| |  _| \___ \___ \           ║
    ║                       | |___|  _  | |___ ___) |__) |          ║
    ║                        \____|_| |_|_____|____/____/           ║
    ║                                                               ║
    ╠═══════════════════════════════════════════════════════════════╣
    ║                                                               ║
    ║          A terminal chess engine written in Rust              ║
    ║                                                               ║
    ║    Features:                                                  ║
    ║      - Minimax search with alpha-beta pruning                 ║
    ║      - Quiescence search for tactical accuracy                ║
    ║      - Opening book support                                   ║
    ║      - Full move validation and game state tracking           ║
    ║                                                               ║
    ╚═══════════════════════════════════════════════════════════════╝
"#
    );
}

fn get_chess_engine_settings() -> ChessEngineSettings {
    let player_color: Color = get_player_color();
    let search_depth: u8 = get_search_depth();
    let starting_position: Board = get_starting_position();

    ChessEngineSettings {
        player_color,
        search_depth,
        starting_position,
    }
}

fn display_instructions(settings: &ChessEngineSettings, display: &DisplaySettings) {
    let color_str = match settings.player_color {
        Color::White => "White",
        Color::Black => "Black",
    };

    println!("┌─────────────────────────────────────────┐");
    println!("│            Game Settings                │");
    println!("├─────────────────────────────────────────┤");
    println!("│  Player color:     {:>19}  │", color_str);
    println!("│  AI search depth:  {:>15} ply  │", settings.search_depth);
    println!("├─────────────────────────────────────────┤");
    println!("│            Commands                     │");
    println!("├─────────────────────────────────────────┤");
    println!("│  e2,e4    - Make a move (from,to)       │");
    println!("│  moves    - Show all legal moves        │");
    println!("│  undo     - Undo last move pair         │");
    println!("│  fen      - Show current FEN            │");
    println!("│  eval     - Show position evaluation    │");
    println!("│  resign   - Resign the game             │");
    println!("│  quit     - Exit the game               │");
    println!("├─────────────────────────────────────────┤");
    println!("│       Display Toggles (for devs)        │");
    println!("├─────────────────────────────────────────┤");
    println!(
        "│  stats    - Search statistics     [{}] │",
        if display.show_search_stats {
            "ON "
        } else {
            "OFF"
        }
    );
    println!(
        "│  tt       - Transposition table   [{}] │",
        if display.show_tt_info { "ON " } else { "OFF" }
    );
    println!(
        "│  evalon   - Eval before/after     [{}] │",
        if display.show_eval { "ON " } else { "OFF" }
    );
    println!(
        "│  analysis - Move analysis         [{}] │",
        if display.show_move_analysis {
            "ON "
        } else {
            "OFF"
        }
    );
    println!("│  verbose  - Toggle all on/off           │");
    println!("│  display  - Show current settings       │");
    println!("└─────────────────────────────────────────┘");
    println!();
}

fn get_player_color() -> Color {
    println!("┌─────────────────────────────────────────┐");
    println!("│         Choose Your Color               │");
    println!("├─────────────────────────────────────────┤");
    println!("│  [w] White  ♔  - Move first             │");
    println!("│  [b] Black  ♚  - AI moves first         │");
    println!("└─────────────────────────────────────────┘");

    loop {
        print!("  > ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        match input.trim().to_lowercase().as_str() {
            "w" | "white" => {
                println!("  ✓ Playing as White\n");
                return Color::White;
            }
            "b" | "black" => {
                println!("  ✓ Playing as Black\n");
                return Color::Black;
            }
            _ => println!("  ✗ Invalid choice. Enter 'w' or 'b'."),
        }
    }
}

fn get_search_depth() -> u8 {
    println!("┌─────────────────────────────────────────┐");
    println!("│         AI Difficulty (1-10)            │");
    println!("├─────────────────────────────────────────┤");
    println!("│  1-3   Beginner    - Fast, weak play    │");
    println!("│  4-5   Intermediate - Balanced          │");
    println!("│  6-7   Advanced    - Strong, slower     │");
    println!("│  8-10  Expert      - Very strong        │");
    println!("├─────────────────────────────────────────┤");
    println!("│  Recommended: 5                         │");
    println!("└─────────────────────────────────────────┘");

    loop {
        print!("  > ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        match input.trim().parse::<u8>() {
            Ok(depth) if (1..=10).contains(&depth) => {
                let difficulty = match depth {
                    1..=3 => "Beginner",
                    4..=5 => "Intermediate",
                    6..=7 => "Advanced",
                    _ => "Expert",
                };
                println!("  ✓ Difficulty: {} (depth {})\n", difficulty, depth);
                return depth;
            }
            _ => println!("  ✗ Invalid choice. Enter a number 1-10."),
        }
    }
}

fn get_starting_position() -> Board {
    println!("┌─────────────────────────────────────────┐");
    println!("│        Starting Position                │");
    println!("├─────────────────────────────────────────┤");
    println!("│  [n] Standard - Normal chess setup      │");
    println!("│  [y] Custom   - Load from FEN string    │");
    println!("└─────────────────────────────────────────┘");

    loop {
        print!("  > ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        match input.trim().to_lowercase().as_str() {
            "n" | "no" => {
                println!("  ✓ Using standard starting position\n");
                return Board::startpos();
            }
            "y" | "yes" => {
                return get_fen_position();
            }
            _ => println!("  ✗ Invalid choice. Enter 'y' or 'n'."),
        }
    }
}

fn get_fen_position() -> Board {
    println!("┌─────────────────────────────────────────┐");
    println!("│          Enter FEN Position             │");
    println!("├─────────────────────────────────────────┤");
    println!("│  Enter a valid FEN string to load a     │");
    println!("│  custom position. Type 'cancel' to      │");
    println!("│  use the standard starting position.    │");
    println!("├─────────────────────────────────────────┤");
    println!("│  Example FEN (starting position):       │");
    println!("│  rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/    │");
    println!("│  RNBQKBNR w KQkq - 0 1                  │");
    println!("└─────────────────────────────────────────┘");

    loop {
        print!("  FEN> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        let input = input.trim();

        if input.to_lowercase() == "cancel" {
            println!("  ✓ Using standard starting position\n");
            return Board::startpos();
        }

        // Try to parse the FEN
        let board = Board::from_fen(input);

        // Validate the board has kings for both sides
        if board.king_sq[0] >= 64 || board.king_sq[1] >= 64 {
            println!("  ✗ Invalid FEN: Both sides must have a king.");
            continue;
        }

        println!("  ✓ FEN loaded successfully!\n");
        return board;
    }
}

fn main() {
    // Parse command-line arguments for display settings
    let display_settings = DisplaySettings::from_args();

    display_introduction();
    let settings: ChessEngineSettings = get_chess_engine_settings();
    display_instructions(&settings, &display_settings);

    let mut game: AiGame = AiGame::new(
        settings.player_color,
        settings.search_depth,
        settings.starting_position,
        display_settings,
    );

    // If player chose black and it's white's turn, AI makes the first move
    if settings.player_color == Color::Black && game.board.side_to_move == Color::White {
        println!("\nAI will make the first move as White.\n");
    }

    game.run();
}
