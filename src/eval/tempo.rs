use crate::board::{Board, Color};

pub struct TempoEvaluator;

impl TempoEvaluator {
    pub fn evaluate(board: &Board) -> i32 {
        match board.side_to_move {
            Color::White => 10,
            Color::Black => -10,
        }
    }
}
