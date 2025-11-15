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
pub struct Evaluator;

impl Evaluator {
    pub fn evaluate(board: &Board) -> i32 {
        // Collect sub-evaluator scores in a fixed-size array for clarity
        let scores: [i32; 10] = [
            MaterialEvaluator::evaluate(board),
            PositionEvaluator::evaluate(board),
            PawnStructureEvaluator::evaluate(board),
            MobilityEvaluator::evaluate(board),
            KingSafetyEvaluator::evaluate(board),
            TempoEvaluator::evaluate(board),
            BishopPairEvaluator::evaluate(board),
            KnightOutpostEvaluator::evaluate(board),
            RookFileEvaluator::evaluate(board),
            CentralControlEvaluator::evaluate(board),
        ];

        // Sum all sub-evaluator scores
        let total: i32 = scores.iter().sum();

        // Return from the side-to-move's perspective
        match board.side_to_move {
            Color::White => total,
            Color::Black => -total,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Piece;

    #[test]
    fn test_starting_position_equal() {
        let board = Board::new();
        let score = Evaluator::evaluate(&board);
        // Should be close to 0 (slight advantage for white having first move)
        assert!(
            score.abs() < 50,
            "Starting position should be roughly equal"
        );
    }

    #[test]
    fn test_center_knight_better_than_edge() {
        let mut board = Board::empty();
        board.squares[pos("e4")].0 = Some((Piece::Knight, Color::White));
        board.squares[pos("e8")].0 = Some((Piece::King, Color::Black));
        board.squares[pos("e1")].0 = Some((Piece::King, Color::White));
        board.white_king_pos = pos("e1");
        board.black_king_pos = pos("e8");
        board.side_to_move = Color::White;

        let center_score = Evaluator::evaluate(&board);

        let mut board2 = Board::empty();
        board2.squares[pos("a1")].0 = Some((Piece::Knight, Color::White));
        board2.squares[pos("e8")].0 = Some((Piece::King, Color::Black));
        board2.squares[pos("e1")].0 = Some((Piece::King, Color::White));
        board2.white_king_pos = pos("e1");
        board2.black_king_pos = pos("e8");
        board2.side_to_move = Color::White;

        let edge_score = Evaluator::evaluate(&board2);

        assert!(
            center_score > edge_score,
            "Knight in center should score higher"
        );
    }

    fn pos(s: &str) -> usize {
        let bytes = s.as_bytes();
        let file = (bytes[0] - b'a') as usize;
        let rank = (bytes[1] - b'1') as usize;
        rank * 8 + file
    }
}
