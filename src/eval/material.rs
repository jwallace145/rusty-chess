use crate::{
    board::{Board2, Color, Piece},
    eval::evaluator::BoardEvaluator,
};

pub struct MaterialEvaluator;

impl BoardEvaluator for MaterialEvaluator {
    fn evaluate(&self, board: &Board2) -> i32 {
        let mut white_material: i32 = 0;
        let mut black_material: i32 = 0;

        // Iterate through each piece type for each color
        for piece_idx in 0..6 {
            let piece = match piece_idx {
                0 => Piece::Pawn,
                1 => Piece::Knight,
                2 => Piece::Bishop,
                3 => Piece::Rook,
                4 => Piece::Queen,
                _ => Piece::King,
            };
            let value = Self::piece_value(piece);

            // Count white pieces
            let white_count = board.pieces[Color::White as usize][piece_idx].count_ones() as i32;
            white_material += white_count * value;

            // Count black pieces
            let black_count = board.pieces[Color::Black as usize][piece_idx].count_ones() as i32;
            black_material += black_count * value;
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
        let board = Board2::new_standard();
        let value: i32 = MaterialEvaluator.evaluate(&board);
        let expected_value: i32 = 0;
        assert_eq!(
            value, expected_value,
            "Initial board state should have value 0"
        );
    }

    #[test]
    fn test_material_evaluator_white_advantage() {
        let mut board = Board2::new_standard();

        // Remove Black pieces: Rook (63), Knight (62), Pawn (55)
        board.pieces[Color::Black as usize][Piece::Rook as usize] &= !(1u64 << 63); // Remove h8 rook
        board.pieces[Color::Black as usize][Piece::Knight as usize] &= !(1u64 << 62); // Remove g8 knight
        board.pieces[Color::Black as usize][Piece::Pawn as usize] &= !(1u64 << 55); // Remove h7 pawn

        // Remove White pieces: Rook (0)
        board.pieces[Color::White as usize][Piece::Rook as usize] &= !(1u64 << 0); // Remove a1 rook

        // Update occupancy
        board.occ[Color::White as usize] =
            board.pieces[Color::White as usize].iter().copied().sum();
        board.occ[Color::Black as usize] =
            board.pieces[Color::Black as usize].iter().copied().sum();
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

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
        let mut board = Board2::new_standard();

        // Remove White pieces: Knights (1, 6)
        board.pieces[Color::White as usize][Piece::Knight as usize] &= !(1u64 << 1); // Remove b1 knight
        board.pieces[Color::White as usize][Piece::Knight as usize] &= !(1u64 << 6); // Remove g1 knight

        // Remove Black pieces: Knight (62), Pawn (55)
        board.pieces[Color::Black as usize][Piece::Knight as usize] &= !(1u64 << 62); // Remove g8 knight
        board.pieces[Color::Black as usize][Piece::Pawn as usize] &= !(1u64 << 55); // Remove h7 pawn

        // Update occupancy
        board.occ[Color::White as usize] =
            board.pieces[Color::White as usize].iter().copied().sum();
        board.occ[Color::Black as usize] =
            board.pieces[Color::Black as usize].iter().copied().sum();
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

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
