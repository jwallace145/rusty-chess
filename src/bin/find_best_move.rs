use rusty_chess::board::{Board, print_board};
use rusty_chess::eval::Evaluator;
use rusty_chess::search::{ChessEngine, SearchParams};
use std::env;
use std::process;

const DEFAULT_MAX_DEPTH: u8 = 5;
const DEFAULT_MIN_SEARCH_TIME_MS: u64 = 16000;

fn print_usage(program_name: &str) {
    eprintln!("Usage: {} <fen> [options]", program_name);
    eprintln!();
    eprintln!("Arguments:");
    eprintln!("  <fen>                   Chess position in FEN notation (required)");
    eprintln!();
    eprintln!("Options:");
    eprintln!(
        "  --depth <n>             Maximum search depth (default: {})",
        DEFAULT_MAX_DEPTH
    );
    eprintln!(
        "  --time <ms>             Minimum search time in milliseconds (default: {})",
        DEFAULT_MIN_SEARCH_TIME_MS
    );
    eprintln!("  --no-book               Disable opening book lookup");
    eprintln!(
        "  --book <path>           Path to opening book file (falls back to built-in London System)"
    );
    eprintln!("  --quiet                 Only output the best move, no statistics");
    eprintln!("  --help                  Show this help message");
    eprintln!();
    eprintln!("Examples:");
    eprintln!(
        "  {} \"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1\"",
        program_name
    );
    eprintln!(
        "  {} \"rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1\" --depth 7",
        program_name
    );
    eprintln!(
        "  {} \"r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3\" --time 5000",
        program_name
    );
}

fn square_to_notation(square: usize) -> String {
    let file = (square % 8) as u8;
    let rank = (square / 8) as u8;

    let file_char = (b'a' + file) as char;
    let rank_char = (b'1' + rank) as char;

    format!("{}{}", file_char, rank_char)
}

struct Config {
    fen: String,
    max_depth: u8,
    min_search_time_ms: u64,
    use_opening_book: bool,
    opening_book_path: String,
    quiet: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            fen: String::new(),
            max_depth: DEFAULT_MAX_DEPTH,
            min_search_time_ms: DEFAULT_MIN_SEARCH_TIME_MS,
            use_opening_book: true,
            opening_book_path: "./opening_book.bin".to_string(),
            quiet: false,
        }
    }
}

fn parse_args() -> Result<Config, String> {
    let args: Vec<String> = env::args().collect();
    let program_name = &args[0];

    if args.len() < 2 {
        print_usage(program_name);
        return Err("No FEN position provided".to_string());
    }

    let mut config = Config::default();
    let mut i = 1;

    // Check for help flag first
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_usage(program_name);
        process::exit(0);
    }

    // First non-flag argument is the FEN
    if !args[i].starts_with("--") {
        config.fen = args[i].clone();
        i += 1;
    } else {
        print_usage(program_name);
        return Err("FEN position must be the first argument".to_string());
    }

    // Parse optional flags
    while i < args.len() {
        match args[i].as_str() {
            "--depth" => {
                i += 1;
                if i >= args.len() {
                    return Err("--depth requires a value".to_string());
                }
                config.max_depth = args[i]
                    .parse()
                    .map_err(|_| format!("Invalid depth value: {}", args[i]))?;
                if config.max_depth == 0 || config.max_depth > 20 {
                    return Err("Depth must be between 1 and 20".to_string());
                }
            }
            "--time" => {
                i += 1;
                if i >= args.len() {
                    return Err("--time requires a value".to_string());
                }
                config.min_search_time_ms = args[i]
                    .parse()
                    .map_err(|_| format!("Invalid time value: {}", args[i]))?;
            }
            "--no-book" => {
                config.use_opening_book = false;
            }
            "--book" => {
                i += 1;
                if i >= args.len() {
                    return Err("--book requires a path".to_string());
                }
                config.opening_book_path = args[i].clone();
            }
            "--quiet" => {
                config.quiet = true;
            }
            _ => {
                return Err(format!("Unknown option: {}", args[i]));
            }
        }
        i += 1;
    }

    Ok(config)
}

fn main() {
    let config = match parse_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    // Parse the FEN and create the board
    let board = Board::from_fen(&config.fen);

    if !config.quiet {
        println!("Position:");
        print_board(&board);
        println!();
        println!("Side to move: {:?}", board.side_to_move);
        println!(
            "Search parameters: depth={}, min_time={}ms",
            config.max_depth, config.min_search_time_ms
        );
        println!();
    }

    // Create the engine
    let mut engine = if config.use_opening_book {
        // Try to load from file first, fall back to built-in London System book
        match ChessEngine::with_opening_book(&config.opening_book_path) {
            Ok(e) => e,
            Err(_) => {
                if !config.quiet {
                    eprintln!("Note: Using built-in London System opening book");
                }
                ChessEngine::with_london_system()
            }
        }
    } else {
        ChessEngine::new()
    };

    // Set up search parameters
    let search_params = SearchParams {
        max_depth: config.max_depth,
        min_search_time_ms: config.min_search_time_ms,
    };

    // Find the best move
    match engine.find_best_move_iterative(&board, &search_params) {
        Some(best_move) => {
            let from = square_to_notation(best_move.from());
            let to = square_to_notation(best_move.to());

            if config.quiet {
                println!("{}{}", from, to);
            } else {
                println!("Best move: {}{}", from, to);
                if board.piece_on(best_move.to() as u8).is_some() || best_move.is_en_passant() {
                    println!("Move type: Capture");
                }
                println!();

                // Show evaluation breakdown
                let evaluator = Evaluator::new();

                // Current position evaluation
                println!("=== Current Position Evaluation ===");
                let before_eval = evaluator.evaluate_detailed(&board);
                println!("{}", before_eval);
                println!();

                // Position after the best move
                let mut board_after = board;
                board_after.make_move(best_move);

                println!("=== After {}{} Evaluation ===", from, to);
                let after_eval = evaluator.evaluate_detailed(&board_after);
                println!("{}", after_eval);
                println!();

                // Show the delta (from the moving side's perspective)
                println!("=== Evaluation Change ===");
                let delta = after_eval.total - before_eval.total;
                // For White moving: positive delta = White gained
                // For Black moving: negative delta = Black gained (position worse for White)
                let side_moved = board.side_to_move;
                let improvement = match side_moved {
                    rusty_chess::board::Color::White => delta, // White wants position to become more positive
                    rusty_chess::board::Color::Black => -delta, // Black wants position to become more negative
                };
                println!(
                    "  Position change: {:+} cp (for {:?})",
                    improvement, side_moved
                );

                // Highlight significant changes
                let mat_delta = after_eval.material - before_eval.material;
                if mat_delta != 0 {
                    println!("  Material change: {:+} cp", mat_delta);
                }
                let threat_delta = after_eval.threat - before_eval.threat;
                if threat_delta.abs() >= 50 {
                    println!("  Threat change:   {:+} cp", threat_delta);
                }
                let mobility_delta = after_eval.mobility - before_eval.mobility;
                if mobility_delta.abs() >= 20 {
                    println!("  Mobility change: {:+} cp", mobility_delta);
                }
            }
        }
        None => {
            if config.quiet {
                println!("none");
            } else {
                println!("No legal moves available (checkmate or stalemate)");
            }
            process::exit(1);
        }
    }
}
