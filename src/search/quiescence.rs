use crate::board::{Board, ChessMove, Color, Piece, chess_move::ChessMoveType};
use crate::eval::Evaluator;

/// Generates noisy moves: captures, pawn promotions, and en passant.
/// These are the only moves searched in quiescence search to resolve tactical sequences.
pub fn generate_noisy_moves(board: &Board, moves: &mut Vec<ChessMove>) {
    moves.clear();

    let us = board.side_to_move;
    let them = board.occupancy(us.opponent());

    // Generate pawn captures, promotions, and en passant
    generate_pawn_noisy_moves(board, us, them, moves);

    // Generate knight captures
    generate_piece_captures(board, us, them, Piece::Knight, moves);

    // Generate bishop captures
    generate_piece_captures(board, us, them, Piece::Bishop, moves);

    // Generate rook captures
    generate_piece_captures(board, us, them, Piece::Rook, moves);

    // Generate queen captures
    generate_piece_captures(board, us, them, Piece::Queen, moves);

    // Generate king captures
    generate_king_captures(board, us, them, moves);

    // Filter out illegal moves (those that leave king in check)
    filter_illegal_moves(board, moves);
}

/// Generate pawn captures, promotions (including non-capture promotions), and en passant
fn generate_pawn_noisy_moves(board: &Board, color: Color, them: u64, moves: &mut Vec<ChessMove>) {
    let mut pawns = board.pieces_of(color, Piece::Pawn);
    let empty = board.empty();

    let (forward, promo_rank): (i8, u8) = match color {
        Color::White => (8, 7),  // Promote on 8th rank (rank 7, 0-indexed)
        Color::Black => (-8, 0), // Promote on 1st rank (rank 0, 0-indexed)
    };

    while pawns != 0 {
        let from = pawns.trailing_zeros() as u8;
        pawns &= pawns - 1;

        let to_sq = (from as i8 + forward) as u8;

        // Non-capture promotions (pushing to promotion rank)
        if to_sq < 64 && (empty & (1u64 << to_sq)) != 0 {
            let to_rank = to_sq / 8;
            if to_rank == promo_rank {
                for promo_piece in [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
                    moves.push(ChessMove {
                        from: from as usize,
                        to: to_sq as usize,
                        capture: false,
                        move_type: ChessMoveType::Promotion(promo_piece),
                    });
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

/// Generate captures for a given piece type (knight, bishop, rook, queen)
fn generate_piece_captures(
    board: &Board,
    color: Color,
    them: u64,
    piece: Piece,
    moves: &mut Vec<ChessMove>,
) {
    let mut pieces = board.pieces_of(color, piece);

    while pieces != 0 {
        let from = pieces.trailing_zeros() as u8;
        pieces &= pieces - 1;

        let attacks = board.attacks_from(piece, from, color);
        let mut captures = attacks & them;

        while captures != 0 {
            let to = captures.trailing_zeros() as u8;
            captures &= captures - 1;

            moves.push(ChessMove {
                from: from as usize,
                to: to as usize,
                capture: true,
                move_type: ChessMoveType::Normal,
            });
        }
    }
}

/// Generate king captures
fn generate_king_captures(board: &Board, color: Color, them: u64, moves: &mut Vec<ChessMove>) {
    let king_sq = board.king_sq[color as usize];
    let attacks = board.attacks_from(Piece::King, king_sq, color);
    let mut captures = attacks & them;

    while captures != 0 {
        let to = captures.trailing_zeros() as u8;
        captures &= captures - 1;

        moves.push(ChessMove {
            from: king_sq as usize,
            to: to as usize,
            capture: true,
            move_type: ChessMoveType::Normal,
        });
    }
}

/// Filter out illegal moves that leave the king in check
fn filter_illegal_moves(board: &Board, moves: &mut Vec<ChessMove>) {
    let mut i = 0;
    while i < moves.len() {
        let mv = moves[i];

        let mut board_copy = *board;
        board_copy.make_move(mv);

        if board_copy.in_check(board.side_to_move) {
            moves.swap_remove(i);
        } else {
            i += 1;
        }
    }
}

/// Order noisy moves by MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
/// This improves alpha-beta pruning efficiency in quiescence search.
fn order_noisy_moves(board: &Board, moves: &mut [ChessMove]) {
    moves.sort_by_key(|m| {
        // Get victim value
        let victim_value = if m.capture {
            if let Some((_, piece)) = board.piece_on(m.to as u8) {
                piece_value(piece)
            } else {
                100 // En passant captures a pawn
            }
        } else {
            // Non-capture promotion - treat as high priority
            if matches!(m.move_type, ChessMoveType::Promotion(_)) {
                return -10000; // Promotions first
            }
            0
        };

        // Get attacker value
        let attacker_value = if let Some((_, piece)) = board.piece_on(m.from as u8) {
            piece_value(piece)
        } else {
            0
        };

        // MVV-LVA: prioritize high victim value, low attacker value
        // Negative for descending sort order
        -(victim_value * 10 - attacker_value)
    });
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

/// Quiescence search to resolve tactical sequences at leaf nodes.
///
/// This function searches only "noisy" moves (captures, promotions) to avoid
/// the horizon effect where the search stops at a tactically unstable position.
///
/// # Arguments
/// * `board` - The current board position
/// * `alpha` - The current alpha bound (best score for the maximizing player)
/// * `beta` - The current beta bound (best score for the minimizing player)
/// * `evaluator` - The static evaluation function
///
/// # Returns
/// The evaluation score from the perspective of the side to move
pub fn quiescence_search(board: &Board, mut alpha: i32, beta: i32, evaluator: &Evaluator) -> i32 {
    // Stand pat: evaluate the current position
    let stand_pat = evaluator.evaluate(board);

    // Beta cutoff: position is already too good for the opponent
    if stand_pat >= beta {
        return beta;
    }

    // Update alpha if stand_pat is better
    if stand_pat > alpha {
        alpha = stand_pat;
    }

    // Generate all noisy moves
    let mut moves = Vec::with_capacity(64);
    generate_noisy_moves(board, &mut moves);

    // If no noisy moves, return stand_pat (position is quiet)
    if moves.is_empty() {
        return alpha;
    }

    // Order moves for better pruning
    order_noisy_moves(board, &mut moves);

    // Search each noisy move
    for chess_move in moves {
        let mut board_copy = *board;
        board_copy.make_move(chess_move);

        // Recursively search with negated alpha/beta (negamax framework)
        let score = -quiescence_search(&board_copy, -beta, -alpha, evaluator);

        // Beta cutoff
        if score >= beta {
            return beta;
        }

        // Update alpha
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_noisy_moves_captures() {
        let mut board = Board::new_empty();

        // White pawn on d4, black pawn on e5
        board.pieces[Color::White as usize][Piece::Pawn as usize] = 1u64 << 27; // d4
        board.pieces[Color::Black as usize][Piece::Pawn as usize] = 1u64 << 36; // e5

        // Kings
        board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 4; // e1
        board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 60; // e8
        board.king_sq[Color::White as usize] = 4;
        board.king_sq[Color::Black as usize] = 60;

        // Update occupancy
        board.occ[Color::White as usize] = (1u64 << 27) | (1u64 << 4);
        board.occ[Color::Black as usize] = (1u64 << 36) | (1u64 << 60);
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

        board.side_to_move = Color::White;

        let mut moves = Vec::new();
        generate_noisy_moves(&board, &mut moves);

        // Should have exactly one capture: d4xe5
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0].from, 27); // d4
        assert_eq!(moves[0].to, 36); // e5
        assert!(moves[0].capture);
    }

    #[test]
    fn test_generate_noisy_moves_promotion() {
        let mut board = Board::new_empty();

        // White pawn on e7 (about to promote)
        board.pieces[Color::White as usize][Piece::Pawn as usize] = 1u64 << 52; // e7

        // Kings
        board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 4; // e1
        board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 63; // h8
        board.king_sq[Color::White as usize] = 4;
        board.king_sq[Color::Black as usize] = 63;

        // Update occupancy
        board.occ[Color::White as usize] = (1u64 << 52) | (1u64 << 4);
        board.occ[Color::Black as usize] = 1u64 << 63;
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

        board.side_to_move = Color::White;

        let mut moves = Vec::new();
        generate_noisy_moves(&board, &mut moves);

        // Should have 4 promotion moves (Q, R, B, N)
        assert_eq!(moves.len(), 4);
        for mv in &moves {
            assert!(matches!(mv.move_type, ChessMoveType::Promotion(_)));
            assert_eq!(mv.from, 52); // e7
            assert_eq!(mv.to, 60); // e8
        }
    }

    #[test]
    fn test_quiescence_search_quiet_position() {
        // In a quiet position with no captures, quiescence should return stand_pat
        let board = Board::startpos();
        let evaluator = Evaluator::new();

        let score = quiescence_search(&board, i32::MIN + 1, i32::MAX, &evaluator);

        // Score should be close to 0 in the starting position
        assert!(
            score.abs() < 100,
            "Starting position should be roughly equal"
        );
    }

    #[test]
    fn test_quiescence_search_winning_capture() {
        let mut board = Board::new_empty();

        // White has a queen that can capture a hanging black queen
        board.pieces[Color::White as usize][Piece::Queen as usize] = 1u64 << 27; // d4
        board.pieces[Color::Black as usize][Piece::Queen as usize] = 1u64 << 36; // e5

        // Kings
        board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 4; // e1
        board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 60; // e8
        board.king_sq[Color::White as usize] = 4;
        board.king_sq[Color::Black as usize] = 60;

        // Update occupancy
        board.occ[Color::White as usize] = (1u64 << 27) | (1u64 << 4);
        board.occ[Color::Black as usize] = (1u64 << 36) | (1u64 << 60);
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

        board.side_to_move = Color::White;

        let evaluator = Evaluator::new();
        let score = quiescence_search(&board, i32::MIN + 1, i32::MAX, &evaluator);

        // After capturing the queen, white should be significantly ahead
        // Score should be positive and substantial
        assert!(score > 500, "White should be winning after queen capture");
    }
}
