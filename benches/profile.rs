use rusty_chess::board::{Board, MoveGenerator};
use rusty_chess::search::ChessEngine;
use std::time::Instant;

fn main() {
    println!("Starting chess engine benchmark...");

    // Move generation microbenchmark
    println!("\n=== Move Generation Microbenchmark ===");
    let board = Board::new();
    let mut moves_buffer = Vec::with_capacity(128);

    // Warm up
    for _ in 0..1000 {
        MoveGenerator::generate_legal_moves(&board, &mut moves_buffer);
    }

    // Benchmark legal move generation
    let iterations = 100_000;
    let start = Instant::now();
    for _ in 0..iterations {
        MoveGenerator::generate_legal_moves(&board, &mut moves_buffer);
    }
    let legal_duration = start.elapsed();

    // Note: pseudo-legal move generation benchmark removed as it's not public
    // Only legal moves are relevant for the engine

    MoveGenerator::generate_legal_moves(&board, &mut moves_buffer);
    let legal_moves_count = moves_buffer.len();

    println!("Iterations: {}", iterations);
    println!("Legal moves generated: {}", legal_moves_count);
    println!("\nLegal move generation:");
    println!("  Total time: {:?}", legal_duration);
    println!(
        "  Per call: {:.2} µs",
        legal_duration.as_micros() as f64 / iterations as f64
    );
    println!(
        "  Calls/sec: {:.0}",
        iterations as f64 / legal_duration.as_secs_f64()
    );
    println!("=======================================\n");

    let mut engine = ChessEngine::new();

    // Run multiple searches for better profiling data
    for depth in 4..=7 {
        println!("\n=== Depth {} ===", depth);
        let result = engine.find_best_move(&board, depth);
        println!("Best move: {:?}", result);
    }

    println!("\nBenchmark complete!");
}
