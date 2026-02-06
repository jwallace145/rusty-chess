use crate::{
    board::{Board, Color, Piece},
    eval::evaluator::BoardEvaluator,
};

const BISHOP_PAIR_BONUS: i32 = 30;

pub struct BishopPairEvaluator;

impl BoardEvaluator for BishopPairEvaluator {
    fn evaluate(&self, board: &Board) -> i32 {
        let white_bishops: u32 = board.count_pieces(Color::White, Piece::Bishop);
        let black_bishops: u32 = board.count_pieces(Color::Black, Piece::Bishop);

        let mut score: i32 = 0;

        if white_bishops >= 2 {
            score += BISHOP_PAIR_BONUS;
        }
        if black_bishops >= 2 {
            score -= BISHOP_PAIR_BONUS;
        }

        score
    }
}
