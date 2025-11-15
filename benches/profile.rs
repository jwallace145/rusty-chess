use rusty_chess::board::Board;
use rusty_chess::search::ChessEngine;
use std::time::Instant;

fn main() {
    println!("Starting chess engine benchmark...");

    // Move generation microbenchmark
    println!("\n=== Move Generation Microbenchmark ===");
    let board = Board::new();

    // Warm up
    for _ in 0..1000 {
        let _ = board.generate_legal_moves();
    }

    // Benchmark legal move generation
    let iterations = 100_000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = board.generate_legal_moves();
    }
    let legal_duration = start.elapsed();

    // Benchmark pseudo-legal move generation
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = board.generate_moves();
    }
    let pseudo_duration = start.elapsed();

    let legal_moves = board.generate_legal_moves();
    let pseudo_moves = board.generate_moves();

    println!("Iterations: {}", iterations);
    println!("Legal moves generated: {}", legal_moves.len());
    println!("Pseudo-legal moves generated: {}", pseudo_moves.len());
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
    println!("\nPseudo-legal move generation:");
    println!("  Total time: {:?}", pseudo_duration);
    println!(
        "  Per call: {:.2} µs",
        pseudo_duration.as_micros() as f64 / iterations as f64
    );
    println!(
        "  Calls/sec: {:.0}",
        iterations as f64 / pseudo_duration.as_secs_f64()
    );
    println!(
        "\nFiltering overhead: {:.2}x slower",
        legal_duration.as_secs_f64() / pseudo_duration.as_secs_f64()
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
