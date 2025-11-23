use crate::{
    board::{Board2, Color, Piece},
    eval::evaluator::BoardEvaluator,
};

const OUTPOST_BONUS: i32 = 20; // base outpost value
const SUPPORTED_BONUS: i32 = 10; // extra if supported by pawn

pub struct KnightOutpostEvaluator;

impl BoardEvaluator for KnightOutpostEvaluator {
    fn evaluate(&self, board: &Board2) -> i32 {
        let mut score = 0;

        // Iterate through white knights
        let mut white_knights = board.pieces[Color::White as usize][Piece::Knight as usize];
        while white_knights != 0 {
            let sq = white_knights.trailing_zeros() as usize;
            score += Self::evaluate_knight(board, sq, Color::White);
            white_knights &= white_knights - 1;
        }

        // Iterate through black knights
        let mut black_knights = board.pieces[Color::Black as usize][Piece::Knight as usize];
        while black_knights != 0 {
            let sq = black_knights.trailing_zeros() as usize;
            score += Self::evaluate_knight(board, sq, Color::Black);
            black_knights &= black_knights - 1;
        }

        score
    }
}

impl KnightOutpostEvaluator {
    fn evaluate_knight(board: &Board2, sq: usize, color: Color) -> i32 {
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
    fn is_outpost(board: &Board2, sq: usize, color: Color) -> bool {
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
            let idx = ((r as usize) * 8 + (f as usize)) as u8;
            if let Some((piece_color, piece)) = board.piece_on(idx)
                && piece == Piece::Pawn
                && piece_color == enemy
            {
                return false; // enemy pawn can attack → NOT an outpost
            }
        }

        true
    }

    /// Checks if a friendly pawn supports this knight
    fn is_supported_by_pawn(board: &Board2, sq: usize, color: Color) -> bool {
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
            let idx = ((r as usize) * 8 + (f as usize)) as u8;
            if let Some((piece_color, piece)) = board.piece_on(idx)
                && piece == Piece::Pawn
                && piece_color == color
            {
                return true; // friendly pawn supports
            }
        }

        false
    }
}
