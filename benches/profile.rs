use rusty_chess::board::Board2;
use rusty_chess::movegen::MoveGenerator;
use rusty_chess::search::ChessEngine;
use std::time::Instant;

fn main() {
    println!("Starting chess engine benchmark with Board2 and MoveGenerator...");

    // Move generation microbenchmark
    println!("\n=== Move Generation Microbenchmark (Board2) ===");
    let board = Board2::new_standard();
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

    // Engine search benchmark
    println!("\n=== ChessEngine Search Benchmark (Board2) ===");
    let mut engine = ChessEngine::new();

    // Run searches at various depths
    for depth in 4..=6 {
        println!("\n--- Depth {} ---", depth);
        let search_start = Instant::now();
        let result = engine.find_best_move(&board, depth);
        let search_time = search_start.elapsed();

        println!("Best move: {:?}", result);
        println!(
            "Total search time: {:.2} ms",
            search_time.as_secs_f64() * 1000.0
        );

        if let Some(metrics) = engine.get_last_search_metrics() {
            let nps = metrics.nodes_explored as f64 / metrics.search_time.as_secs_f64();
            println!("Performance: {:.0} nodes/second", nps);
        }
    }

    println!("\n=== Benchmark Summary ===");
    println!(
        "Move generation: {:.0} calls/sec",
        iterations as f64 / legal_duration.as_secs_f64()
    );
    if let Some(metrics) = engine.get_last_search_metrics() {
        let nps = metrics.nodes_explored as f64 / metrics.search_time.as_secs_f64();
        println!("Search performance: {:.0} nodes/sec (depth 7)", nps);
    }

    println!("\nBenchmark complete!");
}
