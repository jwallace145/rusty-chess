use crate::{
    board::{Board2, Color, Piece},
    eval::evaluator::BoardEvaluator,
};

// Bonus per controlled central square
const CENTER_BONUS: i32 = 5;

// Central square indices in 0..63 board array
const CENTRAL_SQUARES: [usize; 4] = [27, 28, 35, 36]; // d4=27, e4=28, d5=35, e5=36

/// Evaluates control of central squares (d4, e4, d5, e5).
/// Each controlled square adds a small bonus for the controlling side.
pub struct CentralControlEvaluator;

impl BoardEvaluator for CentralControlEvaluator {
    fn evaluate(&self, board: &Board2) -> i32 {
        let mut score = 0;

        for &sq in &CENTRAL_SQUARES {
            // Count White influence
            let white_control = Self::count_control(board, sq, Color::White);
            let black_control = Self::count_control(board, sq, Color::Black);

            score += CENTER_BONUS * (white_control as i32 - black_control as i32);
        }

        score
    }
}

impl CentralControlEvaluator {
    /// Returns the number of pseudo-legal moves that can attack the target square
    fn count_control(board: &Board2, target_sq: usize, color: Color) -> u8 {
        let mut control_count = 0;
        let target_mask = 1u64 << target_sq;

        // Iterate through all piece types for the given color
        for piece_idx in 0..6 {
            let piece = match piece_idx {
                0 => Piece::Pawn,
                1 => Piece::Knight,
                2 => Piece::Bishop,
                3 => Piece::Rook,
                4 => Piece::Queen,
                _ => Piece::King,
            };

            let mut piece_bb = board.pieces[color as usize][piece_idx];
            while piece_bb != 0 {
                let sq = piece_bb.trailing_zeros() as u8;

                // Get attack bitboard for this piece
                let attacks = board.attacks_from(piece, sq, color);

                // Check if this piece can attack the target square
                if attacks & target_mask != 0 {
                    control_count += 1;
                }

                piece_bb &= piece_bb - 1; // Clear the least significant bit
            }
        }

        control_count
    }
}
