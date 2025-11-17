use crate::{
    board::{Board, Color},
    eval::{
        bishop_pair::BishopPairEvaluator, central_control::CentralControlEvaluator,
        king_safety::KingSafetyEvaluator, knight_outpost::KnightOutpostEvaluator,
        material::MaterialEvaluator, mobility::MobilityEvaluator,
        pawn_structure::PawnStructureEvaluator, position::PositionEvaluator,
        rook_file_evaluator::RookFileEvaluator, tempo::TempoEvaluator,
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
}
