use crate::{
    board::{Board, Color, Piece},
    eval::evaluator::BoardEvaluator,
};

const OUTPOST_BONUS: i32 = 20; // base outpost value
const SUPPORTED_BONUS: i32 = 10; // extra if supported by pawn

pub struct KnightOutpostEvaluator;

impl BoardEvaluator for KnightOutpostEvaluator {
    fn evaluate(&self, board: &Board) -> i32 {
        let mut score = 0;

        for (sq, square) in board.squares.iter().enumerate() {
            if let Some((piece, color)) = square.0
                && piece == Piece::Knight
            {
                score += Self::evaluate_knight(board, sq, color);
            }
        }

        score
    }
}

impl KnightOutpostEvaluator {
    fn evaluate_knight(board: &Board, sq: usize, color: Color) -> i32 {
        if !Self::is_outpost(board, sq, color) {
            return 0;
        }

        let mut score = OUTPOST_BONUS;

        // Add support bonus if friendly pawn defends the square
        if Self::is_supported_by_pawn(board, sq, color) {
            score += SUPPORTED_BONUS;
        }

        // White gets positive, Black gets negative
        if color == Color::White { score } else { -score }
    }

    /// Outpost = cannot be attacked by enemy pawns
    fn is_outpost(board: &Board, sq: usize, color: Color) -> bool {
        let file = sq % 8;
        let rank = sq / 8;

        // Enemy pawn attack directions
        let enemy = color.opponent();

        // Squares from which enemy pawn could attack this knight
        let attack_squares = match enemy {
            Color::White => [
                (file as i32 - 1, rank as i32 + 1),
                (file as i32 + 1, rank as i32 + 1),
            ],
            Color::Black => [
                (file as i32 - 1, rank as i32 - 1),
                (file as i32 + 1, rank as i32 - 1),
            ],
        };

        for (f, r) in attack_squares {
            if !(0..8).contains(&f) || !(0..8).contains(&r) {
                continue;
            }
            let idx = (r as usize) * 8 + (f as usize);
            if let Some((piece, piece_color)) = board.squares[idx].0
                && piece == Piece::Pawn
                && piece_color == enemy
            {
                return false; // enemy pawn can attack → NOT an outpost
            }
        }

        true
    }

    /// Checks if a friendly pawn supports this knight
    fn is_supported_by_pawn(board: &Board, sq: usize, color: Color) -> bool {
        let file = sq % 8;
        let rank = sq / 8;

        // Friendly pawn attack directions
        let attack_squares = match color {
            Color::White => [
                (file as i32 - 1, rank as i32 - 1),
                (file as i32 + 1, rank as i32 - 1),
            ],
            Color::Black => [
                (file as i32 - 1, rank as i32 + 1),
                (file as i32 + 1, rank as i32 + 1),
            ],
        };

        for (f, r) in attack_squares {
            if !(0..8).contains(&f) || !(0..8).contains(&r) {
                continue;
            }
            let idx = (r as usize) * 8 + (f as usize);
            if let Some((piece, piece_color)) = board.squares[idx].0
                && piece == Piece::Pawn
                && piece_color == color
            {
                return true; // friendly pawn supports
            }
        }

        false
    }
}
