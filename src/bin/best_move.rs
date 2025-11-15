use rusty_chess::board::Board;
use rusty_chess::search::ChessEngine;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Display usage if no arguments provided
    if args.len() < 2 {
        print_usage(&args[0]);
        return;
    }

    // Parse command-line arguments
    let mut fen = String::new();
    let mut depth = 5; // Default depth
    let mut opening_book_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--fen" | "-f" => {
                if i + 1 < args.len() {
                    fen = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --fen requires a FEN string");
                    print_usage(&args[0]);
                    return;
                }
            }
            "--depth" | "-d" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u8>() {
                        Ok(d) if d > 0 && d <= 20 => {
                            depth = d;
                            i += 2;
                        }
                        _ => {
                            eprintln!("Error: depth must be between 1 and 20");
                            return;
                        }
                    }
                } else {
                    eprintln!("Error: --depth requires a number");
                    print_usage(&args[0]);
                    return;
                }
            }
            "--book" | "-b" => {
                if i + 1 < args.len() {
                    opening_book_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --book requires a file path");
                    print_usage(&args[0]);
                    return;
                }
            }
            "--help" | "-h" => {
                print_usage(&args[0]);
                return;
            }
            _ => {
                eprintln!("Error: unknown option '{}'", args[i]);
                print_usage(&args[0]);
                return;
            }
        }
    }

    // Use default starting position if no FEN provided
    let board = if fen.is_empty() {
        println!("Using standard starting position");
        Board::new()
    } else {
        match Board::from_fen(&fen) {
            Ok(b) => {
                println!("Loaded position from FEN:");
                println!("{}", fen);
                b
            }
            Err(e) => {
                eprintln!("Error parsing FEN: {}", e);
                return;
            }
        }
    };

    // Display the board
    println!();
    board.print();

    // Show FEN representation
    println!("FEN: {}", board.to_fen());
    println!();

    // Check if the position is terminal
    if board.is_checkmate() {
        println!("Position is checkmate!");
        return;
    }

    if board.is_stalemate() {
        println!("Position is stalemate!");
        return;
    }

    // Create engine
    let mut engine = if let Some(book_path) = opening_book_path {
        match ChessEngine::with_opening_book(&book_path) {
            Ok(e) => {
                println!("Loaded opening book from: {}", book_path);
                e
            }
            Err(e) => {
                eprintln!("Warning: Could not load opening book: {}", e);
                println!("Continuing without opening book...");
                ChessEngine::new()
            }
        }
    } else {
        ChessEngine::new()
    };

    // Find best move
    println!("Searching for best move at depth {}...", depth);
    println!();

    let start_time = std::time::Instant::now();
    let best_move = engine.find_best_move(&board, depth);
    let elapsed = start_time.elapsed();

    // Display results
    match best_move {
        Some(chess_move) => {
            println!("Best move found: {}", chess_move.to_uci());
            println!("Search time: {:.2?}", elapsed);
            println!();

            // Get and display search metrics
            if let Some(metrics) = engine.get_last_search_metrics() {
                println!("Search Statistics:");
                println!("  Nodes explored: {}", metrics.nodes_explored);
                println!("  Max depth reached: {}", metrics.max_depth_reached);
                println!("  Beta cutoffs: {}", metrics.beta_cutoffs);
                println!("  Search time: {:.2?}", metrics.search_time);

                if metrics.nodes_explored > 0 {
                    let nps = (metrics.nodes_explored as f64 / elapsed.as_secs_f64()) as u64;
                    println!("  Nodes per second: {}", nps);
                }
            }

            // Apply the move and show resulting position
            println!();
            println!("Position after best move:");
            let mut new_board = board;
            new_board.apply_move(chess_move);
            new_board.print();
            println!("FEN: {}", new_board.to_fen());
        }
        None => {
            println!("No legal moves available (shouldn't happen if not checkmate/stalemate)");
        }
    }
}

fn print_usage(program_name: &str) {
    println!("Chess Position Tester");
    println!();
    println!("Usage: {} [OPTIONS]", program_name);
    println!();
    println!("Options:");
    println!("  -f, --fen <FEN>        FEN string for the position to test");
    println!("                         If not provided, uses standard starting position");
    println!("  -d, --depth <DEPTH>    Search depth (1-20, default: 5)");
    println!("  -b, --book <PATH>      Path to opening book file (optional)");
    println!("  -h, --help             Show this help message");
    println!();
    println!("Examples:");
    println!("  # Test starting position at depth 6");
    println!("  {} --depth 6", program_name);
    println!();
    println!("  # Test a specific position");
    println!(
        "  {} --fen \"r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3\"",
        program_name
    );
    println!();
    println!("  # Test with opening book");
    println!(
        "  {} --fen \"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1\" --book ./opening_book.bin",
        program_name
    );
    println!();
    println!("Common FEN positions:");
    println!("  Starting position:");
    println!("    rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    println!();
    println!("  Scholar's mate position (White to move, mate in 1):");
    println!("    r1bqk2r/pppp1ppp/2n2n2/2b1p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4");
    println!();
    println!("  Endgame: KQ vs K (White to move):");
    println!("    4k3/8/8/8/8/8/8/4K2Q w - - 0 1");
}
