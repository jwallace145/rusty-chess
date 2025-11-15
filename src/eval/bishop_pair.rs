use crate::board::{Board, Color, Piece};

pub struct BishopPairEvaluator;

impl BishopPairEvaluator {
    const BISHOP_PAIR_BONUS: i32 = 30;

    pub fn evaluate(board: &Board) -> i32 {
        let white_bishops = board.count(Color::White, Piece::Bishop);
        let black_bishops = board.count(Color::Black, Piece::Bishop);

        let mut score = 0;

        if white_bishops >= 2 {
            score += Self::BISHOP_PAIR_BONUS;
        }
        if black_bishops >= 2 {
            score -= Self::BISHOP_PAIR_BONUS;
        }

        score
    }
}
