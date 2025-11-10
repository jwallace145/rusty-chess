use super::transposition_table::TranspositionTable;
use super::zobrist;
use crate::board::{Board, ChessMove, Piece};
use crate::eval::Evaluator;

pub struct Minimax;

impl Minimax {
    pub fn find_best_move(board: &Board, depth: u8) -> Option<ChessMove> {
        // Create transposition table for this search
        let mut tt = TranspositionTable::new();
        let result = Self::find_best_move_with_tt(board, depth, &mut tt);

        // Print transposition table statistics
        let (hits, misses) = tt.stats();
        if hits + misses > 0 {
            let hit_rate = (hits as f64 / (hits + misses) as f64) * 100.0;
            println!(
                "TT Stats - Hit rate: {:.1}% ({} hits, {} misses)",
                hit_rate, hits, misses
            );
        }

        result
    }

    fn find_best_move_with_tt(
        board: &Board,
        depth: u8,
        tt: &mut TranspositionTable,
    ) -> Option<ChessMove> {
        let legal_moves = board.generate_legal_moves();

        if legal_moves.is_empty() {
            return None;
        }

        // Order moves for better alpha-beta performance
        let ordered_moves = Self::order_moves(board, legal_moves);

        let mut best_move = ordered_moves[0];
        let mut best_score = -200_000;
        let mut alpha = -200_000;
        let beta = 200_000;

        for chess_move in ordered_moves {
            let mut board_copy = *board;
            board_copy.apply_move(chess_move);

            let score = -Self::alpha_beta(&board_copy, depth - 1, -beta, -alpha, tt);

            if score > best_score {
                best_score = score;
                best_move = chess_move;
            }

            // Update alpha for the root search
            if score > alpha {
                alpha = score;
            }
        }

        Some(best_move)
    }

    fn alpha_beta(
        board: &Board,
        depth: u8,
        mut alpha: i32,
        beta: i32,
        tt: &mut TranspositionTable,
    ) -> i32 {
        // Probe transposition table first
        let hash = zobrist::compute_hash(board);
        if let Some(entry) = tt.probe(hash, depth) {
            return entry.score;
        }

        let legal_moves = board.generate_legal_moves();

        // Check for terminal positions first (checkmate or stalemate)
        if legal_moves.is_empty() {
            let score = if board.is_checkmate() {
                // Losing position - return very negative score
                // Adjust score by depth to prefer faster checkmates
                -100_000 - (depth as i32)
            } else {
                // Stalemate - return draw score
                0
            };
            // Store terminal position in TT
            tt.store(hash, depth, score, None);
            return score;
        }

        if depth == 0 {
            let score = Evaluator::evaluate(board);
            tt.store(hash, depth, score, None);
            return score;
        }

        // Order moves for better pruning efficiency
        let ordered_moves = Self::order_moves(board, legal_moves);
        let mut best_move = None;

        for chess_move in ordered_moves {
            let mut board_copy = *board;
            board_copy.apply_move(chess_move);

            let score = -Self::alpha_beta(&board_copy, depth - 1, -beta, -alpha, tt);

            // Beta cutoff - this position is too good, opponent won't allow it
            if score >= beta {
                tt.store(hash, depth, beta, Some(chess_move));
                return beta;
            }

            // Update alpha if we found a better move
            if score > alpha {
                alpha = score;
                best_move = Some(chess_move);
            }
        }

        // Store the result in transposition table
        tt.store(hash, depth, alpha, best_move);

        alpha
    }

    /// Order moves to search promising moves first (improves alpha-beta pruning).
    /// Priority: captures (high-value victims first), then non-captures.
    fn order_moves(board: &Board, mut moves: Vec<ChessMove>) -> Vec<ChessMove> {
        moves.sort_by_key(|m| {
            if m.capture {
                // Prioritize capturing high-value pieces
                let victim_value = if let Some((piece, _)) = board.squares[m.to].0 {
                    Self::piece_value(piece)
                } else {
                    100 // En passant captures a pawn
                };
                -victim_value // Negative for descending order
            } else {
                0 // Non-captures have neutral priority
            }
        });
        moves
    }

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
    use crate::board::chess_move::ChessMoveType;
    use crate::board::{Color, Piece};

    #[test]
    fn test_finds_checkmate_in_one() {
        let mut board = Board::empty();
        board.squares[pos("c7")].0 = Some((Piece::King, Color::White));
        board.squares[pos("a8")].0 = Some((Piece::King, Color::Black));
        board.squares[pos("c6")].0 = Some((Piece::Queen, Color::White));
        board.white_king_pos = pos("c7");
        board.black_king_pos = pos("a8");
        board.side_to_move = Color::White;

        let best_move = Minimax::find_best_move(&board, 3);
        assert!(best_move.is_some());

        let chess_move = best_move.unwrap();
        let mut test_board = board;
        test_board.apply_move(chess_move);
        assert!(test_board.is_checkmate(), "Should deliver checkmate");
    }

    #[test]
    fn test_finds_checkmate_fools_game() {
        let mut board = Board::new();
        board.apply_move(ChessMove {
            from: pos("f2"),
            to: pos("f3"),
            capture: false,
            move_type: ChessMoveType::Normal,
        });
        board.apply_move(ChessMove {
            from: pos("e7"),
            to: pos("e5"),
            capture: false,
            move_type: ChessMoveType::Normal,
        });
        board.apply_move(ChessMove {
            from: pos("g2"),
            to: pos("g4"),
            capture: false,
            move_type: ChessMoveType::Normal,
        });

        let best_move = Minimax::find_best_move(&board, 3);
        assert!(best_move.is_some());

        let chess_move = best_move.unwrap();
        let mut test_board = board;
        test_board.apply_move(chess_move);
        assert!(test_board.is_checkmate(), "Should deliver checkmate")
    }

    fn pos(s: &str) -> usize {
        let bytes = s.as_bytes();
        let file = (bytes[0] - b'a') as usize;
        let rank = (bytes[1] - b'1') as usize;
        rank * 8 + file
    }
}
