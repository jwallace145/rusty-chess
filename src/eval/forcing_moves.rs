use crate::{
    board::{Board, ChessMove, Color, Piece},
    eval::evaluator::BoardEvaluator,
    movegen::MoveGenerator,
};

/// Evaluator that rewards positions with forcing moves available (checks, captures).
/// This helps the engine prefer tactical positions and value initiative.
pub struct ForcingMovesEvaluator;

impl ForcingMovesEvaluator {
    /// Material values for SEE calculations
    fn piece_value(piece: Piece) -> i32 {
        match piece {
            Piece::Pawn => 100,
            Piece::Knight => 320,
            Piece::Bishop => 330,
            Piece::Rook => 500,
            Piece::Queen => 900,
            Piece::King => 20000,
        }
    }

    /// Check if a move gives check
    fn move_gives_check(board: &Board, mv: &ChessMove) -> bool {
        let mut board_copy = *board;
        board_copy.make_move(*mv);
        board_copy.in_check(board_copy.side_to_move)
    }

    /// Evaluate if a capture is "safe" (SEE >= 0)
    fn is_safe_capture(board: &Board, mv: &ChessMove) -> bool {
        if !mv.capture {
            return false;
        }

        // Get the capturing piece value
        let from_sq = mv.from as u8;
        let to_sq = mv.to as u8;

        let (_, attacker_piece) = match board.piece_on(from_sq) {
            Some(p) => p,
            None => return false,
        };

        let (_, target_piece) = match board.piece_on(to_sq) {
            Some(p) => p,
            None => return false,
        };

        let attacker_value = Self::piece_value(attacker_piece);
        let target_value = Self::piece_value(target_piece);

        // Simple SEE: if capturing piece value <= target value, it's generally safe
        // For more complex positions, we'd need full SEE, but this is a good approximation
        if attacker_value <= target_value {
            return true;
        }

        // Check if the square is defended
        let defenders = board.attackers_to(to_sq, board.side_to_move.opponent());
        if defenders == 0 {
            return true; // Undefended piece
        }

        // If we're capturing with a lower value piece, check if target is defended by something cheaper
        // For now, be conservative: if defended and we're using higher value piece, not safe
        false
    }

    /// Evaluate forcing moves for the side to move only
    /// This evaluator only gives bonuses to the side whose turn it is,
    /// creating a natural tempo bonus for having forcing moves available.
    fn evaluate_side_to_move(board: &Board) -> i32 {
        let mut score = 0;
        let color = board.side_to_move;

        // Generate legal moves for the side to move
        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_legal_moves(board, &mut moves);

        let mut check_count = 0;
        let mut safe_capture_count = 0;
        let mut has_queen_check = false;

        for mv in &moves {
            // Skip castling - it's not a forcing move
            if mv.move_type == crate::board::chess_move::ChessMoveType::Castle {
                continue;
            }

            // Check for checks
            if Self::move_gives_check(board, mv) {
                check_count += 1;

                // Bonus for queen checks (often more dangerous)
                if let Some((_, piece)) = board.piece_on(mv.from as u8)
                    && piece == Piece::Queen
                {
                    has_queen_check = true;
                }
            }

            // Check for safe captures
            if Self::is_safe_capture(board, mv) {
                safe_capture_count += 1;
            }
        }

        // Scoring:
        // - First check: +60 cp
        // - Additional checks: +40 cp each (diminishing returns)
        // - Queen check available: +40 cp bonus
        // - Safe captures: +25 cp each (capped at 3)
        if check_count > 0 {
            score += 60; // First check
            score += (check_count - 1).min(3) * 40; // Additional checks (capped)
        }

        if has_queen_check {
            score += 40;
        }

        score += safe_capture_count.min(3) * 25; // Safe captures (capped at 3)

        // Return from perspective of the side to move
        match color {
            Color::White => score,
            Color::Black => -score,
        }
    }
}

impl BoardEvaluator for ForcingMovesEvaluator {
    fn evaluate(&self, board: &Board) -> i32 {
        // Only evaluate forcing moves for the side to move
        // This creates a consistent tempo-like bonus without asymmetry
        Self::evaluate_side_to_move(board)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forcing_moves_initial_position() {
        let board = Board::startpos();
        let evaluator = ForcingMovesEvaluator;
        let score = evaluator.evaluate(&board);
        // Initial position has no checks available, should be near zero
        assert!(
            score.abs() < 50,
            "Initial position should have small forcing moves score"
        );
    }

    #[test]
    fn test_forcing_moves_with_check() {
        // Position where White has Qg5+ available (from the regression test)
        let fen = "rn1q1rk1/ppp1pp1p/5n2/3p1b2/3P4/2N2N1P/PPP1PPP1/R1Q1KB1R w KQ - 3 1";
        let board = Board::from_fen(fen);
        let evaluator = ForcingMovesEvaluator;
        let score = evaluator.evaluate(&board);
        // White has Qg5+ available, should get significant bonus
        assert!(
            score > 50,
            "Position with check available should favor White significantly, got {}",
            score
        );
    }
}
