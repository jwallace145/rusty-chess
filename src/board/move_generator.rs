use crate::board::{Board, ChessMove, Color, Piece, chess_move::ChessMoveType};

pub struct MoveGenerator;

impl MoveGenerator {
    pub fn generate_legal_moves(board: &Board, moves: &mut Vec<ChessMove>) {
        // Insert pseudo moves into given moves buffer
        moves.clear();
        Self::generate_pseudo_moves(board, moves);

        // Create attacked squares vector to check if king is in check after
        // pseudo move
        let mut attacked_squares: Vec<ChessMove> = Vec::with_capacity(64);

        // Iterate over generate pseudo moves in the moves buffer and test
        // legality of move (i.e. does not put team king in check)
        let mut i: usize = 0;
        while i < moves.len() {
            // Get pseudo move from vector of moves in O(1)
            let pseudo_move: ChessMove = moves[i];

            // Apply pseudo move to board copy to test legality
            //
            // TODO: Board copy operation is probably expensive...
            let mut board_copy: Board = *board;
            board_copy.apply_move(pseudo_move);

            // After applying pseudo move, test if pseudo move
            // is legal by verifying it does not put team king
            // in check
            let leaves_king_in_check: bool =
                Self::is_in_check(&board_copy, board.side_to_move, &mut attacked_squares);

            if leaves_king_in_check {
                // Move is illegal - remove it
                moves.swap_remove(i);
            } else {
                // Move is legal - keep it
                i += 1;
            }
        }
    }

    pub fn is_checkmate(board: &Board) -> bool {
        let mut legal_moves: Vec<ChessMove> = Vec::with_capacity(128);
        MoveGenerator::generate_legal_moves(board, &mut legal_moves);
        let mut attacked_squares: Vec<ChessMove> = Vec::with_capacity(64);
        legal_moves.is_empty()
            && Self::is_in_check(board, board.side_to_move, &mut attacked_squares)
    }

    pub fn is_stalemate(board: &Board) -> bool {
        let mut legal_moves: Vec<ChessMove> = Vec::with_capacity(128);
        MoveGenerator::generate_legal_moves(board, &mut legal_moves);
        let mut attacked_squares: Vec<ChessMove> = Vec::with_capacity(64);
        legal_moves.is_empty()
            && !Self::is_in_check(board, board.side_to_move, &mut attacked_squares)
    }

    pub fn is_in_check(board: &Board, color: Color, attacked_squares: &mut Vec<ChessMove>) -> bool {
        attacked_squares.clear();
        let king_square: usize = board.king_pos(color);
        Self::is_square_attacked(board, king_square, color.opponent(), attacked_squares)
    }

    pub fn is_square_attacked(
        board: &Board,
        square: usize,
        attacker_color: Color,
        attacked_squares: &mut Vec<ChessMove>,
    ) -> bool {
        // Iterate over attackers and generate moves to test attack against given square
        for (index, sq) in board.squares.iter().enumerate() {
            if let Some((piece, color)) = sq.0
                && color == attacker_color
            {
                attacked_squares.clear();

                // Generate moves for attacker and add to attacked squares buffer
                Self::generate_piece_moves(board, index, piece, color, attacked_squares);

                // If the given square is attacked by the current attackter, return true
                if attacked_squares.iter().any(|m| m.to == square) {
                    return true;
                }
            }
        }

        false
    }

    fn generate_pseudo_moves(board: &Board, moves: &mut Vec<ChessMove>) {
        for (i, square) in board.squares.iter().enumerate() {
            if let Some((piece, color)) = square.0
                && color == board.side_to_move
            {
                Self::generate_piece_moves(board, i, piece, color, moves)
            }
        }
    }

    pub fn generate_piece_moves(
        board: &Board,
        index: usize,
        piece: Piece,
        color: Color,
        moves: &mut Vec<ChessMove>,
    ) {
        match piece {
            Piece::Pawn => Self::generate_pawn_moves(board, index, color, moves),
            Piece::Rook => Self::generate_rook_moves(board, index, color, moves),
            Piece::Knight => Self::generate_knight_moves(board, index, color, moves),
            Piece::Bishop => Self::generate_bishop_moves(board, index, color, moves),
            Piece::Queen => Self::generate_queen_moves(board, index, color, moves),
            Piece::King => Self::generate_king_moves(board, index, color, moves),
        }
    }

    fn generate_pawn_moves(board: &Board, index: usize, color: Color, moves: &mut Vec<ChessMove>) {
        let rank = index / 8;
        let file = index % 8;

        let (forward, start_rank, promotion_rank): (isize, usize, usize) = match color {
            Color::White => (1, 1, 7),
            Color::Black => (-1, 6, 0),
        };

        // move forward one square
        let new_rank = (rank as isize + forward) as usize;
        if new_rank < 8 {
            let forward_idx = new_rank * 8 + file;
            if board.squares[forward_idx].0.is_none() {
                let move_type = if new_rank == promotion_rank {
                    ChessMoveType::Promotion(Piece::Queen)
                } else {
                    ChessMoveType::Normal
                };

                moves.push(ChessMove {
                    from: index,
                    to: forward_idx,
                    capture: false,
                    move_type,
                });

                if rank == start_rank {
                    let double_rank = (rank as isize + 2 * forward) as usize;
                    let double_idx = double_rank * 8 + file;
                    if board.squares[double_idx].0.is_none() {
                        moves.push(ChessMove {
                            from: index,
                            to: double_idx,
                            capture: false,
                            move_type: ChessMoveType::Normal,
                        });
                    }
                }
            }
        }

        // Diagonal captures
        for df in [-1, 1] {
            let new_file = file as isize + df;
            if (0..8).contains(&new_file) && new_rank < 8 {
                let capture_idx = new_rank * 8 + new_file as usize;
                if let Some((_, target_color)) = board.squares[capture_idx].0 {
                    if target_color != color {
                        let move_type = if new_rank == promotion_rank {
                            ChessMoveType::Promotion(Piece::Queen)
                        } else {
                            ChessMoveType::Normal
                        };

                        moves.push(ChessMove {
                            from: index,
                            to: capture_idx,
                            capture: true,
                            move_type,
                        });
                    }
                } else if Some(capture_idx) == board.en_passant_target {
                    // en passant capture
                    moves.push(ChessMove {
                        from: index,
                        to: capture_idx,
                        capture: true,
                        move_type: ChessMoveType::EnPassant,
                    });
                }
            }
        }
    }

    fn generate_rook_moves(board: &Board, index: usize, color: Color, moves: &mut Vec<ChessMove>) {
        Self::generate_sliding_moves(
            board,
            index,
            color,
            &[(1, 0), (-1, 0), (0, 1), (0, -1)],
            moves,
        );
    }

    fn generate_knight_moves(
        board: &Board,
        index: usize,
        color: Color,
        moves: &mut Vec<ChessMove>,
    ) {
        let rank = (index / 8) as isize;
        let file = (index % 8) as isize;
        let deltas = [
            (2, 1),
            (1, 2),
            (-1, 2),
            (-2, 1),
            (-2, -1),
            (-1, -2),
            (1, -2),
            (2, -1),
        ];

        for (dr, df) in deltas {
            let new_rank = rank + dr;
            let new_file = file + df;
            if (0..8).contains(&new_rank) && (0..8).contains(&new_file) {
                let to = (new_rank * 8 + new_file) as usize;
                match board.squares[to].0 {
                    None => moves.push(ChessMove {
                        from: index,
                        to,
                        capture: false,
                        move_type: ChessMoveType::Normal,
                    }),
                    Some((_, target_color)) if target_color != color => moves.push(ChessMove {
                        from: index,
                        to,
                        capture: true,
                        move_type: ChessMoveType::Normal,
                    }),
                    _ => {}
                }
            }
        }
    }

    fn generate_bishop_moves(
        board: &Board,
        index: usize,
        color: Color,
        moves: &mut Vec<ChessMove>,
    ) {
        Self::generate_sliding_moves(
            board,
            index,
            color,
            &[(1, 1), (1, -1), (-1, 1), (-1, -1)],
            moves,
        );
    }

    fn generate_queen_moves(board: &Board, index: usize, color: Color, moves: &mut Vec<ChessMove>) {
        Self::generate_sliding_moves(
            board,
            index,
            color,
            &[
                (1, 0),
                (-1, 0),
                (0, 1),
                (0, -1),
                (1, 1),
                (1, -1),
                (-1, 1),
                (-1, -1),
            ],
            moves,
        );
    }

    fn generate_king_moves(board: &Board, index: usize, color: Color, moves: &mut Vec<ChessMove>) {
        let rank = (index / 8) as isize;
        let file = (index % 8) as isize;
        let deltas = [
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ];

        for (dr, df) in deltas {
            let new_rank = rank + dr;
            let new_file = file + df;
            if (0..8).contains(&new_rank) && (0..8).contains(&new_file) {
                let to = (new_rank * 8 + new_file) as usize;
                match board.squares[to].0 {
                    None => moves.push(ChessMove {
                        from: index,
                        to,
                        capture: false,
                        move_type: ChessMoveType::Normal,
                    }),
                    Some((_, target_color)) if target_color != color => {
                        moves.push(ChessMove {
                            from: index,
                            to,
                            capture: true,
                            move_type: ChessMoveType::Normal,
                        });
                    }
                    _ => {}
                }
            }
        }

        // Castling logic
        match color {
            Color::White => {
                // Kingside castling (O-O): e1 to g1
                if !board.white_king_moved
                    && !board.white_kingside_rook_moved
                    && index == 4 // e1
                    && board.squares[5].0.is_none() // f1 empty
                    && board.squares[6].0.is_none() // g1 empty
                    && board.squares[7].0 == Some((Piece::Rook, Color::White))
                // h1 has white rook
                {
                    moves.push(ChessMove {
                        from: index,
                        to: 6, // g1
                        capture: false,
                        move_type: ChessMoveType::Castle,
                    });
                }

                // Queenside castling (O-O-O): e1 to c1
                if !board.white_king_moved
                    && !board.white_queenside_rook_moved
                    && index == 4 // e1
                    && board.squares[3].0.is_none() // d1 empty
                    && board.squares[2].0.is_none() // c1 empty
                    && board.squares[1].0.is_none() // b1 empty
                    && board.squares[0].0 == Some((Piece::Rook, Color::White))
                // a1 has white rook
                {
                    moves.push(ChessMove {
                        from: index,
                        to: 2, // c1
                        capture: false,
                        move_type: ChessMoveType::Castle,
                    });
                }
            }
            Color::Black => {
                // Kingside castling (O-O): e8 to g8
                if !board.black_king_moved
                    && !board.black_kingside_rook_moved
                    && index == 60 // e8
                    && board.squares[61].0.is_none() // f8 empty
                    && board.squares[62].0.is_none() // g8 empty
                    && board.squares[63].0 == Some((Piece::Rook, Color::Black))
                // h8 has black rook
                {
                    moves.push(ChessMove {
                        from: index,
                        to: 62, // g8
                        capture: false,
                        move_type: ChessMoveType::Castle,
                    });
                }

                // Queenside castling (O-O-O): e8 to c8
                if !board.black_king_moved
                    && !board.black_queenside_rook_moved
                    && index == 60 // e8
                    && board.squares[59].0.is_none() // d8 empty
                    && board.squares[58].0.is_none() // c8 empty
                    && board.squares[57].0.is_none() // b8 empty
                    && board.squares[56].0 == Some((Piece::Rook, Color::Black))
                // a8 has black rook
                {
                    moves.push(ChessMove {
                        from: index,
                        to: 58, // c8
                        capture: false,
                        move_type: ChessMoveType::Castle,
                    });
                }
            }
        }
    }

    fn generate_sliding_moves(
        board: &Board,
        index: usize,
        color: Color,
        directions: &[(isize, isize)],
        moves: &mut Vec<ChessMove>,
    ) {
        let rank = (index / 8) as isize;
        let file = (index % 8) as isize;

        for (dr, df) in directions {
            let mut r = rank + dr;
            let mut f = file + df;
            while (0..8).contains(&r) && (0..8).contains(&f) {
                let to = (r * 8 + f) as usize;
                match board.squares[to].0 {
                    None => moves.push(ChessMove {
                        from: index,
                        to,
                        capture: false,
                        move_type: ChessMoveType::Normal,
                    }),
                    Some((_, target_color)) if target_color != color => {
                        moves.push(ChessMove {
                            from: index,
                            to,
                            capture: true,
                            move_type: ChessMoveType::Normal,
                        });
                        break;
                    }
                    _ => break,
                }
                r += dr;
                f += df;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Board, Color, Piece};

    fn pos(s: &str) -> usize {
        let bytes = s.as_bytes();
        let file = (bytes[0] - b'a') as usize;
        let rank = (bytes[1] - b'1') as usize;
        rank * 8 + file
    }

    #[test]
    fn test_generate_pawn_moves() {
        let mut board = Board::empty();

        let piece = Piece::Pawn;
        let color = Color::White;
        let from = pos("e2");
        let expected = vec![
            ChessMove {
                from: pos("e2"),
                to: pos("e3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("e2"),
                to: pos("e4"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
        ];

        board.squares[from].0 = Some((piece, color));
        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_pawn_moves(&board, from, color, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_pawn_moves_single_move_after_start_rank() {
        let mut board = Board::empty();

        let piece = Piece::Pawn;
        let color = Color::White;
        let from = pos("e3");
        let expected = vec![ChessMove {
            from: pos("e3"),
            to: pos("e4"),
            capture: false,
            move_type: ChessMoveType::Normal,
        }];

        board.squares[from].0 = Some((piece, color));
        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_pawn_moves(&board, from, color, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_pawn_moves_blocked_double_move() {
        // Test that a pawn on its starting rank cannot double-move if destination is blocked
        let mut board = Board::empty();

        // Place white pawn on h2
        board.squares[pos("h2")].0 = Some((Piece::Pawn, Color::White));
        // Place a piece on h4 (blocking the double move destination)
        board.squares[pos("h4")].0 = Some((Piece::Queen, Color::Black));

        // The pawn should only be able to move to h3
        let expected = vec![ChessMove {
            from: pos("h2"),
            to: pos("h3"),
            capture: false,
            move_type: ChessMoveType::Normal,
        }];

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_pawn_moves(&board, pos("h2"), Color::White, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_pawn_moves_capture_opponent() {
        let mut board = Board::empty();

        let piece1 = Piece::Pawn;
        let color1 = Color::White;
        let from1 = pos("d4");
        let piece2 = Piece::Pawn;
        let color2 = Color::Black;
        let from2 = pos("e5");
        let piece3 = Piece::Pawn;
        let color3 = Color::Black;
        let from3 = pos("c5");

        board.squares[from1].0 = Some((piece1, color1));
        board.squares[from2].0 = Some((piece2, color2));
        board.squares[from3].0 = Some((piece3, color3));

        let expected = vec![
            ChessMove {
                from: pos("d4"),
                to: pos("d5"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e5"),
                capture: true,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c5"),
                capture: true,
                move_type: ChessMoveType::Normal,
            },
        ];

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_pawn_moves(&board, from1, color1, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_pawn_moves_en_passant() {
        // Test white pawn capturing black pawn via en passant
        let mut board = Board::empty();

        // Place white pawn on e5
        board.squares[pos("e5")].0 = Some((Piece::Pawn, Color::White));
        // Place black pawn on d5
        board.squares[pos("d5")].0 = Some((Piece::Pawn, Color::Black));
        // Set en passant target to d6 (as if black pawn just moved from d7 to d5)
        board.en_passant_target = Some(pos("d6"));
        board.side_to_move = Color::White;

        let expected = vec![
            // Normal forward move
            ChessMove {
                from: pos("e5"),
                to: pos("e6"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            // En passant capture to d6
            ChessMove {
                from: pos("e5"),
                to: pos("d6"),
                capture: true,
                move_type: ChessMoveType::EnPassant,
            },
        ];

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_pawn_moves(&board, pos("e5"), Color::White, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }

        // Test black pawn capturing white pawn via en passant
        let mut board = Board::empty();

        // Place black pawn on d4
        board.squares[pos("d4")].0 = Some((Piece::Pawn, Color::Black));
        // Place white pawn on e4
        board.squares[pos("e4")].0 = Some((Piece::Pawn, Color::White));
        // Set en passant target to e3 (as if white pawn just moved from e2 to e4)
        board.en_passant_target = Some(pos("e3"));
        board.side_to_move = Color::Black;

        let expected = vec![
            // Normal forward move
            ChessMove {
                from: pos("d4"),
                to: pos("d3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            // En passant capture to e3
            ChessMove {
                from: pos("d4"),
                to: pos("e3"),
                capture: true,
                move_type: ChessMoveType::EnPassant,
            },
        ];

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_pawn_moves(&board, pos("d4"), Color::Black, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_pawn_moves_promotion() {
        // Test white pawn promotion by moving forward
        let mut board = Board::empty();

        // Place white pawn on e7 (one square away from promotion)
        board.squares[pos("e7")].0 = Some((Piece::Pawn, Color::White));
        board.side_to_move = Color::White;

        let expected = vec![ChessMove {
            from: pos("e7"),
            to: pos("e8"),
            capture: false,
            move_type: ChessMoveType::Promotion(Piece::Queen),
        }];

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_pawn_moves(&board, pos("e7"), Color::White, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }

        // Test white pawn promotion by capturing
        let mut board = Board::empty();

        // Place white pawn on d7
        board.squares[pos("d7")].0 = Some((Piece::Pawn, Color::White));
        // Place black pieces on e8 and c8 for diagonal captures
        board.squares[pos("e8")].0 = Some((Piece::Rook, Color::Black));
        board.squares[pos("c8")].0 = Some((Piece::Knight, Color::Black));
        board.side_to_move = Color::White;

        let expected = vec![
            // Forward promotion
            ChessMove {
                from: pos("d7"),
                to: pos("d8"),
                capture: false,
                move_type: ChessMoveType::Promotion(Piece::Queen),
            },
            // Capture promotion to e8
            ChessMove {
                from: pos("d7"),
                to: pos("e8"),
                capture: true,
                move_type: ChessMoveType::Promotion(Piece::Queen),
            },
            // Capture promotion to c8
            ChessMove {
                from: pos("d7"),
                to: pos("c8"),
                capture: true,
                move_type: ChessMoveType::Promotion(Piece::Queen),
            },
        ];

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_pawn_moves(&board, pos("d7"), Color::White, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }

        // Test black pawn promotion by moving forward
        let mut board = Board::empty();

        // Place black pawn on e2 (one square away from promotion)
        board.squares[pos("e2")].0 = Some((Piece::Pawn, Color::Black));
        board.side_to_move = Color::Black;

        let expected = vec![ChessMove {
            from: pos("e2"),
            to: pos("e1"),
            capture: false,
            move_type: ChessMoveType::Promotion(Piece::Queen),
        }];

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_pawn_moves(&board, pos("e2"), Color::Black, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }

        // Test black pawn promotion by capturing
        let mut board = Board::empty();

        // Place black pawn on d2
        board.squares[pos("d2")].0 = Some((Piece::Pawn, Color::Black));
        // Place white pieces on e1 and c1 for diagonal captures
        board.squares[pos("e1")].0 = Some((Piece::Rook, Color::White));
        board.squares[pos("c1")].0 = Some((Piece::Knight, Color::White));
        board.side_to_move = Color::Black;

        let expected = vec![
            // Forward promotion
            ChessMove {
                from: pos("d2"),
                to: pos("d1"),
                capture: false,
                move_type: ChessMoveType::Promotion(Piece::Queen),
            },
            // Capture promotion to e1
            ChessMove {
                from: pos("d2"),
                to: pos("e1"),
                capture: true,
                move_type: ChessMoveType::Promotion(Piece::Queen),
            },
            // Capture promotion to c1
            ChessMove {
                from: pos("d2"),
                to: pos("c1"),
                capture: true,
                move_type: ChessMoveType::Promotion(Piece::Queen),
            },
        ];

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_pawn_moves(&board, pos("d2"), Color::Black, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_rook_moves() {
        let mut board = Board::empty();

        let piece = Piece::Rook;
        let color = Color::White;
        let from = pos("a1");

        let mut expected = vec![];

        for square in ["a2", "a3", "a4", "a5", "a6", "a7", "a8"] {
            expected.push(ChessMove {
                from: pos("a1"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        for square in ["b1", "c1", "d1", "e1", "f1", "g1", "h1"] {
            expected.push(ChessMove {
                from: pos("a1"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        board.squares[from].0 = Some((piece, color));
        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_rook_moves(&board, from, color, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_rook_moves_blocked_by_teammate() {
        let mut board = Board::empty();

        let color = Color::White;
        let piece1 = Piece::Rook;
        let from1 = pos("a1");
        let piece2 = Piece::Pawn;
        let from2 = pos("b1");

        let mut expected = vec![];
        for square in ["a2", "a3", "a4", "a5", "a6", "a7", "a8"] {
            expected.push(ChessMove {
                from: pos("a1"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        board.squares[from1].0 = Some((piece1, color));
        board.squares[from2].0 = Some((piece2, color));

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_rook_moves(&board, from1, color, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_rook_moves_capture_opponent() {
        let mut board = Board::empty();

        let piece1 = Piece::Rook;
        let color1 = Color::White;
        let from1 = pos("a1");
        let piece2 = Piece::Rook;
        let color2 = Color::Black;
        let from2 = pos("h1");

        let mut expected = vec![];

        for square in ["a2", "a3", "a4", "a5", "a6", "a7", "a8"] {
            expected.push(ChessMove {
                from: pos("a1"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        for square in ["b1", "c1", "d1", "e1", "f1", "g1"] {
            expected.push(ChessMove {
                from: pos("a1"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        expected.push(ChessMove {
            from: pos("a1"),
            to: pos("h1"),
            capture: true,
            move_type: ChessMoveType::Normal,
        });

        board.squares[from1].0 = Some((piece1, color1));
        board.squares[from2].0 = Some((piece2, color2));

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_rook_moves(&board, from1, color1, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_knight_moves() {
        let mut board = Board::empty();

        let color = Color::White;
        let piece1 = Piece::Knight;
        let from1 = pos("a2");
        let piece2 = Piece::Pawn;
        let from2 = pos("b2");

        board.squares[from1].0 = Some((piece1, color));
        board.squares[from2].0 = Some((piece2, color));

        let expected = vec![
            ChessMove {
                from: pos("a2"),
                to: pos("b4"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("a2"),
                to: pos("c3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("a2"),
                to: pos("c1"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
        ];

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_knight_moves(&board, from1, color, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_knight_moves_blocked_by_teammate() {
        let mut board = Board::empty();

        let color = Color::White;
        let piece1 = Piece::Knight;
        let from1 = pos("a2");
        let piece2 = Piece::Pawn;
        let from2 = pos("c1");

        board.squares[from1].0 = Some((piece1, color));
        board.squares[from2].0 = Some((piece2, color));

        let expected = vec![
            ChessMove {
                from: pos("a2"),
                to: pos("b4"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("a2"),
                to: pos("c3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
        ];

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_knight_moves(&board, from1, color, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_knight_moves_capture_opponent() {
        let mut board = Board::empty();

        let piece1 = Piece::Knight;
        let color1 = Color::White;
        let from1 = pos("a2");
        let piece2 = Piece::Pawn;
        let color2 = Color::Black;
        let from2 = pos("c3");

        board.squares[from1].0 = Some((piece1, color1));
        board.squares[from2].0 = Some((piece2, color2));

        let expected = vec![
            ChessMove {
                from: pos("a2"),
                to: pos("b4"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("a2"),
                to: pos("c3"),
                capture: true,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("a2"),
                to: pos("c1"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
        ];

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_knight_moves(&board, from1, color1, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_bishop_moves() {
        let mut board = Board::empty();

        let piece = Piece::Bishop;
        let color = Color::White;
        let from = pos("d4");

        let mut expected = vec![];

        // diagonal up-right
        for square in ["e5", "f6", "g7", "h8"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal up-left
        for square in ["c5", "b6", "a7"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal down-right
        for square in ["e3", "f2", "g1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal down-left
        for square in ["c3", "b2", "a1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        board.squares[from].0 = Some((piece, color));
        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_bishop_moves(&board, from, color, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_bishop_moves_blocked_by_teammate() {
        let mut board = Board::empty();

        let color = Color::White;
        let piece1 = Piece::Bishop;
        let from1 = pos("d4");
        let piece2 = Piece::Pawn;
        let from2 = pos("f6");

        let mut expected = vec![];

        // diagonal up-right (blocked at f6)
        expected.push(ChessMove {
            from: pos("d4"),
            to: pos("e5"),
            capture: false,
            move_type: ChessMoveType::Normal,
        });

        // diagonal up-left
        for square in ["c5", "b6", "a7"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal down-right
        for square in ["e3", "f2", "g1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal down-left
        for square in ["c3", "b2", "a1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        board.squares[from1].0 = Some((piece1, color));
        board.squares[from2].0 = Some((piece2, color));

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_bishop_moves(&board, from1, color, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_bishop_moves_capture_opponent() {
        let mut board = Board::empty();

        let piece1 = Piece::Bishop;
        let color1 = Color::White;
        let from1 = pos("d4");
        let piece2 = Piece::Pawn;
        let color2 = Color::Black;
        let from2 = pos("f6");

        let mut expected = vec![];

        // diagonal up-right (capture at f6)
        expected.push(ChessMove {
            from: pos("d4"),
            to: pos("e5"),
            capture: false,
            move_type: ChessMoveType::Normal,
        });
        expected.push(ChessMove {
            from: pos("d4"),
            to: pos("f6"),
            capture: true,
            move_type: ChessMoveType::Normal,
        });

        // diagonal up-left
        for square in ["c5", "b6", "a7"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal down-right
        for square in ["e3", "f2", "g1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal down-left
        for square in ["c3", "b2", "a1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        board.squares[from1].0 = Some((piece1, color1));
        board.squares[from2].0 = Some((piece2, color2));

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_bishop_moves(&board, from1, color1, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_queen_moves() {
        let mut board = Board::empty();

        let piece = Piece::Queen;
        let color = Color::White;
        let from = pos("d4");

        let mut expected = vec![];

        // horizontal right
        for square in ["e4", "f4", "g4", "h4"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // horizontal left
        for square in ["c4", "b4", "a4"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // vertical up
        for square in ["d5", "d6", "d7", "d8"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // vertical down
        for square in ["d3", "d2", "d1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal up-right
        for square in ["e5", "f6", "g7", "h8"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal up-left
        for square in ["c5", "b6", "a7"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal down-right
        for square in ["e3", "f2", "g1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal down-left
        for square in ["c3", "b2", "a1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        board.squares[from].0 = Some((piece, color));
        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_queen_moves(&board, from, color, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_queen_moves_blocked_by_teammate() {
        let mut board = Board::empty();

        let color = Color::White;
        let piece1 = Piece::Queen;
        let from1 = pos("d4");
        let piece2 = Piece::Pawn;
        let from2 = pos("d6");

        let mut expected = vec![];

        // horizontal right
        for square in ["e4", "f4", "g4", "h4"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // horizontal left
        for square in ["c4", "b4", "a4"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // vertical up (blocked at d6)
        expected.push(ChessMove {
            from: pos("d4"),
            to: pos("d5"),
            capture: false,
            move_type: ChessMoveType::Normal,
        });

        // vertical down
        for square in ["d3", "d2", "d1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal up-right
        for square in ["e5", "f6", "g7", "h8"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal up-left
        for square in ["c5", "b6", "a7"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal down-right
        for square in ["e3", "f2", "g1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal down-left
        for square in ["c3", "b2", "a1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        board.squares[from1].0 = Some((piece1, color));
        board.squares[from2].0 = Some((piece2, color));

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_queen_moves(&board, from1, color, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_queen_moves_capture_opponent() {
        let mut board = Board::empty();

        let piece1 = Piece::Queen;
        let color1 = Color::White;
        let from1 = pos("d4");
        let piece2 = Piece::Pawn;
        let color2 = Color::Black;
        let from2 = pos("d6");

        let mut expected = vec![];

        // horizontal right
        for square in ["e4", "f4", "g4", "h4"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // horizontal left
        for square in ["c4", "b4", "a4"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // vertical up (capture at d6)
        expected.push(ChessMove {
            from: pos("d4"),
            to: pos("d5"),
            capture: false,
            move_type: ChessMoveType::Normal,
        });
        expected.push(ChessMove {
            from: pos("d4"),
            to: pos("d6"),
            capture: true,
            move_type: ChessMoveType::Normal,
        });

        // vertical down
        for square in ["d3", "d2", "d1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal up-right
        for square in ["e5", "f6", "g7", "h8"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal up-left
        for square in ["c5", "b6", "a7"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal down-right
        for square in ["e3", "f2", "g1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        // diagonal down-left
        for square in ["c3", "b2", "a1"] {
            expected.push(ChessMove {
                from: pos("d4"),
                to: pos(square),
                capture: false,
                move_type: ChessMoveType::Normal,
            });
        }

        board.squares[from1].0 = Some((piece1, color1));
        board.squares[from2].0 = Some((piece2, color2));

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_queen_moves(&board, from1, color1, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_king_moves() {
        let mut board = Board::empty();

        let piece = Piece::King;
        let color = Color::White;
        let from = pos("d4");

        let expected = vec![
            ChessMove {
                from: pos("d4"),
                to: pos("d5"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("d3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e4"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c4"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e5"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c5"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
        ];

        board.squares[from].0 = Some((piece, color));
        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_king_moves(&board, from, color, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_king_moves_blocked_by_teammate() {
        let mut board = Board::empty();

        let color = Color::White;
        let piece1 = Piece::King;
        let from1 = pos("d4");
        let piece2 = Piece::Pawn;
        let from2 = pos("e5");

        let expected = vec![
            ChessMove {
                from: pos("d4"),
                to: pos("d5"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("d3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e4"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c4"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c5"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
        ];

        board.squares[from1].0 = Some((piece1, color));
        board.squares[from2].0 = Some((piece2, color));

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_king_moves(&board, from1, color, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_king_moves_capture_opponent() {
        let mut board = Board::empty();

        let piece1 = Piece::King;
        let color1 = Color::White;
        let from1 = pos("d4");
        let piece2 = Piece::Pawn;
        let color2 = Color::Black;
        let from2 = pos("e5");

        let expected = vec![
            ChessMove {
                from: pos("d4"),
                to: pos("d5"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("d3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e4"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c4"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e5"),
                capture: true,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("e3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c5"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
            ChessMove {
                from: pos("d4"),
                to: pos("c3"),
                capture: false,
                move_type: ChessMoveType::Normal,
            },
        ];

        board.squares[from1].0 = Some((piece1, color1));
        board.squares[from2].0 = Some((piece2, color2));

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_king_moves(&board, from1, color1, &mut moves);

        assert_eq!(moves.len(), expected.len());
        for expected_move in &expected {
            assert!(moves.contains(expected_move));
        }
    }

    #[test]
    fn test_generate_king_moves_castling() {
        // Test white kingside castling (O-O)
        let mut board = Board::empty();
        board.squares[pos("e1")].0 = Some((Piece::King, Color::White));
        board.squares[pos("h1")].0 = Some((Piece::Rook, Color::White));
        board.white_king_pos = pos("e1");
        board.side_to_move = Color::White;

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_king_moves(&board, pos("e1"), Color::White, &mut moves);
        let kingside_castle = ChessMove {
            from: pos("e1"),
            to: pos("g1"),
            capture: false,
            move_type: ChessMoveType::Castle,
        };
        assert!(
            moves.contains(&kingside_castle),
            "White kingside castling should be available"
        );

        // Test white queenside castling (O-O-O)
        let mut board = Board::empty();
        board.squares[pos("e1")].0 = Some((Piece::King, Color::White));
        board.squares[pos("a1")].0 = Some((Piece::Rook, Color::White));
        board.white_king_pos = pos("e1");
        board.side_to_move = Color::White;

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_king_moves(&board, pos("e1"), Color::White, &mut moves);
        let queenside_castle = ChessMove {
            from: pos("e1"),
            to: pos("c1"),
            capture: false,
            move_type: ChessMoveType::Castle,
        };
        assert!(
            moves.contains(&queenside_castle),
            "White queenside castling should be available"
        );

        // Test black kingside castling
        let mut board = Board::empty();
        board.squares[pos("e8")].0 = Some((Piece::King, Color::Black));
        board.squares[pos("h8")].0 = Some((Piece::Rook, Color::Black));
        board.black_king_pos = pos("e8");
        board.side_to_move = Color::Black;

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_king_moves(&board, pos("e8"), Color::Black, &mut moves);
        let kingside_castle = ChessMove {
            from: pos("e8"),
            to: pos("g8"),
            capture: false,
            move_type: ChessMoveType::Castle,
        };
        assert!(
            moves.contains(&kingside_castle),
            "Black kingside castling should be available"
        );

        // Test black queenside castling
        let mut board = Board::empty();
        board.squares[pos("e8")].0 = Some((Piece::King, Color::Black));
        board.squares[pos("a8")].0 = Some((Piece::Rook, Color::Black));
        board.black_king_pos = pos("e8");
        board.side_to_move = Color::Black;

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_king_moves(&board, pos("e8"), Color::Black, &mut moves);
        let queenside_castle = ChessMove {
            from: pos("e8"),
            to: pos("c8"),
            capture: false,
            move_type: ChessMoveType::Castle,
        };
        assert!(
            moves.contains(&queenside_castle),
            "Black queenside castling should be available"
        );

        // Test castling not available when king has moved
        let mut board = Board::empty();
        board.squares[pos("e1")].0 = Some((Piece::King, Color::White));
        board.squares[pos("h1")].0 = Some((Piece::Rook, Color::White));
        board.white_king_pos = pos("e1");
        board.white_king_moved = true;
        board.side_to_move = Color::White;

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_king_moves(&board, pos("e1"), Color::White, &mut moves);
        let kingside_castle = ChessMove {
            from: pos("e1"),
            to: pos("g1"),
            capture: false,
            move_type: ChessMoveType::Castle,
        };
        assert!(
            !moves.contains(&kingside_castle),
            "Castling should not be available when king has moved"
        );

        // Test castling not available when rook has moved
        let mut board = Board::empty();
        board.squares[pos("e1")].0 = Some((Piece::King, Color::White));
        board.squares[pos("h1")].0 = Some((Piece::Rook, Color::White));
        board.white_king_pos = pos("e1");
        board.white_kingside_rook_moved = true;
        board.side_to_move = Color::White;

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_king_moves(&board, pos("e1"), Color::White, &mut moves);
        let kingside_castle = ChessMove {
            from: pos("e1"),
            to: pos("g1"),
            capture: false,
            move_type: ChessMoveType::Castle,
        };
        assert!(
            !moves.contains(&kingside_castle),
            "Castling should not be available when rook has moved"
        );

        // Test castling not available when squares are occupied
        let mut board = Board::empty();
        board.squares[pos("e1")].0 = Some((Piece::King, Color::White));
        board.squares[pos("h1")].0 = Some((Piece::Rook, Color::White));
        board.squares[pos("f1")].0 = Some((Piece::Knight, Color::White)); // blocking piece
        board.white_king_pos = pos("e1");
        board.side_to_move = Color::White;

        let mut moves = Vec::with_capacity(128);
        MoveGenerator::generate_king_moves(&board, pos("e1"), Color::White, &mut moves);
        let kingside_castle = ChessMove {
            from: pos("e1"),
            to: pos("g1"),
            capture: false,
            move_type: ChessMoveType::Castle,
        };
        assert!(
            !moves.contains(&kingside_castle),
            "Castling should not be available when squares are occupied"
        );
    }

    #[test]
    fn test_is_in_check() {
        // Test white king in check from black rook
        let mut board = Board::empty();
        board.squares[pos("e1")].0 = Some((Piece::King, Color::White));
        board.squares[pos("e8")].0 = Some((Piece::Rook, Color::Black));
        board.white_king_pos = pos("e1");
        board.black_king_pos = pos("h8");
        board.side_to_move = Color::White;

        let mut attacked_squares = Vec::with_capacity(64);
        assert!(
            MoveGenerator::is_in_check(&board, Color::White, &mut attacked_squares),
            "White king should be in check from black rook"
        );
        assert!(
            !MoveGenerator::is_in_check(&board, Color::Black, &mut attacked_squares),
            "Black king should not be in check"
        );

        // Test black king in check from white queen
        let mut board = Board::empty();
        board.squares[pos("e8")].0 = Some((Piece::King, Color::Black));
        board.squares[pos("e1")].0 = Some((Piece::Queen, Color::White));
        board.white_king_pos = pos("a1");
        board.black_king_pos = pos("e8");
        board.side_to_move = Color::Black;

        assert!(
            MoveGenerator::is_in_check(&board, Color::Black, &mut attacked_squares),
            "Black king should be in check from white queen"
        );
    }

    #[test]
    fn test_checkmate_back_rank() {
        // Classic back rank mate: white king on g1, black rook on a1, white pawns blocking escape
        let mut board = Board::empty();
        board.squares[pos("g1")].0 = Some((Piece::King, Color::White));
        board.squares[pos("f2")].0 = Some((Piece::Pawn, Color::White));
        board.squares[pos("g2")].0 = Some((Piece::Pawn, Color::White));
        board.squares[pos("h2")].0 = Some((Piece::Pawn, Color::White));
        board.squares[pos("a1")].0 = Some((Piece::Rook, Color::Black)); // Attacks along first rank
        board.squares[pos("e8")].0 = Some((Piece::King, Color::Black));
        board.white_king_pos = pos("g1");
        board.black_king_pos = pos("e8");
        board.side_to_move = Color::White;

        assert!(
            MoveGenerator::is_checkmate(&board),
            "White should be in checkmate (back rank mate)"
        );
    }

    #[test]
    fn test_checkmate_queen_and_king() {
        // Queen and king checkmate in the corner
        let mut board = Board::empty();
        board.squares[pos("a8")].0 = Some((Piece::King, Color::Black));
        board.squares[pos("b6")].0 = Some((Piece::Queen, Color::White));
        board.squares[pos("b8")].0 = Some((Piece::King, Color::White));
        board.white_king_pos = pos("b8");
        board.black_king_pos = pos("a8");
        board.side_to_move = Color::Black;

        assert!(
            MoveGenerator::is_checkmate(&board),
            "Black should be in checkmate (queen and king mate)"
        );
    }

    #[test]
    fn test_stalemate() {
        // Classic stalemate: black king on a8, white king on c7, white queen on c6
        let mut board = Board::empty();
        board.squares[pos("a8")].0 = Some((Piece::King, Color::Black));
        board.squares[pos("c6")].0 = Some((Piece::King, Color::White));
        board.squares[pos("c7")].0 = Some((Piece::Queen, Color::White));
        board.white_king_pos = pos("c6");
        board.black_king_pos = pos("a8");
        board.side_to_move = Color::Black;

        assert!(
            MoveGenerator::is_stalemate(&board),
            "Black should be in stalemate (no legal moves, not in check)"
        );
        assert!(
            !MoveGenerator::is_checkmate(&board),
            "This should be stalemate, not checkmate"
        );
    }

    #[test]
    fn test_not_checkmate_can_block() {
        // King in check but can be blocked
        let mut board = Board::empty();
        board.squares[pos("e1")].0 = Some((Piece::King, Color::White));
        board.squares[pos("e8")].0 = Some((Piece::Rook, Color::Black));
        board.squares[pos("d2")].0 = Some((Piece::Bishop, Color::White)); // Can block on e2
        board.white_king_pos = pos("e1");
        board.black_king_pos = pos("h8");
        board.side_to_move = Color::White;

        let mut attacked_squares = Vec::with_capacity(64);
        assert!(
            MoveGenerator::is_in_check(&board, Color::White, &mut attacked_squares),
            "White should be in check"
        );
        assert!(
            !MoveGenerator::is_checkmate(&board),
            "White should not be in checkmate (can block with bishop)"
        );
    }

    #[test]
    fn test_not_checkmate_can_escape() {
        // King in check but has escape square
        let mut board = Board::empty();
        board.squares[pos("e4")].0 = Some((Piece::King, Color::White));
        board.squares[pos("e8")].0 = Some((Piece::Rook, Color::Black));
        board.white_king_pos = pos("e4");
        board.black_king_pos = pos("h8");
        board.side_to_move = Color::White;

        let mut attacked_squares = Vec::with_capacity(64);
        assert!(
            MoveGenerator::is_in_check(&board, Color::White, &mut attacked_squares),
            "White should be in check"
        );
        assert!(
            !MoveGenerator::is_checkmate(&board),
            "White should not be in checkmate (king can move)"
        );
    }

    #[test]
    fn test_not_checkmate_can_capture_attacker() {
        // King in check but can capture the attacking piece
        let mut board = Board::empty();
        board.squares[pos("e1")].0 = Some((Piece::King, Color::White));
        board.squares[pos("e2")].0 = Some((Piece::Rook, Color::Black)); // King can capture
        board.white_king_pos = pos("e1");
        board.black_king_pos = pos("h8");
        board.side_to_move = Color::White;

        let mut attacked_squares = Vec::with_capacity(64);
        assert!(
            MoveGenerator::is_in_check(&board, Color::White, &mut attacked_squares),
            "White should be in check"
        );
        assert!(
            !MoveGenerator::is_checkmate(&board),
            "White should not be in checkmate (can capture rook)"
        );
    }
}
