use crate::board::{Board, Color, Piece, castling::Side};

/// Convert a Board to a FEN string
///
/// Note: Board doesn't track fullmove number, so it defaults to 1
pub fn board_fen(board: &Board) -> String {
    let mut fen = String::new();

    // Part 1: Piece placement (from rank 8 down to rank 1)
    for rank in (0..8).rev() {
        let mut empty_count = 0;

        for file in 0..8 {
            let sq = rank * 8 + file;
            let bit = 1u64 << sq;

            // Check if there's a piece on this square
            if let Some((color, piece)) = find_piece_at(board, bit) {
                // Write any accumulated empty squares first
                if empty_count > 0 {
                    fen.push_str(&empty_count.to_string());
                    empty_count = 0;
                }

                // Write the piece character
                let ch = piece_to_char(color, piece);
                fen.push(ch);
            } else {
                empty_count += 1;
            }
        }

        // Write any remaining empty squares for this rank
        if empty_count > 0 {
            fen.push_str(&empty_count.to_string());
        }

        // Add rank separator (except after the last rank)
        if rank > 0 {
            fen.push('/');
        }
    }

    // Part 2: Active color
    fen.push(' ');
    fen.push(match board.side_to_move {
        Color::White => 'w',
        Color::Black => 'b',
    });

    // Part 3: Castling availability
    fen.push(' ');
    let mut castling = String::new();
    if board.castling.has(Color::White, Side::KingSide) {
        castling.push('K');
    }
    if board.castling.has(Color::White, Side::QueenSide) {
        castling.push('Q');
    }
    if board.castling.has(Color::Black, Side::KingSide) {
        castling.push('k');
    }
    if board.castling.has(Color::Black, Side::QueenSide) {
        castling.push('q');
    }
    if castling.is_empty() {
        fen.push('-');
    } else {
        fen.push_str(&castling);
    }

    // Part 4: En passant target square
    fen.push(' ');
    if board.en_passant < 64 {
        let file = board.en_passant % 8;
        let rank = board.en_passant / 8;
        let file_char = (b'a' + file) as char;
        let rank_char = (b'1' + rank) as char;
        fen.push(file_char);
        fen.push(rank_char);
    } else {
        fen.push('-');
    }

    // Part 5: Halfmove clock
    fen.push(' ');
    fen.push_str(&board.halfmove_clock.to_string());

    // Part 6: Fullmove number (Board doesn't track this, default to 1)
    fen.push_str(" 1");

    fen
}

/// Find which piece (if any) is at the given bit position
fn find_piece_at(board: &Board, bit: u64) -> Option<(Color, Piece)> {
    for color in [Color::White, Color::Black] {
        for piece in [
            Piece::Pawn,
            Piece::Knight,
            Piece::Bishop,
            Piece::Rook,
            Piece::Queen,
            Piece::King,
        ] {
            if board.pieces[color as usize][piece as usize] & bit != 0 {
                return Some((color, piece));
            }
        }
    }
    None
}

/// Convert a piece and color to its FEN character
fn piece_to_char(color: Color, piece: Piece) -> char {
    match (color, piece) {
        (Color::White, Piece::Pawn) => 'P',
        (Color::White, Piece::Knight) => 'N',
        (Color::White, Piece::Bishop) => 'B',
        (Color::White, Piece::Rook) => 'R',
        (Color::White, Piece::Queen) => 'Q',
        (Color::White, Piece::King) => 'K',
        (Color::Black, Piece::Pawn) => 'p',
        (Color::Black, Piece::Knight) => 'n',
        (Color::Black, Piece::Bishop) => 'b',
        (Color::Black, Piece::Rook) => 'r',
        (Color::Black, Piece::Queen) => 'q',
        (Color::Black, Piece::King) => 'k',
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_fen_starting_position() {
        let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let fen = board_fen(&board);
        assert_eq!(
            fen,
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
    }

    #[test]
    fn test_board_fen_with_en_passant() {
        let board = Board::from_fen("rnbqkbnr/pppp1ppp/8/4pP2/8/8/PPPPP1PP/RNBQKBNR w KQkq e6 0 1");
        let fen = board_fen(&board);
        assert_eq!(
            fen,
            "rnbqkbnr/pppp1ppp/8/4pP2/8/8/PPPPP1PP/RNBQKBNR w KQkq e6 0 1"
        );
    }

    #[test]
    fn test_board_fen_no_castling() {
        let board = Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w - - 0 1");
        let fen = board_fen(&board);
        assert_eq!(fen, "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w - - 0 1");
    }

    #[test]
    fn test_board_fen_black_to_move() {
        let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
        let fen = board_fen(&board);
        assert_eq!(
            fen,
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"
        );
    }

    #[test]
    fn test_board_fen_partial_castling() {
        let board = Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w Kq - 5 1");
        let fen = board_fen(&board);
        assert_eq!(fen, "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w Kq - 5 1");
    }
}
