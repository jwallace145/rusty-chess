use crate::board::{Board, Color, MoveGenerator};

/// Evaluates control of central squares (d4, e4, d5, e5).
/// Each controlled square adds a small bonus for the controlling side.
pub struct CentralControlEvaluator;

impl CentralControlEvaluator {
    // Bonus per controlled central square
    const CENTER_BONUS: i32 = 5;

    // Central square indices in 0..63 board array
    const CENTRAL_SQUARES: [usize; 4] = [27, 28, 35, 36]; // d4=27, e4=28, d5=35, e5=36

    pub fn evaluate(board: &Board) -> i32 {
        let mut score = 0;

        for &sq in &Self::CENTRAL_SQUARES {
            // Count White influence
            let white_control = Self::count_control(board, sq, Color::White);
            let black_control = Self::count_control(board, sq, Color::Black);

            score += Self::CENTER_BONUS * (white_control as i32 - black_control as i32);
        }

        score
    }

    /// Returns the number of pseudo-legal moves that can attack the target square
    fn count_control(board: &Board, target_sq: usize, color: Color) -> u8 {
        let mut moves = Vec::with_capacity(32);
        let mut control_count = 0;

        for (index, square) in board.squares.iter().enumerate() {
            if let Some((piece, piece_color)) = square.0
                && piece_color == color
            {
                moves.clear();
                MoveGenerator::generate_piece_moves(board, index, piece, color, &mut moves);

                for mv in &moves {
                    if mv.to == target_sq {
                        control_count += 1;
                        break; // count each square at most once per color
                    }
                }
            }
        }

        control_count
    }
}
