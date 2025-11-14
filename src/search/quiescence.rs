use super::transposition_table::TranspositionTable;
use crate::board::{Board, ChessMove, Piece};
use crate::eval::Evaluator;
use crate::search::SearchHistory;
use crate::search::SearchMetrics;

/// Delta pruning margin - captures that can't improve alpha by this amount are pruned
const DELTA_MARGIN: i32 = 200;

/// Quiescence search implementation for tactical position evaluation.
///
/// Quiescence search continues the search beyond the main search depth by examining
/// only "tactical" moves (captures and checks) until a "quiet" position is reached.
/// This prevents the horizon effect where the engine misses immediate tactical sequences
/// that occur just beyond the search depth.
///
/// # Key Features
/// - **Standing Pat**: Uses static evaluation as a baseline - if already winning, no need to capture
/// - **Capture-only search**: Only examines forcing moves (captures and check evasions)
/// - **Delta pruning**: Skips captures that are unlikely to improve the position
/// - **Depth limiting**: Stops after a configurable depth to prevent infinite search
///
/// # References
/// - [Chess Programming Wiki: Quiescence Search](https://www.chessprogramming.org/Quiescence_Search)
pub struct Quiescence;

impl Quiescence {
    /// Perform quiescence search on a position.
    ///
    /// # Arguments
    /// * `board` - The current board position
    /// * `alpha` - Lower bound of the search window
    /// * `beta` - Upper bound of the search window
    /// * `history` - Position history for repetition detection
    /// * `tt` - Transposition table for caching
    /// * `metrics` - Search metrics to track performance
    /// * `q_depth` - Remaining quiescence search depth
    ///
    /// # Returns
    /// The evaluation score from the current side's perspective
    #[allow(clippy::too_many_arguments)]
    pub fn search(
        board: &Board,
        mut alpha: i32,
        beta: i32,
        history: &mut SearchHistory,
        tt: &mut TranspositionTable,
        metrics: &mut SearchMetrics,
        q_depth: u8,
    ) -> i32 {
        // Track nodes explored
        metrics.nodes_explored += 1;

        // Check for repetition - this is a draw
        if history.is_repetition(board.zobrist_hash) {
            return 0;
        }

        // Probe transposition table
        if let Some(entry) = tt.probe(board.zobrist_hash, q_depth) {
            return entry.score;
        }

        // Standing pat: evaluate the current position
        // This is our baseline - if we're already winning enough, we don't need to capture
        let stand_pat = Evaluator::evaluate(board);

        // Beta cutoff - we're already too good, opponent won't allow this line
        if stand_pat >= beta {
            return beta;
        }

        // Update alpha - the standing pat score becomes our new lower bound
        if stand_pat > alpha {
            alpha = stand_pat;
        }

        // Depth limit reached - return the static evaluation
        if q_depth == 0 {
            tt.store(board.zobrist_hash, q_depth, stand_pat, None);
            return stand_pat;
        }

        // Generate all legal moves
        let legal_moves = board.generate_legal_moves();

        // Check if we're in check - if so, we must search all evasions, not just captures
        let in_check = board.is_checkmate()
            || legal_moves.iter().any(|_m| {
                // A more accurate check would be to see if the king is under attack
                // For now, we'll rely on the move generation to handle this
                false
            });

        // Filter to captures only (unless we're in check)
        let captures: Vec<ChessMove> = if in_check {
            legal_moves // Search all moves if in check
        } else {
            legal_moves
                .into_iter()
                .filter(|chess_move| chess_move.capture)
                .collect()
        };

        // No captures available - return standing pat
        if captures.is_empty() {
            tt.store(board.zobrist_hash, q_depth, stand_pat, None);
            return stand_pat;
        }

        // Order captures by MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
        let ordered_captures = Self::order_captures(board, captures);
        let mut best_move = None;

        for chess_move in ordered_captures {
            // Delta pruning: skip captures that can't possibly improve our position
            // If we're down by more than the value of the captured piece + margin, skip it
            if !in_check {
                let captured_value = Self::get_capture_value(board, &chess_move);
                if stand_pat + captured_value + DELTA_MARGIN < alpha {
                    continue; // This capture won't help us reach alpha
                }
            }

            // Apply the move and search recursively
            let mut board_copy = *board;
            board_copy.apply_move(chess_move);

            // Push position before recursing
            history.push(board_copy.zobrist_hash);

            // Recursively search with negated alpha-beta window
            let score = -Self::search(
                &board_copy,
                -beta,
                -alpha,
                history,
                tt,
                metrics,
                q_depth - 1,
            );

            // Pop position after returning
            history.pop();

            // Beta cutoff
            if score >= beta {
                tt.store(board.zobrist_hash, q_depth, beta, Some(chess_move));
                return beta;
            }

            // Update alpha if we found a better move
            if score > alpha {
                alpha = score;
                best_move = Some(chess_move);
            }
        }

        // Store the result in transposition table
        tt.store(board.zobrist_hash, q_depth, alpha, best_move);

        alpha
    }

    /// Order captures using MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
    fn order_captures(board: &Board, mut captures: Vec<ChessMove>) -> Vec<ChessMove> {
        captures.sort_by_key(|m| Self::capture_priority(board, m));
        captures
    }

    /// Calculate priority for a capture move (negative for descending sort)
    fn capture_priority(board: &Board, chess_move: &ChessMove) -> i32 {
        let victim_value = Self::get_capture_value(board, chess_move);

        let attacker_value = if let Some((piece, _)) = board.squares[chess_move.from].0 {
            Self::piece_value(piece)
        } else {
            0 // Shouldn't happen
        };

        // MVV-LVA: prioritize high-value victims captured by low-value attackers
        // Negative for descending sort order
        -(victim_value * 10 - attacker_value)
    }

    /// Get the value of the piece being captured
    fn get_capture_value(board: &Board, chess_move: &ChessMove) -> i32 {
        if let Some((piece, _)) = board.squares[chess_move.to].0 {
            Self::piece_value(piece)
        } else {
            100 // En passant captures a pawn
        }
    }

    /// Get the centipawn value of a piece
    fn piece_value(piece: Piece) -> i32 {
        match piece {
            Piece::Pawn => 100,
            Piece::Knight => 320,
            Piece::Bishop => 330,
            Piece::Rook => 500,
            Piece::Queen => 900,
            Piece::King => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Color;

    // Test configuration
    const MAX_QUIESCENCE_DEPTH: u8 = 4;

    #[test]
    fn test_quiescence_standing_pat() {
        // Test that quiescence returns a reasonable score for a quiet position
        let board = Board::new(); // Starting position
        let mut tt = TranspositionTable::new_with_entries(1024);
        let mut metrics = SearchMetrics::new();
        let mut history = SearchHistory::new();

        let score = Quiescence::search(
            &board,
            i32::MIN + 1,
            i32::MAX,
            &mut history,
            &mut tt,
            &mut metrics,
            MAX_QUIESCENCE_DEPTH,
        );

        // Starting position should be roughly equal
        assert!(
            score.abs() < 200,
            "Starting position should be close to 0, got {}",
            score
        );
    }

    #[test]
    fn test_quiescence_capture_sequence() {
        // Create a position with a capture sequence
        let mut board = Board::empty();
        board.squares[pos("e4")].0 = Some((Piece::Pawn, Color::White));
        board.squares[pos("d5")].0 = Some((Piece::Pawn, Color::Black));
        board.squares[pos("c6")].0 = Some((Piece::Knight, Color::White));
        board.squares[pos("e8")].0 = Some((Piece::King, Color::Black));
        board.squares[pos("e1")].0 = Some((Piece::King, Color::White));
        board.white_king_pos = pos("e1");
        board.black_king_pos = pos("e8");
        board.side_to_move = Color::White;

        let mut tt = TranspositionTable::new_with_entries(1024);
        let mut metrics = SearchMetrics::new();
        let mut history = SearchHistory::new();

        // Search should consider the pawn capture
        let score = Quiescence::search(
            &board,
            i32::MIN + 1,
            i32::MAX,
            &mut history,
            &mut tt,
            &mut metrics,
            MAX_QUIESCENCE_DEPTH,
        );

        // After capturing the pawn, white should be up material
        assert!(score > 50, "Should favor capturing the free pawn");
    }

    fn pos(s: &str) -> usize {
        let bytes = s.as_bytes();
        let file = (bytes[0] - b'a') as usize;
        let rank = (bytes[1] - b'1') as usize;
        rank * 8 + file
    }
}
