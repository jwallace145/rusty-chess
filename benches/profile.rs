use rusty_chess::board::Board;
use rusty_chess::search::ChessEngine;

fn main() {
    println!("Starting chess engine benchmark...");

    let board = Board::new();
    let mut engine = ChessEngine::new();

    // Run multiple searches for better profiling data
    for depth in 4..=8 {
        println!("\n=== Depth {} ===", depth);
        let result = engine.find_best_move(&board, depth);
        println!("Best move: {:?}", result);
    }

    println!("\nBenchmark complete!");
}
