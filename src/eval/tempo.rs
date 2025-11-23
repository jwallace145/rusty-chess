use crate::{
    board::{Board2, Color},
    eval::evaluator::BoardEvaluator,
};

pub struct TempoEvaluator;

impl BoardEvaluator for TempoEvaluator {
    fn evaluate(&self, board: &Board2) -> i32 {
        match board.side_to_move {
            Color::White => 10,
            Color::Black => -10,
        }
    }
}
