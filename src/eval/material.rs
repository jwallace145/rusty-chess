use crate::{
    board::{Board, Color, Piece},
    eval::evaluator::BoardEvaluator,
};

pub struct MaterialEvaluator;

impl BoardEvaluator for MaterialEvaluator {
    fn evaluate(&self, board: &Board) -> i32 {
        let mut white_material: i32 = 0;
        let mut black_material: i32 = 0;

        for square in &board.squares {
            if let Some((piece, color)) = square.0 {
                let value: i32 = Self::piece_value(piece);

                match color {
                    Color::White => white_material += value,
                    Color::Black => black_material += value,
                }
            }
        }

        white_material - black_material
    }
}

impl MaterialEvaluator {
    fn piece_value(piece: Piece) -> i32 {
        match piece {
            Piece::Pawn => 100,
            Piece::Knight => 320,
            Piece::Bishop => 330,
            Piece::Rook => 500,
            Piece::Queen => 900,
            Piece::King => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::material::MaterialEvaluator;

    #[test]
    fn test_material_evaluator_piece_values() {
        let value: i32 = MaterialEvaluator::piece_value(Piece::Pawn);
        let expected_value: i32 = 100;
        assert_eq!(value, expected_value, "Pawn should have value 100");

        let value: i32 = MaterialEvaluator::piece_value(Piece::Knight);
        let expected_value: i32 = 320;
        assert_eq!(value, expected_value, "Knight should have value 320");

        let value: i32 = MaterialEvaluator::piece_value(Piece::Bishop);
        let expected_value: i32 = 330;
        assert_eq!(value, expected_value, "Bishop should have value 330");

        let value: i32 = MaterialEvaluator::piece_value(Piece::Rook);
        let expected_value: i32 = 500;
        assert_eq!(value, expected_value, "Rook should have value 500");

        let value: i32 = MaterialEvaluator::piece_value(Piece::Queen);
        let expected_value: i32 = 900;
        assert_eq!(value, expected_value, "Queen should have value 900");

        let value: i32 = MaterialEvaluator::piece_value(Piece::King);
        let expected_value: i32 = 0;
        assert_eq!(value, expected_value, "King should have value 0");
    }

    #[test]
    fn test_material_evaluator_initial_board_state() {
        let board: Board = Board::default();
        let value: i32 = MaterialEvaluator.evaluate(&board);
        let expected_value: i32 = 0;
        assert_eq!(
            value, expected_value,
            "Initial board state should have value 0"
        );
    }

    #[test]
    fn test_material_evaluator_white_advantage() {
        let mut board: Board = Board::default();

        // Remove Black pieces
        board.squares[63].0 = None; // Rook
        board.squares[62].0 = None; // Knight
        board.squares[55].0 = None; // Pawn

        // Remove White pieces
        board.squares[0].0 = None; // Rook

        let value: i32 = MaterialEvaluator.evaluate(&board);
        let expected_value: i32 = 420;

        assert_eq!(
            value, expected_value,
            "White should have material advantage of 420"
        );
        assert!(
            expected_value > 0,
            "White material advantage should be positive"
        );
    }

    #[test]
    fn test_material_evaluator_black_advantage() {
        let mut board: Board = Board::default();

        // Remove White pieces
        board.squares[1].0 = None; // Knight
        board.squares[6].0 = None; // Knight

        // Remove Black pieces
        board.squares[62].0 = None; // Knight
        board.squares[55].0 = None; // Pawn

        let value: i32 = MaterialEvaluator.evaluate(&board);
        let expected_value: i32 = -220;

        assert_eq!(
            value, expected_value,
            "Black should have material advantage of -220"
        );
        assert!(
            expected_value < 0,
            "Black material advantage should be negative"
        );
    }
}
