use crate::{
    board::{Board, Color},
    eval::{
        material::MaterialEvaluator, mobility::MobilityEvaluator,
        pawn_structure::PawnStructureEvaluator, position::PositionEvaluator,
        threatened::ThreatenedEvaluator,
    },
};

/// Evaluates chess board positions to guide the minimax search algorithm.
///
/// Converts board states into numerical scores by combining multiple components:
///
/// - **Material**: Assigns values to pieces (e.g., Queen=900, Rook=500, Pawn=100)
///   to encourage maintaining strong pieces
/// - **Positional**: Rewards pieces for occupying favorable squares using
///   piece-square tables (e.g., knights in the center, pawns advancing)
/// - **Pawn Structure**: Evaluates pawn formations, penalizing weaknesses
/// - **Mobility**: Rewards having more legal moves available
/// - **Threatened Pieces**: Penalizes hanging and poorly defended pieces
///
/// Returns positive scores when White is winning, negative when Black is winning.
pub struct Evaluator;

impl Evaluator {
    pub fn evaluate(board: &Board) -> i32 {
        let material: i32 = MaterialEvaluator::evaluate(board);
        let position: i32 = PositionEvaluator::evaluate(board);
        let pawn_structure: i32 = PawnStructureEvaluator::evaluate(board);
        let mobility: i32 = MobilityEvaluator::evaluate(board);
        let threatened: i32 = ThreatenedEvaluator::evaluate(board);

        let total: i32 = material + position + pawn_structure + mobility + threatened;

        // Return score from side to move's perspective
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
