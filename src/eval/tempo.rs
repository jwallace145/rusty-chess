use crate::{
    board::{Board, Color},
    eval::evaluator::BoardEvaluator,
};

pub struct TempoEvaluator;

impl BoardEvaluator for TempoEvaluator {
    fn evaluate(&self, board: &Board) -> i32 {
        match board.side_to_move {
            Color::White => 10,
            Color::Black => -10,
        }
    }
}
