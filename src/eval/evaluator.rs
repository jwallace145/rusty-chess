use crate::{
    board::{Board, Color},
    eval::{
        bishop_pair::BishopPairEvaluator, central_control::CentralControlEvaluator,
        forcing_moves::ForcingMovesEvaluator, fork::ForkEvaluator,
        king_safety::KingSafetyEvaluator, knight_outpost::KnightOutpostEvaluator,
        line_pressure::LinePressureEvaluator, material::MaterialEvaluator,
        mobility::MobilityEvaluator, pawn_structure::PawnStructureEvaluator,
        position::PositionEvaluator, rook_file_evaluator::RookFileEvaluator, tempo::TempoEvaluator,
        threat::ThreatEvaluator,
    },
};

/// Trait for all evaluators
pub trait BoardEvaluator {
    /// Returns the evaluation score from White's perspective
    fn evaluate(&self, board: &Board) -> i32;
}

/// Evaluates a chess board position to guide the minimax search algorithm.
///
/// This evaluator converts board states into a numerical score by combining
/// multiple sub-evaluators. Each sub-evaluator produces a score in centipawns
/// that contributes to the total evaluation.
///
/// Sub-evaluators include:
/// - **Material**: Assigns values to pieces (e.g., Queen=900, Rook=500, Pawn=100)
///   to encourage retaining valuable pieces.
/// - **Positional**: Rewards pieces for occupying favorable squares via piece-square tables
///   (e.g., knights in the center, advanced pawns).
/// - **Pawn Structure**: Penalizes doubled, isolated, or backward pawns and rewards connected pawns.
/// - **Mobility**: Rewards the side with more available moves/squares attacked.
/// - **King Safety**: Considers castling, pawn shield, open files, and enemy pressure near the king.
/// - **Tempo**: Provides a small bonus for the side to move.
/// - **Bishop Pair**: Rewards having two bishops.
/// - **Knight Outpost**: Rewards knights on squares safe from enemy pawns and advanced into the enemy territory.
/// - **Threat**: Penalizes hanging pieces and pieces attacked by lower-value pieces (e.g., pawns).
///
/// Positive scores favor White; negative scores favor Black.
pub struct Evaluator {
    evaluators: Vec<(Box<dyn BoardEvaluator>, i32)>, // evaluator + weight
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Detailed breakdown of evaluation components (all from White's perspective)
#[derive(Debug, Clone)]
pub struct EvaluationBreakdown {
    pub material: i32,
    pub position: i32,
    pub pawn_structure: i32,
    pub mobility: i32,
    pub king_safety: i32,
    pub tempo: i32,
    pub bishop_pair: i32,
    pub knight_outpost: i32,
    pub rook_file: i32,
    pub central_control: i32,
    pub threat: i32,
    pub line_pressure: i32,
    pub fork: i32,
    pub forcing_moves: i32,
    pub total: i32,
}

impl std::fmt::Display for EvaluationBreakdown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  Material:        {:+5} cp", self.material)?;
        writeln!(f, "  Position:        {:+5} cp", self.position)?;
        writeln!(f, "  Pawn Structure:  {:+5} cp", self.pawn_structure)?;
        writeln!(f, "  Mobility:        {:+5} cp", self.mobility)?;
        writeln!(f, "  King Safety:     {:+5} cp", self.king_safety)?;
        writeln!(f, "  Tempo:           {:+5} cp", self.tempo)?;
        writeln!(f, "  Bishop Pair:     {:+5} cp", self.bishop_pair)?;
        writeln!(f, "  Knight Outpost:  {:+5} cp", self.knight_outpost)?;
        writeln!(f, "  Rook on File:    {:+5} cp", self.rook_file)?;
        writeln!(f, "  Central Control: {:+5} cp", self.central_control)?;
        writeln!(f, "  Threats:         {:+5} cp", self.threat)?;
        writeln!(f, "  Line Pressure:   {:+5} cp", self.line_pressure)?;
        writeln!(f, "  Fork Potential:  {:+5} cp", self.fork)?;
        writeln!(f, "  Forcing Moves:   {:+5} cp", self.forcing_moves)?;
        writeln!(f, "  ─────────────────────────")?;
        write!(f, "  TOTAL:           {:+5} cp", self.total)
    }
}

impl Evaluator {
    pub fn new() -> Self {
        let evaluators: Vec<(Box<dyn BoardEvaluator>, i32)> = vec![
            (Box::new(MaterialEvaluator), 1),
            (Box::new(PositionEvaluator), 1),
            (Box::new(PawnStructureEvaluator), 1),
            (Box::new(MobilityEvaluator), 1),
            (Box::new(KingSafetyEvaluator), 1),
            (Box::new(TempoEvaluator), 1),
            (Box::new(BishopPairEvaluator), 1),
            (Box::new(KnightOutpostEvaluator), 1),
            (Box::new(RookFileEvaluator), 1),
            (Box::new(CentralControlEvaluator), 1),
            (Box::new(ThreatEvaluator), 1),
            (Box::new(LinePressureEvaluator), 1),
            (Box::new(ForkEvaluator), 1),
            (Box::new(ForcingMovesEvaluator), 1),
        ];

        Self { evaluators }
    }

    pub fn evaluate(&self, board: &Board) -> i32 {
        // Sum weighted evaluator scores
        let mut total: i32 = 0;

        for (evaluator, weight) in &self.evaluators {
            let score: i32 = evaluator.evaluate(board);
            total += score * weight;
        }

        // Adjust for side to move
        match board.side_to_move {
            Color::White => total,
            Color::Black => -total,
        }
    }

    /// Returns a detailed breakdown of all evaluation components.
    /// All scores are from White's perspective (positive = White advantage).
    pub fn evaluate_detailed(&self, board: &Board) -> EvaluationBreakdown {
        let material = MaterialEvaluator.evaluate(board);
        let position = PositionEvaluator.evaluate(board);
        let pawn_structure = PawnStructureEvaluator.evaluate(board);
        let mobility = MobilityEvaluator.evaluate(board);
        let king_safety = KingSafetyEvaluator.evaluate(board);
        let tempo = TempoEvaluator.evaluate(board);
        let bishop_pair = BishopPairEvaluator.evaluate(board);
        let knight_outpost = KnightOutpostEvaluator.evaluate(board);
        let rook_file = RookFileEvaluator.evaluate(board);
        let central_control = CentralControlEvaluator.evaluate(board);
        let threat = ThreatEvaluator.evaluate(board);
        let line_pressure = LinePressureEvaluator.evaluate(board);
        let fork = ForkEvaluator.evaluate(board);
        let forcing_moves = ForcingMovesEvaluator.evaluate(board);

        let total = material
            + position
            + pawn_structure
            + mobility
            + king_safety
            + tempo
            + bishop_pair
            + knight_outpost
            + rook_file
            + central_control
            + threat
            + line_pressure
            + fork
            + forcing_moves;

        EvaluationBreakdown {
            material,
            position,
            pawn_structure,
            mobility,
            king_safety,
            tempo,
            bishop_pair,
            knight_outpost,
            rook_file,
            central_control,
            threat,
            line_pressure,
            fork,
            forcing_moves,
            total,
        }
    }
}
