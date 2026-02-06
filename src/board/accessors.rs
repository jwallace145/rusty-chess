use super::Board;
use crate::board::{Color, Piece};

impl Board {
    /// Return the pieces as a `u64` bitboard of the given color
    /// and piece type.
    #[inline]
    pub fn pieces_of(&self, color: Color, piece: Piece) -> u64 {
        self.pieces[color as usize][piece as usize]
    }

    /// Return the current positions of the given color as a
    /// `u64` bitboard.
    ///
    /// E.g, if a `1` occupies a certain bit in the `u64` bitboard,
    /// then that square is occupied by a piece of the given color.
    #[inline]
    pub fn occupancy(&self, color: Color) -> u64 {
        self.occ[color as usize]
    }

    /// Return all of the currently occupied chess board squares
    /// as a `u64` bitboard (for black and white pieces).
    #[inline]
    pub fn occupied(&self) -> u64 {
        self.occ_all
    }

    /// Return if the board is empty as a `u64` bitboard.
    ///
    /// E.g., If the board is empty, then no pieces will
    /// be currently occupying it and the negation of that
    /// occupancy bitboard returns all 1's (in other words
    /// `true`).
    #[inline]
    pub fn empty(&self) -> u64 {
        !self.occ_all
    }

    /// Return the piece, if one exists, on the given chess board
    /// square.
    pub fn piece_on(&self, sq: u8) -> Option<(Color, Piece)> {
        let mask: u64 = 1u64 << sq;

        if self.occ_all & mask == 0 {
            return None;
        }

        let color: Color = if self.occ[Color::White as usize] & mask != 0 {
            Color::White
        } else {
            Color::Black
        };
        let c: usize = color as usize;

        for p in 0..6 {
            if self.pieces[c][p] & mask != 0 {
                let piece = match p {
                    0 => Piece::Pawn,
                    1 => Piece::Knight,
                    2 => Piece::Bishop,
                    3 => Piece::Rook,
                    4 => Piece::Queen,
                    _ => Piece::King,
                };
                return Some((color, piece));
            }
        }

        // This code should be unreachable unless the board state is corrupt...
        // In that case, panic and log some diagnostic information to help troubleshoot
        panic!(
            "Corrupt board state: square {} is set in occ_all (occ_all: 0x{:016x}, \
             occ[white]: 0x{:016x}, occ[black]: 0x{:016x}) and color {:?} occupancy, \
             but no piece bitboard contains it (pieces[{:?}]: {:?})",
            sq, self.occ_all, self.occ[0], self.occ[1], color, color, self.pieces[c]
        )
    }

    /// Return the current position of the king on the chessboard for
    /// the given color.
    pub fn king_square(&self, color: Color) -> u8 {
        self.king_sq[color as usize]
    }

    /// Return true or false as to whether the given color has castled or not.
    pub fn has_castled(&self, color: Color) -> bool {
        let king_sq = self.king_sq[color as usize];
        match color {
            Color::White => {
                // White king castled if it's on g1 (6) or c1 (2)
                king_sq == 6 || king_sq == 2
            }
            Color::Black => {
                // Black king castled if it's on g8 (62) or c8 (58)
                king_sq == 62 || king_sq == 58
            }
        }
    }

    /// Return the number of pieces of the given color and piece type
    /// remaining on the chess board.
    pub fn count_pieces(&self, color: Color, piece: Piece) -> u32 {
        self.pieces[color as usize][piece as usize].count_ones()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Color, Piece};

    // ── pieces_of ──────────────────────────────────────────────

    #[test]
    fn pieces_of_white_pawns_startpos() {
        let board = Board::startpos();
        // Rank 2 (bits 8..=15) should all be white pawns
        assert_eq!(
            board.pieces_of(Color::White, Piece::Pawn),
            0x0000_0000_0000_FF00
        );
    }

    #[test]
    fn pieces_of_black_pawns_startpos() {
        let board = Board::startpos();
        // Rank 7 (bits 48..=55)
        assert_eq!(
            board.pieces_of(Color::Black, Piece::Pawn),
            0x00FF_0000_0000_0000
        );
    }

    #[test]
    fn pieces_of_white_knights_startpos() {
        let board = Board::startpos();
        // b1 (1) and g1 (6)
        assert_eq!(
            board.pieces_of(Color::White, Piece::Knight),
            (1u64 << 1) | (1u64 << 6)
        );
    }

    #[test]
    fn pieces_of_empty_board() {
        let board = Board::new_empty();
        assert_eq!(board.pieces_of(Color::White, Piece::Pawn), 0);
        assert_eq!(board.pieces_of(Color::Black, Piece::King), 0);
    }

    // ── occupancy ──────────────────────────────────────────────

    #[test]
    fn occupancy_white_startpos() {
        let board = Board::startpos();
        // Ranks 1-2 (bits 0..=15)
        assert_eq!(board.occupancy(Color::White), 0x0000_0000_0000_FFFF);
    }

    #[test]
    fn occupancy_black_startpos() {
        let board = Board::startpos();
        // Ranks 7-8 (bits 48..=63)
        assert_eq!(board.occupancy(Color::Black), 0xFFFF_0000_0000_0000);
    }

    #[test]
    fn occupancy_empty_board() {
        let board = Board::new_empty();
        assert_eq!(board.occupancy(Color::White), 0);
        assert_eq!(board.occupancy(Color::Black), 0);
    }

    // ── occupied / empty ───────────────────────────────────────

    #[test]
    fn occupied_startpos() {
        let board = Board::startpos();
        assert_eq!(board.occupied(), 0xFFFF_0000_0000_FFFF);
    }

    #[test]
    fn empty_startpos() {
        let board = Board::startpos();
        // Middle four ranks should be empty
        assert_eq!(board.empty(), !0xFFFF_0000_0000_FFFF);
    }

    #[test]
    fn occupied_and_empty_are_complementary() {
        let board = Board::startpos();
        assert_eq!(board.occupied() ^ board.empty(), u64::MAX);
    }

    #[test]
    fn empty_board_everything_empty() {
        let board = Board::new_empty();
        assert_eq!(board.occupied(), 0);
        assert_eq!(board.empty(), u64::MAX);
    }

    // ── piece_on ───────────────────────────────────────────────

    #[test]
    fn piece_on_a1_is_white_rook() {
        let board = Board::startpos();
        assert_eq!(board.piece_on(0), Some((Color::White, Piece::Rook)));
    }

    #[test]
    fn piece_on_e1_is_white_king() {
        let board = Board::startpos();
        assert_eq!(board.piece_on(4), Some((Color::White, Piece::King)));
    }

    #[test]
    fn piece_on_d8_is_black_queen() {
        let board = Board::startpos();
        assert_eq!(board.piece_on(59), Some((Color::Black, Piece::Queen)));
    }

    #[test]
    fn piece_on_e8_is_black_king() {
        let board = Board::startpos();
        assert_eq!(board.piece_on(60), Some((Color::Black, Piece::King)));
    }

    #[test]
    fn piece_on_e2_is_white_pawn() {
        let board = Board::startpos();
        assert_eq!(board.piece_on(12), Some((Color::White, Piece::Pawn)));
    }

    #[test]
    fn piece_on_empty_square_returns_none() {
        let board = Board::startpos();
        // e4 (square 28) is empty at start
        assert_eq!(board.piece_on(28), None);
    }

    #[test]
    fn piece_on_empty_board_returns_none() {
        let board = Board::new_empty();
        assert_eq!(board.piece_on(0), None);
        assert_eq!(board.piece_on(63), None);
    }

    // ── king_square ────────────────────────────────────────────

    #[test]
    fn king_square_white_startpos() {
        let board = Board::startpos();
        assert_eq!(board.king_square(Color::White), 4); // e1
    }

    #[test]
    fn king_square_black_startpos() {
        let board = Board::startpos();
        assert_eq!(board.king_square(Color::Black), 60); // e8
    }

    // ── has_castled ────────────────────────────────────────────

    #[test]
    fn has_castled_false_at_startpos() {
        let board = Board::startpos();
        assert!(!board.has_castled(Color::White));
        assert!(!board.has_castled(Color::Black));
    }

    #[test]
    fn has_castled_white_kingside() {
        // FEN with white king on g1 (castled kingside)
        let board = Board::from_fen("r1bqkbnr/pppppppp/2n5/8/8/5N2/PPPPPPPP/RNBQ1RK1 b kq - 3 2");
        assert!(board.has_castled(Color::White));
    }

    #[test]
    fn has_castled_white_queenside() {
        // FEN with white king on c1 (castled queenside)
        let board = Board::from_fen("r1bqkbnr/pppppppp/2n5/8/8/2N5/PPPPPPPP/2KR1BNR b kq - 3 2");
        assert!(board.has_castled(Color::White));
    }

    #[test]
    fn has_castled_black_kingside() {
        // FEN with black king on g8 (castled kingside)
        let board = Board::from_fen("rnbq1rk1/pppppppp/5n2/8/8/5N2/PPPPPPPP/RNBQKB1R w KQ - 4 3");
        assert!(board.has_castled(Color::Black));
    }

    // ── count_pieces ───────────────────────────────────────────

    #[test]
    fn count_pieces_startpos() {
        let board = Board::startpos();
        assert_eq!(board.count_pieces(Color::White, Piece::Pawn), 8);
        assert_eq!(board.count_pieces(Color::White, Piece::Knight), 2);
        assert_eq!(board.count_pieces(Color::White, Piece::Bishop), 2);
        assert_eq!(board.count_pieces(Color::White, Piece::Rook), 2);
        assert_eq!(board.count_pieces(Color::White, Piece::Queen), 1);
        assert_eq!(board.count_pieces(Color::White, Piece::King), 1);

        assert_eq!(board.count_pieces(Color::Black, Piece::Pawn), 8);
        assert_eq!(board.count_pieces(Color::Black, Piece::Knight), 2);
        assert_eq!(board.count_pieces(Color::Black, Piece::Bishop), 2);
        assert_eq!(board.count_pieces(Color::Black, Piece::Rook), 2);
        assert_eq!(board.count_pieces(Color::Black, Piece::Queen), 1);
        assert_eq!(board.count_pieces(Color::Black, Piece::King), 1);
    }

    #[test]
    fn count_pieces_empty_board() {
        let board = Board::new_empty();
        assert_eq!(board.count_pieces(Color::White, Piece::Pawn), 0);
        assert_eq!(board.count_pieces(Color::Black, Piece::King), 0);
    }

    #[test]
    fn count_pieces_custom_fen() {
        // Position with fewer pieces (endgame-like)
        let board = Board::from_fen("8/8/4k3/8/8/4K3/8/8 w - - 0 1");
        assert_eq!(board.count_pieces(Color::White, Piece::King), 1);
        assert_eq!(board.count_pieces(Color::Black, Piece::King), 1);
        assert_eq!(board.count_pieces(Color::White, Piece::Pawn), 0);
        assert_eq!(board.count_pieces(Color::White, Piece::Queen), 0);
    }
}
