use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use rusty_chess::board::{Board, ChessMove};
use rusty_chess::search::ChessEngine;

const BENCHMARK_POSITIONS: &[(&str, &str)] = &[
    // --- Mid-Game (High Branching Factor, Complex Evaluation) ---
    // A standard position after some development, good for testing general search efficiency.
    (
        "mid_game_1_ruy_lopez",
        "r4rk1/2pqbppp/p2p1n2/1p2p3/4P3/1P1P1N2/PP1B1PPP/R2Q1RK1 w - - 0 16",
    ),
    // A complex, closed Sicilian setup, testing positional evaluation features.
    (
        "mid_game_2_sicilian",
        "r1b2r2/pp1p2kp/2n2p1p/3np3/4P2B/2P2N2/PP3PPP/R4RK1 w - - 0 17",
    ),
    // A common test position from various engines (like the one used in the Stockfish test suite).
    (
        "mid_game_3_complex",
        "r1b1r1k1/ppq2ppp/2p1pn2/8/2PP4/1R3B2/P1P2PPP/3Q1RK1 w - - 0 17",
    ),
    // --- End-Game (Lower Node Count, Deep Search Required, Evaluation Dominates) ---
    // Rook and Pawn endgame, often requires searching to deep horizons.
    ("end_game_1_rp", "8/8/8/5k2/3R4/8/8/4K3 w - - 0 1"),
    // Queen vs. Rook/Pawn - complex material imbalance, good for evaluation stability.
    ("end_game_2_qrpp", "8/5q2/5k2/8/5P2/8/4K3/8 w - - 0 1"),
];

fn find_best_move(board: &Board, depth: u8) -> Option<ChessMove> {
    let mut engine: ChessEngine = ChessEngine::new();
    engine.find_best_move(board, depth)
}

fn criterion_benchmark(c: &mut Criterion) {
    let depths_to_test: [u8; 2] = [3, 4];

    for (name_tag, fen) in BENCHMARK_POSITIONS {
        let board = Board::from_fen(fen);

        let mut group = c.benchmark_group(format!("Position: {}", name_tag));

        group.sample_size(10);

        for &depth in depths_to_test.iter() {
            let bench_name = format!("Depth {}", depth);

            group.bench_with_input(bench_name, &board, move |b, board| {
                b.iter(|| {
                    black_box(find_best_move(board, depth));
                });
            });
        }

        group.finish();
    }
}

// 4. Configure the Criterion Groups
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
