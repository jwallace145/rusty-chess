use crate::board::{Board2, ChessMove, Color, Piece, chess_move::ChessMoveType};

pub struct MoveGenerator;

impl MoveGenerator {
    pub fn generate_legal_moves(board: &Board2, moves: &mut Vec<ChessMove>) {
        moves.clear();

        // Generate all pseudo-legal moves
        Self::generate_pseudo_moves(board, moves);

        // Filter out moves that leave the king in check
        let mut i = 0;
        while i < moves.len() {
            let mv = moves[i];

            // Test if move is legal by making it on a copy
            let mut board_copy = *board;
            board_copy.make_move(mv);

            // Check if our king is in check after the move
            if board_copy.in_check(board.side_to_move) {
                // Illegal move - remove it
                moves.swap_remove(i);
            } else {
                // Legal move - keep it
                i += 1;
            }
        }
    }

    fn generate_pseudo_moves(board: &Board2, moves: &mut Vec<ChessMove>) {
        let us = board.side_to_move;

        // Generate moves for each piece type
        Self::generate_pawn_moves(board, us, moves);
        Self::generate_knight_moves(board, us, moves);
        Self::generate_bishop_moves(board, us, moves);
        Self::generate_rook_moves(board, us, moves);
        Self::generate_queen_moves(board, us, moves);
        Self::generate_king_moves(board, us, moves);
    }

    fn generate_pawn_moves(board: &Board2, color: Color, moves: &mut Vec<ChessMove>) {
        let mut pawns = board.pieces_of(color, Piece::Pawn);
        let empty = board.empty();
        let them = board.occupancy(color.opponent());

        let (forward, start_rank, promo_rank): (i8, u8, u8) = match color {
            Color::White => (8, 1, 6),
            Color::Black => (-8, 6, 1),
        };

        // Iterate over all pawns
        while pawns != 0 {
            let from = pawns.trailing_zeros() as u8;
            pawns &= pawns - 1; // Clear the least significant bit

            let from_rank = from / 8;
            let to_sq = (from as i8 + forward) as u8;

            // Single push
            if to_sq < 64 && (empty & (1u64 << to_sq)) != 0 {
                let to_rank = to_sq / 8;
                if to_rank == promo_rank {
                    // Promotion
                    for promo_piece in [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
                        moves.push(ChessMove {
                            from: from as usize,
                            to: to_sq as usize,
                            capture: false,
                            move_type: ChessMoveType::Promotion(promo_piece),
                        });
                    }
                } else {
                    moves.push(ChessMove {
                        from: from as usize,
                        to: to_sq as usize,
                        capture: false,
                        move_type: ChessMoveType::Normal,
                    });

                    // Double push from starting position
                    if from_rank == start_rank {
                        let to_sq2 = (from as i8 + 2 * forward) as u8;
                        if (empty & (1u64 << to_sq2)) != 0 {
                            moves.push(ChessMove {
                                from: from as usize,
                                to: to_sq2 as usize,
                                capture: false,
                                move_type: ChessMoveType::Normal,
                            });
                        }
                    }
                }
            }

            // Captures
            let attacks = board.attacks_from(Piece::Pawn, from, color);
            let mut captures = attacks & them;

            while captures != 0 {
                let to = captures.trailing_zeros() as u8;
                captures &= captures - 1;

                let to_rank = to / 8;
                if to_rank == promo_rank {
                    // Capture promotion
                    for promo_piece in [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
                        moves.push(ChessMove {
                            from: from as usize,
                            to: to as usize,
                            capture: true,
                            move_type: ChessMoveType::Promotion(promo_piece),
                        });
                    }
                } else {
                    moves.push(ChessMove {
                        from: from as usize,
                        to: to as usize,
                        capture: true,
                        move_type: ChessMoveType::Normal,
                    });
                }
            }

            // En passant
            if board.en_passant < 64 {
                let ep_square = board.en_passant;
                if (attacks & (1u64 << ep_square)) != 0 {
                    moves.push(ChessMove {
                        from: from as usize,
                        to: ep_square as usize,
                        capture: true,
                        move_type: ChessMoveType::EnPassant,
                    });
                }
            }
        }
    }

    fn generate_knight_moves(board: &Board2, color: Color, moves: &mut Vec<ChessMove>) {
        let mut knights = board.pieces_of(color, Piece::Knight);
        let our_pieces = board.occupancy(color);

        while knights != 0 {
            let from = knights.trailing_zeros() as u8;
            knights &= knights - 1;

            let attacks = board.attacks_from(Piece::Knight, from, color);
            let mut targets = attacks & !our_pieces;

            while targets != 0 {
                let to = targets.trailing_zeros() as u8;
                targets &= targets - 1;

                let is_capture = (board.occupancy(color.opponent()) & (1u64 << to)) != 0;
                moves.push(ChessMove {
                    from: from as usize,
                    to: to as usize,
                    capture: is_capture,
                    move_type: ChessMoveType::Normal,
                });
            }
        }
    }

    fn generate_bishop_moves(board: &Board2, color: Color, moves: &mut Vec<ChessMove>) {
        let mut bishops = board.pieces_of(color, Piece::Bishop);
        let our_pieces = board.occupancy(color);

        while bishops != 0 {
            let from = bishops.trailing_zeros() as u8;
            bishops &= bishops - 1;

            let attacks = board.attacks_from(Piece::Bishop, from, color);
            let mut targets = attacks & !our_pieces;

            while targets != 0 {
                let to = targets.trailing_zeros() as u8;
                targets &= targets - 1;

                let is_capture = (board.occupancy(color.opponent()) & (1u64 << to)) != 0;
                moves.push(ChessMove {
                    from: from as usize,
                    to: to as usize,
                    capture: is_capture,
                    move_type: ChessMoveType::Normal,
                });
            }
        }
    }

    fn generate_rook_moves(board: &Board2, color: Color, moves: &mut Vec<ChessMove>) {
        let mut rooks = board.pieces_of(color, Piece::Rook);
        let our_pieces = board.occupancy(color);

        while rooks != 0 {
            let from = rooks.trailing_zeros() as u8;
            rooks &= rooks - 1;

            let attacks = board.attacks_from(Piece::Rook, from, color);
            let mut targets = attacks & !our_pieces;

            while targets != 0 {
                let to = targets.trailing_zeros() as u8;
                targets &= targets - 1;

                let is_capture = (board.occupancy(color.opponent()) & (1u64 << to)) != 0;
                moves.push(ChessMove {
                    from: from as usize,
                    to: to as usize,
                    capture: is_capture,
                    move_type: ChessMoveType::Normal,
                });
            }
        }
    }

    fn generate_queen_moves(board: &Board2, color: Color, moves: &mut Vec<ChessMove>) {
        let mut queens = board.pieces_of(color, Piece::Queen);
        let our_pieces = board.occupancy(color);

        while queens != 0 {
            let from = queens.trailing_zeros() as u8;
            queens &= queens - 1;

            let attacks = board.attacks_from(Piece::Queen, from, color);
            let mut targets = attacks & !our_pieces;

            while targets != 0 {
                let to = targets.trailing_zeros() as u8;
                targets &= targets - 1;

                let is_capture = (board.occupancy(color.opponent()) & (1u64 << to)) != 0;
                moves.push(ChessMove {
                    from: from as usize,
                    to: to as usize,
                    capture: is_capture,
                    move_type: ChessMoveType::Normal,
                });
            }
        }
    }

    fn generate_king_moves(board: &Board2, color: Color, moves: &mut Vec<ChessMove>) {
        let king_sq = board.king_sq[color as usize];
        let our_pieces = board.occupancy(color);

        // Normal king moves
        let attacks = board.attacks_from(Piece::King, king_sq, color);
        let mut targets = attacks & !our_pieces;

        while targets != 0 {
            let to = targets.trailing_zeros() as u8;
            targets &= targets - 1;

            let is_capture = (board.occupancy(color.opponent()) & (1u64 << to)) != 0;
            moves.push(ChessMove {
                from: king_sq as usize,
                to: to as usize,
                capture: is_capture,
                move_type: ChessMoveType::Normal,
            });
        }

        // Castling
        Self::generate_castling_moves(board, color, moves);
    }

    fn generate_castling_moves(board: &Board2, color: Color, moves: &mut Vec<ChessMove>) {
        use crate::board::castling::Side;

        match color {
            Color::White => {
                // Kingside castling
                if board.castling.has(Color::White, Side::KingSide) {
                    let king_sq = 4; // e1

                    // Check if squares between king and rook are empty
                    if board.piece_on(5).is_none() && board.piece_on(6).is_none() {
                        // Check if king is not in check
                        if !board.in_check(Color::White) {
                            // Check if king doesn't move through check (f1)
                            if !board.is_square_attacked(5, Color::Black) {
                                // Check if king doesn't land in check (g1) - this will be verified later
                                moves.push(ChessMove {
                                    from: king_sq,
                                    to: 6, // g1
                                    capture: false,
                                    move_type: ChessMoveType::Castle,
                                });
                            }
                        }
                    }
                }

                // Queenside castling
                if board.castling.has(Color::White, Side::QueenSide) {
                    let king_sq = 4; // e1

                    // Check if squares between king and rook are empty
                    if board.piece_on(3).is_none()
                        && board.piece_on(2).is_none()
                        && board.piece_on(1).is_none()
                    {
                        // Check if king is not in check
                        if !board.in_check(Color::White) {
                            // Check if king doesn't move through check (d1)
                            if !board.is_square_attacked(3, Color::Black) {
                                moves.push(ChessMove {
                                    from: king_sq,
                                    to: 2, // c1
                                    capture: false,
                                    move_type: ChessMoveType::Castle,
                                });
                            }
                        }
                    }
                }
            }
            Color::Black => {
                // Kingside castling
                if board.castling.has(Color::Black, Side::KingSide) {
                    let king_sq = 60; // e8

                    // Check if squares between king and rook are empty
                    if board.piece_on(61).is_none() && board.piece_on(62).is_none() {
                        // Check if king is not in check
                        if !board.in_check(Color::Black) {
                            // Check if king doesn't move through check (f8)
                            if !board.is_square_attacked(61, Color::White) {
                                moves.push(ChessMove {
                                    from: king_sq,
                                    to: 62, // g8
                                    capture: false,
                                    move_type: ChessMoveType::Castle,
                                });
                            }
                        }
                    }
                }

                // Queenside castling
                if board.castling.has(Color::Black, Side::QueenSide) {
                    let king_sq = 60; // e8

                    // Check if squares between king and rook are empty
                    if board.piece_on(59).is_none()
                        && board.piece_on(58).is_none()
                        && board.piece_on(57).is_none()
                    {
                        // Check if king is not in check
                        if !board.in_check(Color::Black) {
                            // Check if king doesn't move through check (d8)
                            if !board.is_square_attacked(59, Color::White) {
                                moves.push(ChessMove {
                                    from: king_sq,
                                    to: 58, // c8
                                    capture: false,
                                    move_type: ChessMoveType::Castle,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn is_checkmate(board: &Board2) -> bool {
        let mut legal_moves = Vec::with_capacity(128);
        Self::generate_legal_moves(board, &mut legal_moves);
        legal_moves.is_empty() && board.in_check(board.side_to_move)
    }

    pub fn is_stalemate(board: &Board2) -> bool {
        let mut legal_moves = Vec::with_capacity(128);
        Self::generate_legal_moves(board, &mut legal_moves);
        legal_moves.is_empty() && !board.in_check(board.side_to_move)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::board2::Board2;

    #[test]
    fn test_simple_pawn_moves() {
        // Test basic pawn move generation
        let mut board = Board2::new_empty();

        // White pawn on e2
        board.pieces[Color::White as usize][Piece::Pawn as usize] = 1u64 << 12; // e2
        board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 4; // e1
        board.king_sq[Color::White as usize] = 4;

        // Black king far away
        board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 60; // e8
        board.king_sq[Color::Black as usize] = 60;

        // Update occupancy
        board.occ[Color::White as usize] = (1u64 << 12) | (1u64 << 4);
        board.occ[Color::Black as usize] = 1u64 << 60;
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

        board.side_to_move = Color::White;

        let mut moves = Vec::new();
        MoveGenerator::generate_legal_moves(&board, &mut moves);

        // Pawn on e2 should be able to move to e3 and e4 (2 moves)
        // King on e1 should be able to move to d1, f1, d2, f2 (4 moves) - e2 is blocked by pawn
        // Total: 6 moves
        assert_eq!(moves.len(), 6);
    }

    #[test]
    fn test_no_moves_in_checkmate() {
        // Create a simple checkmate position
        let mut board = Board2::new_empty();

        // White king in corner
        board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 0; // a1
        board.king_sq[Color::White as usize] = 0;

        // Black queen and king for checkmate
        board.pieces[Color::Black as usize][Piece::Queen as usize] = 1u64 << 9; // b2
        board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 18; // c3
        board.king_sq[Color::Black as usize] = 18;

        // Update occupancy
        board.occ[Color::White as usize] = 1u64 << 0;
        board.occ[Color::Black as usize] = (1u64 << 9) | (1u64 << 18);
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

        board.side_to_move = Color::White;

        let mut moves = Vec::new();
        MoveGenerator::generate_legal_moves(&board, &mut moves);

        // White should have no legal moves (checkmated)
        assert_eq!(moves.len(), 0);
    }

    #[test]
    fn test_king_cannot_move_into_check() {
        // King can't move into check
        let mut board = Board2::new_empty();

        // White king in center
        board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 28; // e4
        board.king_sq[Color::White as usize] = 28;

        // Black rook attacking e5
        board.pieces[Color::Black as usize][Piece::Rook as usize] = 1u64 << 60; // e8
        board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 63; // h8
        board.king_sq[Color::Black as usize] = 63;

        // Update occupancy
        board.occ[Color::White as usize] = 1u64 << 28;
        board.occ[Color::Black as usize] = (1u64 << 60) | (1u64 << 63);
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

        board.side_to_move = Color::White;

        let mut moves = Vec::new();
        MoveGenerator::generate_legal_moves(&board, &mut moves);

        // King should not be able to move to e5 (checked by rook)
        let e5_move = moves.iter().find(|m| m.to == 36); // e5 = 36
        assert!(
            e5_move.is_none(),
            "King should not be able to move into check"
        );

        // But king should still have other legal moves
        assert!(!moves.is_empty());
    }

    #[test]
    fn test_pinned_piece_cannot_move() {
        // A pinned piece cannot move if it exposes the king
        let mut board = Board2::new_empty();

        // White king on e1
        board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 4; // e1
        board.king_sq[Color::White as usize] = 4;

        // White rook on e2 (pinned)
        board.pieces[Color::White as usize][Piece::Rook as usize] = 1u64 << 12; // e2

        // Black rook on e8 (pinning the white rook)
        board.pieces[Color::Black as usize][Piece::Rook as usize] = 1u64 << 60; // e8
        board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 63; // h8
        board.king_sq[Color::Black as usize] = 63;

        // Update occupancy
        board.occ[Color::White as usize] = (1u64 << 4) | (1u64 << 12);
        board.occ[Color::Black as usize] = (1u64 << 60) | (1u64 << 63);
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

        board.side_to_move = Color::White;

        let mut moves = Vec::new();
        MoveGenerator::generate_legal_moves(&board, &mut moves);

        // The white rook should not be able to move horizontally (only vertically along the pin)
        // It can only move to e3-e7 (capturing the black rook)
        for mv in &moves {
            if mv.from == 12 {
                // e2
                let to_file = mv.to % 8;
                assert_eq!(to_file, 4, "Pinned rook should only move along the e-file");
            }
        }
    }
}
