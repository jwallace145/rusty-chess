use crate::board::{Board, ChessMove};
use crate::eval::Evaluator;

pub struct Minimax;

impl Minimax {
    pub fn find_best_move(board: &Board, depth: u8) -> Option<ChessMove> {
        let legal_moves = board.generate_legal_moves();

        if legal_moves.is_empty() {
            return None;
        }

        let mut best_move = legal_moves[0];
        let mut best_score = i32::MIN;

        for chess_move in legal_moves {
            let mut board_copy = *board;
            board_copy.apply_move(chess_move);

            let score = -Self::minimax(&board_copy, depth - 1);

            if score > best_score {
                best_score = score;
                best_move = chess_move;
            }
        }

        Some(best_move)
    }

    fn minimax(board: &Board, depth: u8) -> i32 {
        let legal_moves = board.generate_legal_moves();

        // Check for terminal positions first (checkmate or stalemate)
        if legal_moves.is_empty() {
            if board.is_checkmate() {
                // Losing position - return very negative score
                // Adjust score by depth to prefer faster checkmates
                return -100_000 - (depth as i32);
            } else {
                // Stalemate - return draw score
                return 0;
            }
        }

        if depth == 0 {
            return Evaluator::evaluate(board);
        }

        let mut best_score = i32::MIN;

        for chess_move in legal_moves {
            let mut board_copy = *board;
            board_copy.apply_move(chess_move);

            let score = -Self::minimax(&board_copy, depth - 1);

            if score > best_score {
                best_score = score;
            }
        }

        best_score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::chess_move::ChessMoveType;
    use crate::board::{Color, Piece};

    #[test]
    fn test_finds_checkmate_in_one() {
        // Set up a position where white can checkmate in one move
        // Position: White King on c7, Black King on a8, White Queen on c6
        // Checkmate: Qa6# or Qb7# (king has no escape squares)
        let mut board = Board::empty();
        board.squares[pos("c7")].0 = Some((Piece::King, Color::White));
        board.squares[pos("a8")].0 = Some((Piece::King, Color::Black));
        board.squares[pos("c6")].0 = Some((Piece::Queen, Color::White));
        board.white_king_pos = pos("c7");
        board.black_king_pos = pos("a8");
        board.side_to_move = Color::White;

        let best_move = Minimax::find_best_move(&board, 3);
        assert!(best_move.is_some());

        // Queen should move to a6 or b7 for checkmate
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
