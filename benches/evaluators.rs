use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use rusty_chess::board::Board;
use rusty_chess::eval::{
    BoardEvaluator, Evaluator, bishop_pair::BishopPairEvaluator,
    central_control::CentralControlEvaluator, fork::ForkEvaluator,
    king_safety::KingSafetyEvaluator, knight_outpost::KnightOutpostEvaluator,
    line_pressure::LinePressureEvaluator, material::MaterialEvaluator, mobility::MobilityEvaluator,
    pawn_structure::PawnStructureEvaluator, position::PositionEvaluator,
    rook_file_evaluator::RookFileEvaluator, tempo::TempoEvaluator, threat::ThreatEvaluator,
};

/// Test positions representing different game phases and complexity levels
const BENCHMARK_POSITIONS: &[(&str, &str)] = &[
    // Starting position - baseline
    (
        "starting",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    ),
    // Mid-game with high piece activity
    (
        "midgame_complex",
        "r1b2rk1/ppq2ppp/2p1pn2/8/2PP4/1R3B2/P1P2PPP/3Q1RK1 w - - 0 17",
    ),
    // Tactical position with many threats
    (
        "tactical",
        "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
    ),
    // Endgame with few pieces
    ("endgame_simple", "8/8/4k3/8/8/4K3/4P3/8 w - - 0 1"),
    // Complex endgame with rooks
    ("endgame_rooks", "8/5pk1/R7/8/8/8/5PP1/4R1K1 w - - 0 1"),
];

/// Macro to create a benchmark for a single evaluator
macro_rules! bench_evaluator {
    ($group:expr, $evaluator:expr, $name:expr, $board:expr) => {
        $group.bench_with_input(BenchmarkId::new($name, ""), $board, |b, board| {
            b.iter(|| black_box($evaluator.evaluate(board)))
        });
    };
}

/// Benchmark all sub-evaluators on a single position to compare their relative performance
fn benchmark_all_evaluators_single_position(c: &mut Criterion) {
    let board = Board::from_fen(BENCHMARK_POSITIONS[1].1); // midgame_complex

    let mut group = c.benchmark_group("SubEvaluator Comparison (midgame)");
    group.sample_size(1000);

    bench_evaluator!(group, MaterialEvaluator, "01_Material", &board);
    bench_evaluator!(group, PositionEvaluator, "02_Position", &board);
    bench_evaluator!(group, PawnStructureEvaluator, "03_PawnStructure", &board);
    bench_evaluator!(group, MobilityEvaluator, "04_Mobility", &board);
    bench_evaluator!(group, KingSafetyEvaluator, "05_KingSafety", &board);
    bench_evaluator!(group, TempoEvaluator, "06_Tempo", &board);
    bench_evaluator!(group, BishopPairEvaluator, "07_BishopPair", &board);
    bench_evaluator!(group, KnightOutpostEvaluator, "08_KnightOutpost", &board);
    bench_evaluator!(group, RookFileEvaluator, "09_RookFile", &board);
    bench_evaluator!(group, CentralControlEvaluator, "10_CentralControl", &board);
    bench_evaluator!(group, ThreatEvaluator, "11_Threat", &board);
    bench_evaluator!(group, LinePressureEvaluator, "12_LinePressure", &board);
    bench_evaluator!(group, ForkEvaluator, "13_Fork", &board);

    group.finish();
}

/// Benchmark each evaluator across all positions to see how position complexity affects them
fn benchmark_evaluators_across_positions(c: &mut Criterion) {
    // List of evaluators with their names
    let evaluators: Vec<(&str, Box<dyn BoardEvaluator>)> = vec![
        ("Material", Box::new(MaterialEvaluator)),
        ("Position", Box::new(PositionEvaluator)),
        ("PawnStructure", Box::new(PawnStructureEvaluator)),
        ("Mobility", Box::new(MobilityEvaluator)),
        ("KingSafety", Box::new(KingSafetyEvaluator)),
        ("Tempo", Box::new(TempoEvaluator)),
        ("BishopPair", Box::new(BishopPairEvaluator)),
        ("KnightOutpost", Box::new(KnightOutpostEvaluator)),
        ("RookFile", Box::new(RookFileEvaluator)),
        ("CentralControl", Box::new(CentralControlEvaluator)),
        ("Threat", Box::new(ThreatEvaluator)),
        ("LinePressure", Box::new(LinePressureEvaluator)),
        ("Fork", Box::new(ForkEvaluator)),
    ];

    for (eval_name, evaluator) in evaluators.iter() {
        let mut group = c.benchmark_group(format!("Evaluator: {}", eval_name));
        group.sample_size(500);

        for (pos_name, fen) in BENCHMARK_POSITIONS {
            let board = Board::from_fen(fen);

            group.bench_with_input(BenchmarkId::new(*pos_name, ""), &board, |b, board| {
                b.iter(|| black_box(evaluator.evaluate(board)))
            });
        }

        group.finish();
    }
}

/// Benchmark the complete Evaluator (all sub-evaluators combined)
fn benchmark_full_evaluator(c: &mut Criterion) {
    let evaluator = Evaluator::new();

    let mut group = c.benchmark_group("Full Evaluator");
    group.sample_size(500);

    for (pos_name, fen) in BENCHMARK_POSITIONS {
        let board = Board::from_fen(fen);

        group.bench_with_input(BenchmarkId::new(*pos_name, ""), &board, |b, board| {
            b.iter(|| black_box(evaluator.evaluate(board)))
        });
    }

    group.finish();
}

/// High-iteration benchmark to measure evaluator throughput
fn benchmark_evaluator_throughput(c: &mut Criterion) {
    let board = Board::from_fen(BENCHMARK_POSITIONS[1].1); // midgame_complex
    let evaluator = Evaluator::new();

    let mut group = c.benchmark_group("Throughput (1000 iterations)");
    group.sample_size(100);

    // Benchmark running the full evaluator 1000 times
    group.bench_function("Full Evaluator x1000", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                black_box(evaluator.evaluate(&board));
            }
        })
    });

    // Individual sub-evaluator throughput for comparison
    let sub_evaluators: Vec<(&str, Box<dyn BoardEvaluator>)> = vec![
        ("Material", Box::new(MaterialEvaluator)),
        ("Mobility", Box::new(MobilityEvaluator)),
        ("KingSafety", Box::new(KingSafetyEvaluator)),
        ("Threat", Box::new(ThreatEvaluator)),
    ];

    for (name, eval) in sub_evaluators.iter() {
        group.bench_function(format!("{} x1000", name), |b| {
            b.iter(|| {
                for _ in 0..1000 {
                    black_box(eval.evaluate(&board));
                }
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_all_evaluators_single_position,
    benchmark_evaluators_across_positions,
    benchmark_full_evaluator,
    benchmark_evaluator_throughput,
);
criterion_main!(benches);
