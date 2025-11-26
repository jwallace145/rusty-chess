use rusty_chess::board::{Board2, Color};
use rusty_chess::eval::{
    BoardEvaluator, bishop_pair::BishopPairEvaluator, central_control::CentralControlEvaluator,
    fork::ForkEvaluator, king_safety::KingSafetyEvaluator, knight_outpost::KnightOutpostEvaluator,
    line_pressure::LinePressureEvaluator, material::MaterialEvaluator, mobility::MobilityEvaluator,
    pawn_structure::PawnStructureEvaluator, position::PositionEvaluator,
    rook_file_evaluator::RookFileEvaluator, tempo::TempoEvaluator, threat::ThreatEvaluator,
};
use std::env;
use std::process;

fn print_usage(program_name: &str) {
    eprintln!("Usage: {} <fen> [options]", program_name);
    eprintln!();
    eprintln!(
        "Evaluates a chess position and returns a score similar to chess.com's evaluation bar."
    );
    eprintln!("Positive scores favor White, negative scores favor Black.");
    eprintln!();
    eprintln!("Arguments:");
    eprintln!("  <fen>                   Chess position in FEN notation (required)");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --quiet                 Only output the final score, no breakdown");
    eprintln!("  --json                  Output results in JSON format");
    eprintln!("  --help                  Show this help message");
    eprintln!();
    eprintln!("Examples:");
    eprintln!(
        "  {} \"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1\"",
        program_name
    );
    eprintln!(
        "  {} \"rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1\" --quiet",
        program_name
    );
    eprintln!(
        "  {} \"r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4\" --json",
        program_name
    );
}

#[derive(Default)]
struct Config {
    fen: String,
    quiet: bool,
    json: bool,
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
            "--quiet" => {
                config.quiet = true;
            }
            "--json" => {
                config.json = true;
            }
            _ => {
                return Err(format!("Unknown option: {}", args[i]));
            }
        }
        i += 1;
    }

    Ok(config)
}

/// Named evaluator for display purposes
struct NamedEvaluator {
    name: &'static str,
    evaluator: Box<dyn BoardEvaluator>,
}

impl NamedEvaluator {
    fn new(name: &'static str, evaluator: Box<dyn BoardEvaluator>) -> Self {
        Self { name, evaluator }
    }
}

/// Holds the result of a single evaluator
struct EvaluatorResult {
    name: &'static str,
    score: i32,
}

/// Holds all evaluation results
struct EvaluationResult {
    sub_evaluations: Vec<EvaluatorResult>,
    total_score: i32,
    side_to_move: Color,
}

impl EvaluationResult {
    /// Returns the score adjusted for perspective (always from White's POV when side_to_move is considered)
    fn final_score(&self) -> i32 {
        match self.side_to_move {
            Color::White => self.total_score,
            Color::Black => -self.total_score,
        }
    }

    /// Returns the score formatted like chess.com (+1.25, -0.50, etc.)
    fn formatted_score(&self) -> String {
        let score = self.final_score();
        let pawns = score as f64 / 100.0;
        if pawns >= 0.0 {
            format!("+{:.2}", pawns)
        } else {
            format!("{:.2}", pawns)
        }
    }
}

fn create_evaluators() -> Vec<NamedEvaluator> {
    vec![
        NamedEvaluator::new("Material", Box::new(MaterialEvaluator)),
        NamedEvaluator::new("Position", Box::new(PositionEvaluator)),
        NamedEvaluator::new("Pawn Structure", Box::new(PawnStructureEvaluator)),
        NamedEvaluator::new("Mobility", Box::new(MobilityEvaluator)),
        NamedEvaluator::new("King Safety", Box::new(KingSafetyEvaluator)),
        NamedEvaluator::new("Tempo", Box::new(TempoEvaluator)),
        NamedEvaluator::new("Bishop Pair", Box::new(BishopPairEvaluator)),
        NamedEvaluator::new("Knight Outpost", Box::new(KnightOutpostEvaluator)),
        NamedEvaluator::new("Rook Files", Box::new(RookFileEvaluator)),
        NamedEvaluator::new("Central Control", Box::new(CentralControlEvaluator)),
        NamedEvaluator::new("Threats", Box::new(ThreatEvaluator)),
        NamedEvaluator::new("Line Pressure", Box::new(LinePressureEvaluator)),
        NamedEvaluator::new("Forks", Box::new(ForkEvaluator)),
    ]
}

fn evaluate_board(board: &Board2) -> EvaluationResult {
    let evaluators = create_evaluators();
    let mut sub_evaluations = Vec::new();
    let mut total_score: i32 = 0;

    for named_eval in evaluators {
        let score = named_eval.evaluator.evaluate(board);
        total_score += score;
        sub_evaluations.push(EvaluatorResult {
            name: named_eval.name,
            score,
        });
    }

    EvaluationResult {
        sub_evaluations,
        total_score,
        side_to_move: board.side_to_move,
    }
}

fn print_evaluation_table(result: &EvaluationResult) {
    println!("+---------------------+-------------+------------------------+");
    println!("|                     Board Evaluation                       |");
    println!("+---------------------+-------------+------------------------+");
    println!("| Evaluator           | Centipawns  | Pawns                  |");
    println!("+---------------------+-------------+------------------------+");

    for eval in &result.sub_evaluations {
        let pawns = eval.score as f64 / 100.0;
        let pawns_str = if pawns >= 0.0 {
            format!("+{:.2}", pawns)
        } else {
            format!("{:.2}", pawns)
        };
        let bar = create_bar(eval.score);
        println!(
            "| {:19} | {:>11} | {:>6} {:15} |",
            eval.name, eval.score, pawns_str, bar
        );
    }

    println!("+---------------------+-------------+------------------------+");
    let total_pawns = result.total_score as f64 / 100.0;
    let total_pawns_str = if total_pawns >= 0.0 {
        format!("+{:.2}", total_pawns)
    } else {
        format!("{:.2}", total_pawns)
    };
    println!(
        "| {:19} | {:>11} | {:>6}                 |",
        "Raw Total", result.total_score, total_pawns_str
    );

    let final_score = result.final_score();
    let final_pawns = final_score as f64 / 100.0;
    let final_pawns_str = if final_pawns >= 0.0 {
        format!("+{:.2}", final_pawns)
    } else {
        format!("{:.2}", final_pawns)
    };
    println!(
        "| {:19} | {:>11} | {:>6}                 |",
        "Final (side adj.)", final_score, final_pawns_str
    );
    println!("+---------------------+-------------+------------------------+");
}

fn create_bar(score: i32) -> String {
    // Create a visual bar: max 7 chars each direction
    let max_chars = 7;
    let scale = 200; // 200 centipawns = full bar

    if score >= 0 {
        let filled = ((score.min(scale) as f64 / scale as f64) * max_chars as f64) as usize;
        format!("{}{}|", " ".repeat(max_chars), "#".repeat(filled))
    } else {
        let filled = (((-score).min(scale) as f64 / scale as f64) * max_chars as f64) as usize;
        let padding = max_chars - filled;
        format!("{}{}|", " ".repeat(padding), "#".repeat(filled))
    }
}

fn print_json(fen: &str, board: &Board2, result: &EvaluationResult) {
    let sub_evals: Vec<String> = result
        .sub_evaluations
        .iter()
        .map(|e| {
            format!(
                r#"    {{"name": "{}", "centipawns": {}, "pawns": {:.2}}}"#,
                e.name,
                e.score,
                e.score as f64 / 100.0
            )
        })
        .collect();

    println!("{{");
    println!(r#"  "fen": "{}","#, fen);
    println!(r#"  "side_to_move": "{:?}","#, board.side_to_move);
    println!(r#"  "sub_evaluations": ["#);
    println!("{}", sub_evals.join(",\n"));
    println!("  ],");
    println!(r#"  "raw_total_centipawns": {},"#, result.total_score);
    println!(r#"  "final_score_centipawns": {},"#, result.final_score());
    println!(
        r#"  "final_score_pawns": {:.2},"#,
        result.final_score() as f64 / 100.0
    );
    println!(r#"  "evaluation": "{}""#, result.formatted_score());
    println!("}}");
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
    let board = Board2::from_fen(&config.fen);

    // Evaluate the board
    let result = evaluate_board(&board);

    if config.json {
        print_json(&config.fen, &board, &result);
    } else if config.quiet {
        println!("{}", result.formatted_score());
    } else {
        println!();
        println!("Position:");
        board.print();
        println!();
        println!("Side to move: {:?}", board.side_to_move);
        println!();

        print_evaluation_table(&result);

        println!();
        println!(
            "Evaluation: {} (positive = White advantage, negative = Black advantage)",
            result.formatted_score()
        );
        println!();
    }
}
