use crate::eval::Evaluator;

pub struct Minimax {
    evaluator: Evaluator,
}

impl Minimax {
    pub fn new() -> Self {
        Self {
            evaluator: Evaluator::new(),
        }
    }

    pub fn search(
        &self,
        board: &Board,
        depth: u8,
        alpha: i32,
        beta: i32,
        maximizing_player: bool,
    ) -> None {
        // Base Case: leaf node reached, evaluate current position
        if (depth == 0 || self.game_is_over(board)) {
            self.evaluator.evaluate(board);
        }

        if maximizing_player {
            max_evaluation = -10000000;

            MoveGenerator::generate_legal_moves(board)
        }
    }

    fn game_is_over(&self, board: &Board) -> bool {
        board.is_checkmate() || board.is_stalemate()
    }
}
