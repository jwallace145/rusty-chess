use crate::board::{Color, Piece};
use std::sync::OnceLock;

static ZOBRIST_TABLE: OnceLock<ZobristTable> = OnceLock::new();

/// A Zobrist hash table for efficiently encoding chess board positions.
///
/// Zobrist hashing enables fast board state comparison and is essential for implementing
/// transposition tables in the chess engine. Each unique board position maps to a 64-bit
/// hash that can be incrementally updated as moves are made and unmade.
///
/// ## How It Works
///
/// The table stores random 64-bit values for every possible board component:
/// - 768 values for pieces (2 colors × 6 piece types × 64 squares)
/// - 1 value for Black to move
/// - 4 values for castling rights (White/Black kingside/queenside)
/// - 8 values for en passant files
///
/// A board's hash is computed by XOR-combining the values for all active components.
/// Since XOR is its own inverse (A ⊕ B ⊕ B = A), the hash can be efficiently updated
/// when making or unmaking moves without recomputing from scratch.
///
/// ## Usage
///
/// The table is initialized once with random values and accessed via [`ZobristTable::get()`].
/// Use [`compute_hash()`] to generate a hash for any board position.
///
/// # Fields
/// - `pieces`: Random values indexed by [color][piece_type][square]
/// - `black_to_move`: Value to XOR when Black is to move
/// - `castling`: Values for the four castling rights
/// - `en_passant`: Values for en passant availability on each file
///
/// # References
/// - [Wikipedia: Zobrist Hashing](https://en.wikipedia.org/wiki/Zobrist_hashing)
pub struct ZobristTable {
    pieces: [[[u64; 64]; 6]; 2], // [color][piece_type][square]
    black_to_move: u64,          // used to maintain current turn
    castling: [u64; 4],          // white kingside, white queenside, black kingside, black queenside
    en_passant: [u64; 8],        // one for each file
}

/// Identifies which castling right
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastlingRight {
    WhiteKingside,
    WhiteQueenside,
    BlackKingside,
    BlackQueenside,
}

impl ZobristTable {
    fn new() -> Self {
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};

        // Use fixed seed for reproducibility
        let mut rng = StdRng::seed_from_u64(0x0123456789ABCDEF);

        // Seed the piece hashes
        let mut pieces = [[[0u64; 64]; 6]; 2];
        for color_pieces in &mut pieces {
            for piece_squares in color_pieces {
                for square_hash in piece_squares {
                    *square_hash = rng.random::<u64>();
                }
            }
        }

        // Seed Black to move hash
        let black_to_move = rng.random::<u64>();

        // Seed castling hashes
        let mut castling = [0u64; 4];
        for item in &mut castling {
            *item = rng.random::<u64>();
        }

        // Seed en passant hashes
        let mut en_passant = [0u64; 8];
        for item in &mut en_passant {
            *item = rng.random::<u64>();
        }

        Self {
            pieces,
            black_to_move,
            castling,
            en_passant,
        }
    }

    pub fn get() -> &'static ZobristTable {
        ZOBRIST_TABLE.get_or_init(ZobristTable::new)
    }

    fn piece_index(piece: Piece) -> usize {
        match piece {
            Piece::Pawn => 0,
            Piece::Knight => 1,
            Piece::Bishop => 2,
            Piece::Rook => 3,
            Piece::Queen => 4,
            Piece::King => 5,
        }
    }

    fn color_index(color: Color) -> usize {
        match color {
            Color::White => 0,
            Color::Black => 1,
        }
    }

    fn castling_index(right: CastlingRight) -> usize {
        match right {
            CastlingRight::WhiteKingside => 0,
            CastlingRight::WhiteQueenside => 1,
            CastlingRight::BlackKingside => 2,
            CastlingRight::BlackQueenside => 3,
        }
    }

    // === Public API for incremental updates ===

    /// Get the hash value for a piece on a square
    #[inline]
    pub fn piece(&self, piece: Piece, color: Color, square: usize) -> u64 {
        self.pieces[Self::color_index(color)][Self::piece_index(piece)][square]
    }

    /// Get the hash value for the side to move
    #[inline]
    pub fn side_to_move(&self) -> u64 {
        self.black_to_move
    }

    /// Get hash value for side to move (only for Black; White is implicit by absence)
    #[inline]
    pub fn castling(&self, right: CastlingRight) -> u64 {
        self.castling[Self::castling_index(right)]
    }

    /// Get hash value for en passant on a file (0-7)
    #[inline]
    pub fn en_passant(&self, file: usize) -> u64 {
        debug_assert!(file < 8, "File must be 0-7");
        self.en_passant[file]
    }
}

/// Compute hash from scratch for a board position.
/// Use this only for initialization or validation.
/// For move updates, use incremental XOR operations.
pub fn compute_hash(board: &crate::board::Board) -> u64 {
    let table = ZobristTable::get();
    let mut hash = 0u64;

    // Hash all pieces
    for (square, sq) in board.squares.iter().enumerate() {
        if let Some((piece, color)) = sq.0 {
            hash ^= table.piece(piece, color, square);
        }
    }

    // Hash castling rights
    if board.can_castle_white_kingside() {
        hash ^= table.castling(CastlingRight::WhiteKingside);
    }
    if board.can_castle_white_queenside() {
        hash ^= table.castling(CastlingRight::WhiteQueenside);
    }
    if board.can_castle_black_kingside() {
        hash ^= table.castling(CastlingRight::BlackKingside);
    }
    if board.can_castle_black_queenside() {
        hash ^= table.castling(CastlingRight::BlackQueenside);
    }

    // Hash side to move
    if board.side_to_move == Color::Black {
        hash ^= table.side_to_move();
    }

    // Hash en passant
    if let Some(ep_square) = board.en_passant_target {
        let file = ep_square % 8;
        hash ^= table.en_passant(file);
    }

    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::chess_move::ChessMoveType;
    use crate::board::{Board, ChessMove};

    #[test]
    fn test_zobrist_table_initialization() {
        let table = ZobristTable::new();

        // Verify all piece hashes are non-zero
        for color in 0..2 {
            for piece in 0..6 {
                for square in 0..64 {
                    assert_ne!(
                        table.pieces[color][piece][square], 0,
                        "Piece hash should be non-zero for color={}, piece={}, square={}",
                        color, piece, square
                    );
                }
            }
        }

        // Verify black_to_move hash is non-zero
        assert_ne!(
            table.black_to_move, 0,
            "Black to move hash should be non-zero"
        );

        // Verify all castling hashes are non-zero
        for i in 0..4 {
            assert_ne!(
                table.castling[i], 0,
                "Castling hash {} should be non-zero",
                i
            );
        }

        // Verify all en passant hashes are non-zero
        for i in 0..8 {
            assert_ne!(
                table.en_passant[i], 0,
                "En passant hash {} should be non-zero",
                i
            );
        }

        // Verify deterministic initialization (same seed produces same values)
        let table2 = ZobristTable::new();
        assert_eq!(
            table.black_to_move, table2.black_to_move,
            "Tables should be deterministic"
        );
        assert_eq!(
            table.pieces[0][0][0], table2.pieces[0][0][0],
            "Piece hashes should be deterministic"
        );

        // Verify some values are different (RNG is working)
        assert_ne!(
            table.pieces[0][0][0], table.pieces[0][0][1],
            "Adjacent piece hashes should be different"
        );
        assert_ne!(
            table.castling[0], table.castling[1],
            "Different castling rights should have different hashes"
        );
    }

    #[test]
    fn test_zobrist_hash_equality() {
        let mut board1 = Board::new();

        // e2 -> e3
        board1.apply_move(ChessMove {
            from: pos("e2"),
            to: pos("e3"),
            capture: false,
            move_type: ChessMoveType::Normal,
        });

        // e7 -> e6
        board1.apply_move(ChessMove {
            from: pos("e7"),
            to: pos("e6"),
            capture: false,
            move_type: ChessMoveType::Normal,
        });

        // e3 -> e4
        board1.apply_move(ChessMove {
            from: pos("e3"),
            to: pos("e4"),
            capture: false,
            move_type: ChessMoveType::Normal,
        });

        // e6 -> e5
        board1.apply_move(ChessMove {
            from: pos("e6"),
            to: pos("e5"),
            capture: false,
            move_type: ChessMoveType::Normal,
        });

        // Nf3 - clear any en passant state
        board1.apply_move(ChessMove {
            from: pos("g1"),
            to: pos("f3"),
            capture: false,
            move_type: ChessMoveType::Normal,
        });

        let mut board2 = Board::new();

        // e2 -> e4 (sets en passant target)
        board2.apply_move(ChessMove {
            from: pos("e2"),
            to: pos("e4"),
            capture: false,
            move_type: ChessMoveType::Normal,
        });

        // e7 -> e5 (sets en passant target)
        board2.apply_move(ChessMove {
            from: pos("e7"),
            to: pos("e5"),
            capture: false,
            move_type: ChessMoveType::Normal,
        });

        // Nf3 - clear the en passant state
        board2.apply_move(ChessMove {
            from: pos("g1"),
            to: pos("f3"),
            capture: false,
            move_type: ChessMoveType::Normal,
        });

        let zobrist_hash_board1 = compute_hash(&board1);
        let zobrist_hash_board2 = compute_hash(&board2);

        assert_eq!(
            zobrist_hash_board1, zobrist_hash_board2,
            "Zobrist hashes should be equivalent for the same position reached via different move orders"
        )
    }

    fn pos(s: &str) -> usize {
        let bytes = s.as_bytes();
        let file = (bytes[0] - b'a') as usize;
        let rank = (bytes[1] - b'1') as usize;
        rank * 8 + file
    }
}
