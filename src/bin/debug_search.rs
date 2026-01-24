//! Debug tool to diagnose why the engine selects different moves for identical positions
//! between live play and FEN analysis.
//!
//! Usage: cargo run --bin debug-search -- "<FEN>"

use rusty_chess::board::Board;
use rusty_chess::movegen::MoveGenerator;
use rusty_chess::search::{ChessEngine, compute_hash_board};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <FEN> [depth]", args[0]);
        eprintln!();
        eprintln!("Arguments:");
        eprintln!("  FEN    - Chess position in FEN notation");
        eprintln!("  depth  - Search depth (default: 4)");
        eprintln!();
        eprintln!("This tool helps debug why the engine might select different moves");
        eprintln!("for the same position in different contexts.");
        std::process::exit(1);
    }

    let fen = &args[1];

    println!("=== SEARCH DEBUGGING TOOL ===\n");

    // Step 1: Verify FEN parsing and hash consistency
    println!("--- Step 1: FEN Identity Verification ---");
    let board = Board::from_fen(fen);
    let regenerated_fen = board.to_fen();
    let reloaded_board = Board::from_fen(&regenerated_fen);

    println!("Input FEN:       {}", fen);
    println!("Regenerated FEN: {}", regenerated_fen);
    println!(
        "FEN match: {}",
        if fen.trim() == regenerated_fen {
            "YES"
        } else {
            "NO - POTENTIAL BUG!"
        }
    );
    println!();

    // Step 2: Hash consistency
    println!("--- Step 2: Zobrist Hash Verification ---");
    let hash1 = board.hash;
    let hash2 = compute_hash_board(&board);
    let hash3 = reloaded_board.hash;

    println!("Board hash (stored):     0x{:016x}", hash1);
    println!("Board hash (recomputed): 0x{:016x}", hash2);
    println!("Reloaded board hash:     0x{:016x}", hash3);
    println!(
        "Hash consistency: {}",
        if hash1 == hash2 && hash2 == hash3 {
            "OK"
        } else {
            "MISMATCH - BUG!"
        }
    );
    println!();

    // Step 3: Board state details
    println!("--- Step 3: Board State Details ---");
    println!("Side to move: {:?}", board.side_to_move);
    println!("Castling rights: {:?}", board.castling);
    println!(
        "En passant square: {}",
        if board.en_passant < 64 {
            let file = (board.en_passant % 8) as u8;
            let rank = (board.en_passant / 8) as u8;
            format!("{}{}", (b'a' + file) as char, (b'1' + rank) as char)
        } else {
            "-".to_string()
        }
    );
    println!("Halfmove clock: {}", board.halfmove_clock);
    println!();

    // Step 4: Generate legal moves
    println!("--- Step 4: Legal Moves ---");
    let mut moves = Vec::with_capacity(128);
    MoveGenerator::generate_legal_moves(&board, &mut moves);
    println!("Total legal moves: {}", moves.len());
    for (i, mv) in moves.iter().enumerate() {
        let from = square_name(mv.from);
        let to = square_name(mv.to);
        println!(
            "  {:2}. {}{} (capture: {}, type: {:?})",
            i + 1,
            from,
            to,
            mv.capture,
            mv.move_type
        );
    }
    println!();

    // Step 5: Search with FRESH TT (like find_best_move.rs)
    // Using FIXED DEPTH search (not time-based) for determinism
    // Use depth from command line or default to 4
    let depth: u8 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(4);
    println!(
        "--- Step 5: Search with FRESH TT (fixed depth {}) ---",
        depth
    );
    let mut engine_fresh = ChessEngine::new();
    engine_fresh.set_use_opening_book(false); // Disable book for fair comparison

    let best_move_fresh = engine_fresh.find_best_move(&board, depth);
    let (hits_fresh, misses_fresh) = engine_fresh.get_tt_stats();

    if let Some(mv) = best_move_fresh {
        println!(
            "Best move (fresh TT): {}{}",
            square_name(mv.from),
            square_name(mv.to)
        );
    } else {
        println!("Best move (fresh TT): None");
    }
    println!("TT stats: {} hits, {} misses", hits_fresh, misses_fresh);
    println!();

    // Step 6: Search AGAIN with the SAME engine (TT has entries now)
    println!(
        "--- Step 6: Search with WARM TT (same engine, fixed depth {}) ---",
        depth
    );
    let best_move_warm = engine_fresh.find_best_move(&board, depth);
    let (hits_warm, misses_warm) = engine_fresh.get_tt_stats();

    if let Some(mv) = best_move_warm {
        println!(
            "Best move (warm TT): {}{}",
            square_name(mv.from),
            square_name(mv.to)
        );
    } else {
        println!("Best move (warm TT): None");
    }
    println!(
        "TT stats: {} hits, {} misses (cumulative)",
        hits_warm, misses_warm
    );
    println!();

    // Step 7: Compare fresh vs warm
    println!("--- Step 7: Comparison ---");
    match (best_move_fresh, best_move_warm) {
        (Some(fresh), Some(warm)) => {
            if fresh == warm {
                println!("RESULT: Moves MATCH - TT state does not affect this position");
            } else {
                println!("RESULT: Moves DIFFER!");
                println!(
                    "  Fresh TT: {}{}",
                    square_name(fresh.from),
                    square_name(fresh.to)
                );
                println!(
                    "  Warm TT:  {}{}",
                    square_name(warm.from),
                    square_name(warm.to)
                );
                println!();
                println!("DIAGNOSIS: The transposition table affects move selection.");
                println!(
                    "This explains why live play (warm TT) differs from FEN analysis (fresh TT)."
                );
            }
        }
        _ => {
            println!("RESULT: Could not compare (no moves found)");
        }
    }
    println!();

    // Step 8: Test with TT cleared between searches
    println!(
        "--- Step 8: Search with TT cleared before search (fixed depth {}) ---",
        depth
    );
    engine_fresh.new_game(); // Clear TT
    let best_move_cleared = engine_fresh.find_best_move(&board, depth);

    if let Some(mv) = best_move_cleared {
        println!(
            "Best move (cleared TT): {}{}",
            square_name(mv.from),
            square_name(mv.to)
        );
    }

    if let (Some(fresh), Some(cleared)) = (best_move_fresh, best_move_cleared) {
        if fresh == cleared {
            println!("Fresh and cleared TT give SAME result - deterministic!");
        } else {
            println!("Fresh and cleared TT give DIFFERENT results - BUG!");
        }
    }
    println!();

    // Step 9: Test with a COMPLETELY NEW engine instance
    println!(
        "--- Step 9: Search with NEW engine instance (fixed depth {}) ---",
        depth
    );
    let mut engine_new = ChessEngine::new();
    engine_new.set_use_opening_book(false);
    let best_move_new = engine_new.find_best_move(&board, depth);

    if let Some(mv) = best_move_new {
        println!(
            "Best move (new engine): {}{}",
            square_name(mv.from),
            square_name(mv.to)
        );
    }

    if let (Some(fresh), Some(new_mv)) = (best_move_fresh, best_move_new) {
        if fresh == new_mv {
            println!("New engine gives SAME result as first fresh search - deterministic!");
        } else {
            println!("New engine gives DIFFERENT result - non-determinism exists!");
        }
    }
    println!();

    // Step 10: Test with London System book
    println!("--- Step 10: With Opening Book ---");
    let mut engine_book = ChessEngine::with_london_system();
    let best_move_book = engine_book.find_best_move(&board, depth);

    if let Some(mv) = best_move_book {
        println!(
            "Best move (with book): {}{}",
            square_name(mv.from),
            square_name(mv.to)
        );
    } else {
        println!("Best move (with book): None");
    }

    println!();
    println!("=== SUMMARY ===");
    println!("If moves differ between fresh TT and warm TT, the solution is to");
    println!("either clear the TT before each search, or accept that the engine");
    println!("may find different (but valid) moves when it has cached information.");
}

fn square_name(sq: usize) -> String {
    let file = (sq % 8) as u8;
    let rank = (sq / 8) as u8;
    format!("{}{}", (b'a' + file) as char, (b'1' + rank) as char)
}
