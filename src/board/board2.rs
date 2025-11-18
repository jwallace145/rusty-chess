use super::{CastlingRights, Color, Piece};

pub struct Board2 {
    // Pieces (represented as 64-bit bitboards)
    // One for each type of piece and color (12 total)
    pub pieces: [[u64; 6]; 2], // [color][piece]

    // Occupancy (represented as 64-bit bitboards)
    pub occ: [u64; 2], // [color]
    pub occ_all: u64,  // Occupancy regardless of color

    // Game state
    pub side_to_move: Color,
    pub castling: CastlingRights,
    pub en_passant: u8, // 0-63 or 64 if none
    pub halfmove_clock: u8,

    // Kings (represented as 64-bit bitboards)
    // One for each color (2 total)
    pub king_sq: [u8; 2],

    // Hash (represented as 64-bit unsigned integer)
    // The hash of the board state
    pub hash: u64,
}

impl Default for Board2 {
    fn default() -> Self {
        Self::new_standard()
    }
}

#[allow(dead_code)]
impl Board2 {
    /// Creates a new, empty board state
    pub fn new_empty() -> Self {
        Self {
            pieces: [[0u64; 6]; 2],            // No pieces
            occ: [0u64; 2],                    // No occupancy
            occ_all: 0u64,                     // No squares occupied
            side_to_move: Color::White,        // Default to white to move
            castling: CastlingRights::empty(), // No castling rights
            en_passant: 64,                    // 64 = no en passant square
            halfmove_clock: 0,                 // Halfmove clock at 0
            king_sq: [64; 2],                  // No kings on board
            hash: 0u64,                        // Initial hash 0
        }
    }

    pub fn new_standard() -> Self {
        let mut board = Self::new_empty(); // start from empty

        // White pieces
        board.pieces[Color::White as usize][Piece::Pawn as usize] = 0x0000_0000_0000_ff00;
        board.pieces[Color::White as usize][Piece::Rook as usize] = 0x0000_0000_0000_0081;
        board.pieces[Color::White as usize][Piece::Knight as usize] = 0x0000_0000_0000_0042;
        board.pieces[Color::White as usize][Piece::Bishop as usize] = 0x0000_0000_0000_0024;
        board.pieces[Color::White as usize][Piece::Queen as usize] = 0x0000_0000_0000_0010;
        board.pieces[Color::White as usize][Piece::King as usize] = 0x0000_0000_0000_0008;

        // Black pieces
        board.pieces[Color::Black as usize][Piece::Pawn as usize] = 0x00ff_0000_0000_0000;
        board.pieces[Color::Black as usize][Piece::Rook as usize] = 0x8100_0000_0000_0000;
        board.pieces[Color::Black as usize][Piece::Knight as usize] = 0x4200_0000_0000_0000;
        board.pieces[Color::Black as usize][Piece::Bishop as usize] = 0x2400_0000_0000_0000;
        board.pieces[Color::Black as usize][Piece::Queen as usize] = 0x1000_0000_0000_0000;
        board.pieces[Color::Black as usize][Piece::King as usize] = 0x0800_0000_0000_0000;

        // Occupancy
        board.occ[Color::White as usize] =
            board.pieces[Color::White as usize].iter().copied().sum();
        board.occ[Color::Black as usize] =
            board.pieces[Color::Black as usize].iter().copied().sum();
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

        // Kings' positions
        board.king_sq[Color::White as usize] = 4; // e1
        board.king_sq[Color::Black as usize] = 60; // e8

        // Castling rights (all allowed at start)
        board.castling = CastlingRights::full();

        // Side to move
        board.side_to_move = Color::White;

        // No en passant
        board.en_passant = 64;

        // Halfmove clock
        board.halfmove_clock = 0;

        board.hash = 0; // You can initialize with Zobrist hash later

        board
    }

    // Piece and occupancy accessors

    #[inline]
    pub fn pieces_of(&self, color: Color, piece: Piece) -> u64 {
        self.pieces[color as usize][piece as usize]
    }

    #[inline]
    pub fn occupancy(&self, color: Color) -> u64 {
        self.occ[color as usize]
    }

    #[inline]
    pub fn occupied(&self) -> u64 {
        self.occ_all
    }

    #[inline]
    pub fn empty(&self) -> u64 {
        !self.occ_all
    }

    // Square and piece queries

    pub fn piece_on(&self, sq: u8) -> Option<(Color, Piece)> {
        let mask = 1u64 << sq;

        if self.occ_all & mask == 0 {
            return None;
        }

        let color = if self.occ[Color::White as usize] & mask != 0 {
            Color::White
        } else {
            Color::Black
        };
        let c = color as usize;

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

        // Should be unreachable unless the board is corrupt
        None
    }

    // Attack generation
    // pub fn attacks_from(&self, piece: Piece, sq: u8, color: Color) -> u64;
    // pub fn is_square_attacked(&self, sq: u8, by: Color) -> bool;
    // pub fn in_check(&self, color: Color) -> bool;

    // Move generation
    // pub fn generate_moves(&self, list: &mut MoveList);
    // pub fn generate_legal_moves(&self, list: &mut MoveList);
    // pub fn generate_captures(&self, list: &mut MoveList);

    // State operations
    // pub fn make_move(&mut self, mv: Move) -> Undo;
    // pub fn unmake_move(&mut self, mv: Move, undo: Undo);

    // King helpers
    // pub fn king_square(&self, color: Color) -> u8;

    // Piece counts
    // pub fn count_pieces(&self, color: Color, piece: Piece) -> u32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_refactor_pieces_of() {
        let board: Board2 = Board2::default();

        // Get White pieces
        let white_pawns: u64 = board.pieces_of(Color::White, Piece::Pawn);
        let white_rooks: u64 = board.pieces_of(Color::White, Piece::Rook);
        let white_knights: u64 = board.pieces_of(Color::White, Piece::Knight);
        let white_bishops: u64 = board.pieces_of(Color::White, Piece::Bishop);
        let white_queen: u64 = board.pieces_of(Color::White, Piece::Queen);
        let white_king: u64 = board.pieces_of(Color::White, Piece::King);

        // Assert White starting position
        assert_eq!(white_pawns, 0x0000_0000_0000_ff00);
        assert_eq!(white_rooks, 0x0000_0000_0000_0081);
        assert_eq!(white_knights, 0x0000_0000_0000_0042);
        assert_eq!(white_bishops, 0x0000_0000_0000_0024);
        assert_eq!(white_queen, 0x0000_0000_0000_0010);
        assert_eq!(white_king, 0x0000_0000_0000_0008);

        // Get Black pieces
        let black_pawns: u64 = board.pieces_of(Color::Black, Piece::Pawn);
        let black_rooks: u64 = board.pieces_of(Color::Black, Piece::Rook);
        let black_knights: u64 = board.pieces_of(Color::Black, Piece::Knight);
        let black_bishops: u64 = board.pieces_of(Color::Black, Piece::Bishop);
        let black_queen: u64 = board.pieces_of(Color::Black, Piece::Queen);
        let black_king: u64 = board.pieces_of(Color::Black, Piece::King);

        // Assert Black starting position
        assert_eq!(black_pawns, 0x00ff_0000_0000_0000);
        assert_eq!(black_rooks, 0x8100_0000_0000_0000);
        assert_eq!(black_knights, 0x4200_0000_0000_0000);
        assert_eq!(black_bishops, 0x2400_0000_0000_0000);
        assert_eq!(black_queen, 0x1000_0000_0000_0000);
        assert_eq!(black_king, 0x0800_0000_0000_0000);
    }

    #[test]
    fn test_board_refactor_occupancy() {
        let board: Board2 = Board2::default();

        // Assert White pieces occupied squares
        let white_pieces: u64 = board.occupancy(Color::White);
        assert_eq!(white_pieces, 0x0000_0000_0000_ffff);

        // Assert Black pieces occupied squares
        let black_pieces: u64 = board.occupancy(Color::Black);
        assert_eq!(black_pieces, 0xffff_0000_0000_0000);
    }

    #[test]
    fn test_board_refactor_occupied() {
        let board: Board2 = Board2::default();

        // Assert starting pieces occupied squares
        let pieces: u64 = board.occupied();
        assert_eq!(pieces, 0xffff_0000_0000_ffff);
    }

    #[test]
    fn test_board_refactor_empty() {
        let board: Board2 = Board2::default();

        // Assert starting position empty squares
        let empty: u64 = board.empty();
        assert_eq!(empty, 0x0000_ffff_ffff_0000);
    }

    #[test]
    fn test_board_refactor_piece_on() {
        let board: Board2 = Board2::default();

        // Assert White pieces
        // White rooks on a1 (0) and h1 (7)
        assert_eq!(board.piece_on(0), Some((Color::White, Piece::Rook)));
        assert_eq!(board.piece_on(7), Some((Color::White, Piece::Rook)));

        // White knights on b1 (1) and g1 (6)
        assert_eq!(board.piece_on(1), Some((Color::White, Piece::Knight)));
        assert_eq!(board.piece_on(6), Some((Color::White, Piece::Knight)));

        // White bishops on c1 (2) and f1 (5)
        assert_eq!(board.piece_on(2), Some((Color::White, Piece::Bishop)));
        assert_eq!(board.piece_on(5), Some((Color::White, Piece::Bishop)));

        // White queen on d1 (3) and king on e1 (4)
        assert_eq!(board.piece_on(3), Some((Color::White, Piece::King)));
        assert_eq!(board.piece_on(4), Some((Color::White, Piece::Queen)));

        // White pawns on rank 2 (squares 8-15)
        for sq in 8..16 {
            assert_eq!(board.piece_on(sq), Some((Color::White, Piece::Pawn)));
        }

        // Assert Black pieces
        // Black rooks on a8 (56) and h8 (63)
        assert_eq!(board.piece_on(56), Some((Color::Black, Piece::Rook)));
        assert_eq!(board.piece_on(63), Some((Color::Black, Piece::Rook)));

        // Black knights on b8 (57) and g8 (62)
        assert_eq!(board.piece_on(57), Some((Color::Black, Piece::Knight)));
        assert_eq!(board.piece_on(62), Some((Color::Black, Piece::Knight)));

        // Black bishops on c8 (58) and f8 (61)
        assert_eq!(board.piece_on(58), Some((Color::Black, Piece::Bishop)));
        assert_eq!(board.piece_on(61), Some((Color::Black, Piece::Bishop)));

        // Black queen on d8 (59) and king on e8 (60)
        assert_eq!(board.piece_on(59), Some((Color::Black, Piece::King)));
        assert_eq!(board.piece_on(60), Some((Color::Black, Piece::Queen)));

        // Black pawns on rank 7 (squares 48-55)
        for sq in 48..56 {
            assert_eq!(board.piece_on(sq), Some((Color::Black, Piece::Pawn)));
        }

        // Assert empty squares (middle of the board)
        for sq in 16..48 {
            assert_eq!(board.piece_on(sq), None);
        }
    }
}
