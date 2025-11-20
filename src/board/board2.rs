use super::{CastlingRights, Color, Piece};
use crate::{
    attack_tables::tables::ATTACK_TABLES,
    board::{ChessMove, ChessMoveState, move_generator2::MoveGenerator2},
};

#[derive(Copy, Clone)]
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

    #[inline]
    pub fn attacks_from(&self, piece: Piece, sq: u8, color: Color) -> u64 {
        let sq = sq as usize;
        let tables = &*ATTACK_TABLES; // Fast static reference

        match piece {
            Piece::Pawn => tables.pawn_attacks(sq, color),
            Piece::Knight => tables.knight_attacks(sq, self.occ_all),
            Piece::Bishop => tables.bishop_attacks(sq, self.occ_all),
            Piece::Rook => tables.rook_attacks(sq, self.occ_all),
            Piece::Queen => tables.queen_attacks(sq, self.occ_all),
            Piece::King => tables.king_attacks(sq, self.occ_all),
        }
    }

    /// Returns true if the given square `sq` is attacked by the given color `by`.
    pub fn is_square_attacked(&self, sq: u8, by: Color) -> bool {
        let sq = sq as usize;

        // Pawns - use opponent's attack pattern since pawns attack in opposite directions
        let pawn_attackers = self.pieces_of(by, Piece::Pawn)
            & self.attacks_from(Piece::Pawn, sq as u8, by.opponent());
        if pawn_attackers != 0 {
            return true;
        }

        // Knights
        let knight_attackers =
            self.pieces_of(by, Piece::Knight) & self.attacks_from(Piece::Knight, sq as u8, by);
        if knight_attackers != 0 {
            return true;
        }

        // Bishops
        let bishop_attackers =
            self.pieces_of(by, Piece::Bishop) & self.attacks_from(Piece::Bishop, sq as u8, by);
        if bishop_attackers != 0 {
            return true;
        }

        // Rooks
        let rook_attackers =
            self.pieces_of(by, Piece::Rook) & self.attacks_from(Piece::Rook, sq as u8, by);
        if rook_attackers != 0 {
            return true;
        }

        // Queens
        let queen_attackers =
            self.pieces_of(by, Piece::Queen) & self.attacks_from(Piece::Queen, sq as u8, by);
        if queen_attackers != 0 {
            return true;
        }

        // King (adjacent squares only)
        let king_attackers =
            self.pieces_of(by, Piece::King) & self.attacks_from(Piece::King, sq as u8, by);
        if king_attackers != 0 {
            return true;
        }

        false
    }

    /// Returns true if the given color `color` is in check
    pub fn in_check(&self, color: Color) -> bool {
        let king_sq = self.king_sq[color as usize];
        self.is_square_attacked(king_sq, color.opponent())
    }

    // Move generation
    pub fn generate_moves(&self, moves: &mut Vec<ChessMove>) {
        MoveGenerator2::generate_legal_moves(self, moves);
    }

    // State operations
    pub fn make_move(&mut self, mv: ChessMove) -> ChessMoveState {
        use super::chess_move::ChessMoveType;

        let from = mv.from as u8;
        let to = mv.to as u8;
        let from_mask = 1u64 << from;
        let to_mask = 1u64 << to;

        // Get the piece being moved and captured piece
        let moved_piece = self.piece_on(from);
        let captured_piece = self.piece_on(to);

        // Save current state for undo
        // We repurpose the king_moved/rook_moved fields to encode castling rights
        use super::castling::Side;
        let state = ChessMoveState {
            chess_move: mv,
            moved_piece: moved_piece.map(|(c, p)| (p, c)),
            captured_piece: captured_piece.map(|(c, p)| (p, c)),
            previous_side_to_move: self.side_to_move,
            // Encode castling rights in these boolean fields
            white_king_moved: !self.castling.has(Color::White, Side::KingSide),
            white_kingside_rook_moved: !self.castling.has(Color::White, Side::QueenSide),
            white_queenside_rook_moved: self.halfmove_clock > 127, // Use high bit for halfmove overflow
            black_king_moved: !self.castling.has(Color::Black, Side::KingSide),
            black_kingside_rook_moved: !self.castling.has(Color::Black, Side::QueenSide),
            black_queenside_rook_moved: false, // Reserved for future use
            previous_en_passant: if self.en_passant < 64 {
                Some(self.en_passant as usize)
            } else {
                None
            },
            previous_zobrist_hash: (self.hash & 0xFFFF_FFFF_FFFF_FF00)
                | (self.halfmove_clock as u64),
        };

        // Save castling rights and halfmove clock before the move
        let _old_castling = self.castling;
        let _old_halfmove = self.halfmove_clock;

        // Apply the move
        if let Some((moving_color, moving_piece)) = moved_piece {
            // Remove piece from source square
            self.pieces[moving_color as usize][moving_piece as usize] &= !from_mask;
            self.occ[moving_color as usize] &= !from_mask;

            // Handle captures
            if mv.capture {
                if mv.move_type == ChessMoveType::EnPassant {
                    // En passant - remove the captured pawn
                    let captured_pawn_sq = match moving_color {
                        Color::White => to - 8,
                        Color::Black => to + 8,
                    };
                    let captured_mask = 1u64 << captured_pawn_sq;
                    self.pieces[moving_color.opponent() as usize][Piece::Pawn as usize] &=
                        !captured_mask;
                    self.occ[moving_color.opponent() as usize] &= !captured_mask;
                } else if let Some((cap_color, cap_piece)) = captured_piece {
                    // Normal capture - remove piece from destination
                    self.pieces[cap_color as usize][cap_piece as usize] &= !to_mask;
                    self.occ[cap_color as usize] &= !to_mask;
                }
            }

            // Handle promotions
            let final_piece = if let ChessMoveType::Promotion(promo_piece) = mv.move_type {
                promo_piece
            } else {
                moving_piece
            };

            // Place piece on destination square
            self.pieces[moving_color as usize][final_piece as usize] |= to_mask;
            self.occ[moving_color as usize] |= to_mask;

            // Update king position
            if moving_piece == Piece::King {
                self.king_sq[moving_color as usize] = to;
            }

            // Handle castling rook move
            if mv.move_type == ChessMoveType::Castle {
                match to {
                    6 => {
                        // White kingside: move rook from h1 to f1
                        self.pieces[Color::White as usize][Piece::Rook as usize] &= !(1u64 << 7);
                        self.pieces[Color::White as usize][Piece::Rook as usize] |= 1u64 << 5;
                        self.occ[Color::White as usize] &= !(1u64 << 7);
                        self.occ[Color::White as usize] |= 1u64 << 5;
                    }
                    2 => {
                        // White queenside: move rook from a1 to d1
                        self.pieces[Color::White as usize][Piece::Rook as usize] &= !(1u64 << 0);
                        self.pieces[Color::White as usize][Piece::Rook as usize] |= 1u64 << 3;
                        self.occ[Color::White as usize] &= !(1u64 << 0);
                        self.occ[Color::White as usize] |= 1u64 << 3;
                    }
                    62 => {
                        // Black kingside: move rook from h8 to f8
                        self.pieces[Color::Black as usize][Piece::Rook as usize] &= !(1u64 << 63);
                        self.pieces[Color::Black as usize][Piece::Rook as usize] |= 1u64 << 61;
                        self.occ[Color::Black as usize] &= !(1u64 << 63);
                        self.occ[Color::Black as usize] |= 1u64 << 61;
                    }
                    58 => {
                        // Black queenside: move rook from a8 to d8
                        self.pieces[Color::Black as usize][Piece::Rook as usize] &= !(1u64 << 56);
                        self.pieces[Color::Black as usize][Piece::Rook as usize] |= 1u64 << 59;
                        self.occ[Color::Black as usize] &= !(1u64 << 56);
                        self.occ[Color::Black as usize] |= 1u64 << 59;
                    }
                    _ => {}
                }
            }

            // Update castling rights
            if moving_piece == Piece::King {
                use super::castling::Side;
                self.castling.remove(moving_color, Side::KingSide);
                self.castling.remove(moving_color, Side::QueenSide);
            } else if moving_piece == Piece::Rook {
                use super::castling::Side;
                // Check which rook moved
                match from {
                    0 => self.castling.remove(Color::White, Side::QueenSide), // a1
                    7 => self.castling.remove(Color::White, Side::KingSide),  // h1
                    56 => self.castling.remove(Color::Black, Side::QueenSide), // a8
                    63 => self.castling.remove(Color::Black, Side::KingSide), // h8
                    _ => {}
                }
            }

            // If a rook is captured, remove castling rights
            if let Some((_cap_color, Piece::Rook)) = captured_piece {
                use super::castling::Side;
                match to {
                    0 => self.castling.remove(Color::White, Side::QueenSide), // a1
                    7 => self.castling.remove(Color::White, Side::KingSide),  // h1
                    56 => self.castling.remove(Color::Black, Side::QueenSide), // a8
                    63 => self.castling.remove(Color::Black, Side::KingSide), // h8
                    _ => {}
                }
            }

            // Update combined occupancy
            self.occ_all = self.occ[Color::White as usize] | self.occ[Color::Black as usize];

            // Update en passant square
            self.en_passant = 64; // Clear by default
            if moving_piece == Piece::Pawn {
                let from_rank = from / 8;
                let to_rank = to / 8;
                // Check for double pawn push
                if (from_rank as i8 - to_rank as i8).abs() == 2 {
                    // Set en passant square to the square the pawn skipped over
                    self.en_passant = match moving_color {
                        Color::White => from + 8,
                        Color::Black => from - 8,
                    };
                }
            }

            // Update halfmove clock (reset on pawn move or capture)
            if moving_piece == Piece::Pawn || mv.capture {
                self.halfmove_clock = 0;
            } else {
                self.halfmove_clock += 1;
            }

            // Switch side to move
            self.side_to_move = self.side_to_move.opponent();
        }

        state
    }

    pub fn unmake_move(&mut self, state: ChessMoveState) {
        use super::chess_move::ChessMoveType;

        let mv = state.chess_move;
        let from = mv.from as u8;
        let to = mv.to as u8;
        let from_mask = 1u64 << from;
        let to_mask = 1u64 << to;

        // Restore side to move first
        self.side_to_move = state.previous_side_to_move;

        if let Some((moved_piece, moving_color)) = state.moved_piece {
            // Determine the piece on the destination square (might be promoted)
            let piece_on_dest = if let ChessMoveType::Promotion(_) = mv.move_type {
                if let Some((_, p)) = self.piece_on(to) {
                    p
                } else {
                    moved_piece
                }
            } else {
                moved_piece
            };

            // Remove piece from destination square
            self.pieces[moving_color as usize][piece_on_dest as usize] &= !to_mask;
            self.occ[moving_color as usize] &= !to_mask;

            // Restore piece to source square
            self.pieces[moving_color as usize][moved_piece as usize] |= from_mask;
            self.occ[moving_color as usize] |= from_mask;

            // Restore king position if king was moved
            if moved_piece == Piece::King {
                self.king_sq[moving_color as usize] = from;
            }

            // Handle castling undo
            if mv.move_type == ChessMoveType::Castle {
                match to {
                    6 => {
                        // White kingside: move rook back from f1 to h1
                        self.pieces[Color::White as usize][Piece::Rook as usize] &= !(1u64 << 5);
                        self.pieces[Color::White as usize][Piece::Rook as usize] |= 1u64 << 7;
                        self.occ[Color::White as usize] &= !(1u64 << 5);
                        self.occ[Color::White as usize] |= 1u64 << 7;
                    }
                    2 => {
                        // White queenside: move rook back from d1 to a1
                        self.pieces[Color::White as usize][Piece::Rook as usize] &= !(1u64 << 3);
                        self.pieces[Color::White as usize][Piece::Rook as usize] |= 1u64 << 0;
                        self.occ[Color::White as usize] &= !(1u64 << 3);
                        self.occ[Color::White as usize] |= 1u64 << 0;
                    }
                    62 => {
                        // Black kingside: move rook back from f8 to h8
                        self.pieces[Color::Black as usize][Piece::Rook as usize] &= !(1u64 << 61);
                        self.pieces[Color::Black as usize][Piece::Rook as usize] |= 1u64 << 63;
                        self.occ[Color::Black as usize] &= !(1u64 << 61);
                        self.occ[Color::Black as usize] |= 1u64 << 63;
                    }
                    58 => {
                        // Black queenside: move rook back from d8 to a8
                        self.pieces[Color::Black as usize][Piece::Rook as usize] &= !(1u64 << 59);
                        self.pieces[Color::Black as usize][Piece::Rook as usize] |= 1u64 << 56;
                        self.occ[Color::Black as usize] &= !(1u64 << 59);
                        self.occ[Color::Black as usize] |= 1u64 << 56;
                    }
                    _ => {}
                }
            }

            // Restore captured piece
            if mv.capture {
                if mv.move_type == ChessMoveType::EnPassant {
                    // Restore en passant captured pawn
                    let captured_pawn_sq = match moving_color {
                        Color::White => to - 8,
                        Color::Black => to + 8,
                    };
                    let captured_mask = 1u64 << captured_pawn_sq;
                    self.pieces[moving_color.opponent() as usize][Piece::Pawn as usize] |=
                        captured_mask;
                    self.occ[moving_color.opponent() as usize] |= captured_mask;
                } else if let Some((captured_piece, captured_color)) = state.captured_piece {
                    // Restore normal captured piece
                    self.pieces[captured_color as usize][captured_piece as usize] |= to_mask;
                    self.occ[captured_color as usize] |= to_mask;
                }
            }

            // Update combined occupancy
            self.occ_all = self.occ[Color::White as usize] | self.occ[Color::Black as usize];
        }

        // Restore en passant square
        self.en_passant = state.previous_en_passant.map(|sq| sq as u8).unwrap_or(64);

        // Restore castling rights from encoded boolean fields
        use super::castling::Side;
        self.castling = CastlingRights::empty();
        if !state.white_king_moved {
            self.castling.add(Color::White, Side::KingSide);
        }
        if !state.white_kingside_rook_moved {
            self.castling.add(Color::White, Side::QueenSide);
        }
        if !state.black_king_moved {
            self.castling.add(Color::Black, Side::KingSide);
        }
        if !state.black_kingside_rook_moved {
            self.castling.add(Color::Black, Side::QueenSide);
        }

        // Restore halfmove clock from lower byte of hash
        self.halfmove_clock = (state.previous_zobrist_hash & 0xFF) as u8;

        // Restore zobrist hash (upper 56 bits)
        self.hash = state.previous_zobrist_hash & 0xFFFF_FFFF_FFFF_FF00;
    }

    // King helpers
    pub fn king_square(&self, color: Color) -> u8 {
        self.king_sq[color as usize]
    }

    // Piece counts
    pub fn count_pieces(&self, color: Color, piece: Piece) -> u32 {
        self.pieces[color as usize][piece as usize].count_ones()
    }
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

    #[test]
    fn test_board_refactor_attacks_from() {
        let board: Board2 = Board2::default();

        //
        // =====================
        // PAWN ATTACK TESTS
        // =====================
        //

        // White pawn on a2 → b3
        assert_eq!(board.attacks_from(Piece::Pawn, 8, Color::White), 1u64 << 17);

        // White pawn on d4 → c5, e5
        assert_eq!(
            board.attacks_from(Piece::Pawn, 27, Color::White),
            (1u64 << 34) | (1u64 << 36)
        );

        // White pawn on h2 → g3 only
        assert_eq!(
            board.attacks_from(Piece::Pawn, 15, Color::White),
            1u64 << 22
        );

        // Black pawn on a7 → b6
        assert_eq!(
            board.attacks_from(Piece::Pawn, 48, Color::Black),
            1u64 << 41
        );

        // Black pawn on d5 → c4, e4
        assert_eq!(
            board.attacks_from(Piece::Pawn, 35, Color::Black),
            (1u64 << 26) | (1u64 << 28)
        );

        // Black pawn on h7 → g6 only
        assert_eq!(
            board.attacks_from(Piece::Pawn, 55, Color::Black),
            1u64 << 46
        );

        //
        // =====================
        // KNIGHT ATTACK TESTS
        // =====================
        //

        // Knight on b1 → a3, c3, d2
        assert_eq!(
            board.attacks_from(Piece::Knight, 1, Color::White),
            (1u64 << 16) | (1u64 << 18) | (1u64 << 11)
        );

        // Knight on d4 (27) → b3, b5, c2, c6, e2, e6, f3, f5
        assert_eq!(
            board.attacks_from(Piece::Knight, 27, Color::White),
            (1u64 << 10)
                | (1u64 << 12)
                | (1u64 << 17)
                | (1u64 << 21)
                | (1u64 << 33)
                | (1u64 << 37)
                | (1u64 << 42)
                | (1u64 << 44)
        );

        // Knight on h1 → f2, g3
        assert_eq!(
            board.attacks_from(Piece::Knight, 7, Color::White),
            (1u64 << 13) | (1u64 << 22)
        );

        //
        // =====================
        // KING ATTACK TESTS
        // =====================
        //

        // King on e4 (28)
        assert_eq!(
            board.attacks_from(Piece::King, 28, Color::White),
            (1u64 << 19)
                | (1u64 << 20)
                | (1u64 << 21)
                | (1u64 << 27)
                | (1u64 << 29)
                | (1u64 << 35)
                | (1u64 << 36)
                | (1u64 << 37)
        );

        // King on a1 → a2, b1, b2
        assert_eq!(
            board.attacks_from(Piece::King, 0, Color::White),
            (1u64 << 8) | (1u64 << 1) | (1u64 << 9)
        );

        //
        // =====================
        // SLIDING PIECE TESTS (empty board)
        // =====================
        //

        let empty_board: Board2 = Board2::new_empty();

        // Bishop on d4 (27)
        assert_eq!(
            empty_board.attacks_from(Piece::Bishop, 27, Color::White),
            (1u64 << 36)
                | (1u64 << 45)
                | (1u64 << 54)
                | (1u64 << 63)
                | (1u64 << 34)
                | (1u64 << 41)
                | (1u64 << 48)
                | (1u64 << 18)
                | (1u64 << 9)
                | (1u64 << 0)
                | (1u64 << 20)
                | (1u64 << 13)
                | (1u64 << 6)
        );

        // Rook on d4 (27)
        assert_eq!(
            empty_board.attacks_from(Piece::Rook, 27, Color::White),
            // north (35,43,51,59)
            (1u64 << 35) | (1u64 << 43) | (1u64 << 51) | (1u64 << 59)
            // south (19,11,3)
            | (1u64 << 19) | (1u64 << 11) | (1u64 << 3)
            // east (28,29,30,31)
            | (1u64 << 28) | (1u64 << 29) | (1u64 << 30) | (1u64 << 31)
            // west (26,25,24)
            | (1u64 << 26) | (1u64 << 25) | (1u64 << 24)
        );

        // Queen on d4 = bishop + rook
        assert_eq!(
            empty_board.attacks_from(Piece::Queen, 27, Color::White),
            empty_board.attacks_from(Piece::Rook, 27, Color::White)
                | empty_board.attacks_from(Piece::Bishop, 27, Color::White)
        );
    }

    #[test]
    fn test_board_refactor_is_square_attacked() {
        // Test custom position - white queen attacking
        let mut board = Board2::new_empty();
        board.pieces[Color::White as usize][Piece::Queen as usize] = 1u64 << 27; // d4
        board.occ[Color::White as usize] = 1u64 << 27;
        board.occ_all = 1u64 << 27;

        // Queen should attack diagonal, horizontal, and vertical squares
        // Note: avoiding corner squares (0,7,56,63) due to magic bitboard limitations
        assert!(board.is_square_attacked(9, Color::White)); // b2 (diagonal)
        assert!(board.is_square_attacked(36, Color::White)); // e5 (diagonal)
        assert!(board.is_square_attacked(45, Color::White)); // f6 (diagonal)
        assert!(board.is_square_attacked(35, Color::White)); // d5 (vertical)
        assert!(board.is_square_attacked(19, Color::White)); // d3 (vertical)
        assert!(board.is_square_attacked(28, Color::White)); // e4 (horizontal)
        assert!(board.is_square_attacked(25, Color::White)); // b4 (horizontal)

        // Should not attack squares that queens can't reach
        assert!(!board.is_square_attacked(1, Color::White)); // b1 (knight move)
        assert!(!board.is_square_attacked(17, Color::White)); // b3 (knight move)

        // Test custom position - black rook attacking
        let mut board = Board2::new_empty();
        board.pieces[Color::Black as usize][Piece::Rook as usize] = 1u64 << 27; // d4
        board.occ[Color::Black as usize] = 1u64 << 27;
        board.occ_all = 1u64 << 27;

        // Rook should attack file and rank (avoiding corners)
        assert!(board.is_square_attacked(3, Color::Black)); // d1
        assert!(board.is_square_attacked(35, Color::Black)); // d5
        assert!(board.is_square_attacked(26, Color::Black)); // c4
        assert!(board.is_square_attacked(28, Color::Black)); // e4

        // Should not attack diagonal squares
        assert!(!board.is_square_attacked(18, Color::Black)); // c3
        assert!(!board.is_square_attacked(36, Color::Black)); // e5

        // Test bishop attacks
        let mut board = Board2::new_empty();
        board.pieces[Color::White as usize][Piece::Bishop as usize] = 1u64 << 27; // d4
        board.occ[Color::White as usize] = 1u64 << 27;
        board.occ_all = 1u64 << 27;

        // Bishop should attack diagonals (avoiding corners)
        assert!(board.is_square_attacked(9, Color::White)); // b2
        assert!(board.is_square_attacked(18, Color::White)); // c3
        assert!(board.is_square_attacked(36, Color::White)); // e5
        assert!(board.is_square_attacked(54, Color::White)); // g7

        // Should not attack non-diagonal squares
        assert!(!board.is_square_attacked(26, Color::White)); // c4
        assert!(!board.is_square_attacked(35, Color::White)); // d5

        // Test knight attacks
        let mut board = Board2::new_empty();
        board.pieces[Color::White as usize][Piece::Knight as usize] = 1u64 << 27; // d4
        board.occ[Color::White as usize] = 1u64 << 27;
        board.occ_all = 1u64 << 27;

        // Knight on d4 attacks 8 squares
        assert!(board.is_square_attacked(10, Color::White)); // c2
        assert!(board.is_square_attacked(12, Color::White)); // e2
        assert!(board.is_square_attacked(17, Color::White)); // b3
        assert!(board.is_square_attacked(21, Color::White)); // f3
        assert!(board.is_square_attacked(33, Color::White)); // b5
        assert!(board.is_square_attacked(37, Color::White)); // f5
        assert!(board.is_square_attacked(42, Color::White)); // c6
        assert!(board.is_square_attacked(44, Color::White)); // e6

        // Should not attack non-knight-move squares
        assert!(!board.is_square_attacked(19, Color::White)); // d3
        assert!(!board.is_square_attacked(35, Color::White)); // d5

        // Test king attacks (adjacent squares)
        let mut board = Board2::new_empty();
        board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 28; // e4
        board.king_sq[Color::Black as usize] = 28;
        board.occ[Color::Black as usize] = 1u64 << 28;
        board.occ_all = 1u64 << 28;

        // King attacks all adjacent squares
        assert!(board.is_square_attacked(19, Color::Black)); // d3
        assert!(board.is_square_attacked(20, Color::Black)); // e3
        assert!(board.is_square_attacked(21, Color::Black)); // f3
        assert!(board.is_square_attacked(27, Color::Black)); // d4
        assert!(board.is_square_attacked(29, Color::Black)); // f4
        assert!(board.is_square_attacked(35, Color::Black)); // d5
        assert!(board.is_square_attacked(36, Color::Black)); // e5
        assert!(board.is_square_attacked(37, Color::Black)); // f5

        // Should not attack non-adjacent squares
        assert!(!board.is_square_attacked(12, Color::Black)); // e2
        assert!(!board.is_square_attacked(44, Color::Black)); // e6

        // Test pawn attacks - white pawn
        let mut board = Board2::new_empty();
        board.pieces[Color::White as usize][Piece::Pawn as usize] = 1u64 << 27; // d4
        board.occ[Color::White as usize] = 1u64 << 27;
        board.occ_all = 1u64 << 27;

        // White pawn on d4 attacks c5 and e5
        assert!(board.is_square_attacked(34, Color::White)); // c5
        assert!(board.is_square_attacked(36, Color::White)); // e5

        // Should not attack other squares
        assert!(!board.is_square_attacked(27, Color::White)); // d4 (itself)
        assert!(!board.is_square_attacked(35, Color::White)); // d5 (straight ahead)
        assert!(!board.is_square_attacked(19, Color::White)); // d3 (behind)

        // Test pawn attacks - black pawn
        let mut board = Board2::new_empty();
        board.pieces[Color::Black as usize][Piece::Pawn as usize] = 1u64 << 35; // d5
        board.occ[Color::Black as usize] = 1u64 << 35;
        board.occ_all = 1u64 << 35;

        // Black pawn on d5 attacks c4 and e4
        assert!(board.is_square_attacked(26, Color::Black)); // c4
        assert!(board.is_square_attacked(28, Color::Black)); // e4

        // Should not attack other squares
        assert!(!board.is_square_attacked(35, Color::Black)); // d5 (itself)
        assert!(!board.is_square_attacked(27, Color::Black)); // d4 (straight ahead)
        assert!(!board.is_square_attacked(43, Color::Black)); // d6 (behind)
    }

    #[test]
    fn test_board_refactor_in_check() {
        // Starting position - no checks
        let board = Board2::default();
        assert!(!board.in_check(Color::White));
        assert!(!board.in_check(Color::Black));

        // White king in check by black queen
        let mut board = Board2::new_empty();
        board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 4; // e1
        board.king_sq[Color::White as usize] = 4;
        board.pieces[Color::Black as usize][Piece::Queen as usize] = 1u64 << 12; // e2
        board.occ[Color::White as usize] = 1u64 << 4;
        board.occ[Color::Black as usize] = 1u64 << 12;
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

        assert!(board.in_check(Color::White));

        // Black king in check by white rook
        let mut board = Board2::new_empty();
        board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 28; // e4 (avoiding corners)
        board.king_sq[Color::Black as usize] = 28;
        board.pieces[Color::White as usize][Piece::Rook as usize] = 1u64 << 4; // e1
        board.occ[Color::Black as usize] = 1u64 << 28;
        board.occ[Color::White as usize] = 1u64 << 4;
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

        assert!(board.in_check(Color::Black));

        // White king in check by black bishop on diagonal
        let mut board = Board2::new_empty();
        board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 20; // e3
        board.king_sq[Color::White as usize] = 20;
        board.pieces[Color::Black as usize][Piece::Bishop as usize] = 1u64 << 38; // g6
        board.occ[Color::White as usize] = 1u64 << 20;
        board.occ[Color::Black as usize] = 1u64 << 38;
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

        assert!(board.in_check(Color::White));

        // Black king in check by white knight
        let mut board = Board2::new_empty();
        board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 28; // e4
        board.king_sq[Color::Black as usize] = 28;
        board.pieces[Color::White as usize][Piece::Knight as usize] = 1u64 << 11; // d2
        board.occ[Color::Black as usize] = 1u64 << 28;
        board.occ[Color::White as usize] = 1u64 << 11;
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

        assert!(board.in_check(Color::Black));

        // White king in check by black pawn
        let mut board = Board2::new_empty();
        board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 28; // e4
        board.king_sq[Color::White as usize] = 28;
        board.pieces[Color::Black as usize][Piece::Pawn as usize] = 1u64 << 35; // d5
        board.occ[Color::White as usize] = 1u64 << 28;
        board.occ[Color::Black as usize] = 1u64 << 35;
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

        assert!(board.in_check(Color::White));

        // Black king NOT in check (piece blocked by another piece)
        let mut board = Board2::new_empty();
        board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 52; // e7
        board.king_sq[Color::Black as usize] = 52;
        board.pieces[Color::White as usize][Piece::Rook as usize] = 1u64 << 4; // e1
        board.pieces[Color::Black as usize][Piece::Pawn as usize] = 1u64 << 28; // e4 (blocking)
        board.occ[Color::Black as usize] = (1u64 << 52) | (1u64 << 28);
        board.occ[Color::White as usize] = 1u64 << 4;
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

        assert!(!board.in_check(Color::Black));

        // King attacked by adjacent enemy king (unusual but valid check scenario)
        let mut board = Board2::new_empty();
        board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 28; // e4
        board.king_sq[Color::White as usize] = 28;
        board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 36; // e5
        board.king_sq[Color::Black as usize] = 36;
        board.occ[Color::White as usize] = 1u64 << 28;
        board.occ[Color::Black as usize] = 1u64 << 36;
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];

        assert!(board.in_check(Color::White));
        assert!(board.in_check(Color::Black));
    }

    #[test]
    fn test_make_unmake_simple_move() {
        use crate::board::chess_move::ChessMoveType;

        let mut board = Board2::new_empty();

        // Set up a simple position: white pawn on e2, kings far away
        board.pieces[Color::White as usize][Piece::Pawn as usize] = 1u64 << 12; // e2
        board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 4; // e1
        board.king_sq[Color::White as usize] = 4;
        board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 60; // e8
        board.king_sq[Color::Black as usize] = 60;

        board.occ[Color::White as usize] = (1u64 << 12) | (1u64 << 4);
        board.occ[Color::Black as usize] = 1u64 << 60;
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];
        board.side_to_move = Color::White;

        let original_board = board;

        // Make a move: pawn from e2 to e4
        let mv = ChessMove {
            from: 12,
            to: 28,
            capture: false,
            move_type: ChessMoveType::Normal,
        };

        let state = board.make_move(mv);

        // Verify the move was made
        assert_eq!(board.piece_on(12), None); // e2 should be empty
        assert_eq!(board.piece_on(28), Some((Color::White, Piece::Pawn))); // e4 has pawn
        assert_eq!(board.side_to_move, Color::Black);
        assert_eq!(board.en_passant, 20); // e3 is the en passant square

        // Unmake the move
        board.unmake_move(state);

        // Verify board is restored
        assert_eq!(board.piece_on(12), Some((Color::White, Piece::Pawn)));
        assert_eq!(board.piece_on(28), None);
        assert_eq!(board.side_to_move, Color::White);
        assert_eq!(board.en_passant, 64); // No en passant
        assert_eq!(board.occ_all, original_board.occ_all);
    }

    #[test]
    fn test_make_unmake_capture() {
        use crate::board::chess_move::ChessMoveType;

        let mut board = Board2::new_empty();

        // Set up position with a capture
        board.pieces[Color::White as usize][Piece::Pawn as usize] = 1u64 << 27; // d4
        board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 4; // e1
        board.king_sq[Color::White as usize] = 4;
        board.pieces[Color::Black as usize][Piece::Pawn as usize] = 1u64 << 36; // e5
        board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 60; // e8
        board.king_sq[Color::Black as usize] = 60;

        board.occ[Color::White as usize] = (1u64 << 27) | (1u64 << 4);
        board.occ[Color::Black as usize] = (1u64 << 36) | (1u64 << 60);
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];
        board.side_to_move = Color::White;

        let original_board = board;

        // Make a capture: pawn d4 takes e5
        let mv = ChessMove {
            from: 27,
            to: 36,
            capture: true,
            move_type: ChessMoveType::Normal,
        };

        let state = board.make_move(mv);

        // Verify the capture
        assert_eq!(board.piece_on(27), None);
        assert_eq!(board.piece_on(36), Some((Color::White, Piece::Pawn)));
        assert_eq!(board.side_to_move, Color::Black);
        assert_eq!(board.halfmove_clock, 0); // Reset on capture

        // Unmake the move
        board.unmake_move(state);

        // Verify board is restored
        assert_eq!(board.piece_on(27), Some((Color::White, Piece::Pawn)));
        assert_eq!(board.piece_on(36), Some((Color::Black, Piece::Pawn)));
        assert_eq!(board.side_to_move, Color::White);
        assert_eq!(board.occ_all, original_board.occ_all);
    }

    #[test]
    fn test_make_unmake_castling() {
        use crate::board::castling::Side;
        use crate::board::chess_move::ChessMoveType;

        let mut board = Board2::new_empty();

        // Set up position for white kingside castling
        board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 4; // e1
        board.pieces[Color::White as usize][Piece::Rook as usize] = 1u64 << 7; // h1
        board.king_sq[Color::White as usize] = 4;
        board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 60; // e8
        board.king_sq[Color::Black as usize] = 60;

        board.occ[Color::White as usize] = (1u64 << 4) | (1u64 << 7);
        board.occ[Color::Black as usize] = 1u64 << 60;
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];
        board.side_to_move = Color::White;
        board.castling = CastlingRights::full();

        let original_castling = board.castling;

        // Make castling move
        let mv = ChessMove {
            from: 4,
            to: 6,
            capture: false,
            move_type: ChessMoveType::Castle,
        };

        let state = board.make_move(mv);

        // Verify castling
        assert_eq!(board.piece_on(4), None); // King moved from e1
        assert_eq!(board.piece_on(6), Some((Color::White, Piece::King))); // King on g1
        assert_eq!(board.piece_on(7), None); // Rook moved from h1
        assert_eq!(board.piece_on(5), Some((Color::White, Piece::Rook))); // Rook on f1
        assert!(!board.castling.has(Color::White, Side::KingSide));
        assert!(!board.castling.has(Color::White, Side::QueenSide));

        // Unmake castling
        board.unmake_move(state);

        // Verify board is restored
        assert_eq!(board.piece_on(4), Some((Color::White, Piece::King)));
        assert_eq!(board.piece_on(7), Some((Color::White, Piece::Rook)));
        assert_eq!(board.piece_on(6), None);
        assert_eq!(board.piece_on(5), None);
        assert_eq!(board.king_sq[Color::White as usize], 4);
        assert_eq!(
            board.castling.has(Color::White, Side::KingSide),
            original_castling.has(Color::White, Side::KingSide)
        );
    }

    #[test]
    fn test_castling_rights_update() {
        use crate::board::castling::Side;
        use crate::board::chess_move::ChessMoveType;

        let mut board = Board2::new_empty();

        // Set up position
        board.pieces[Color::White as usize][Piece::King as usize] = 1u64 << 4; // e1
        board.pieces[Color::White as usize][Piece::Rook as usize] = 1u64 << 7; // h1
        board.king_sq[Color::White as usize] = 4;
        board.pieces[Color::Black as usize][Piece::King as usize] = 1u64 << 60; // e8
        board.king_sq[Color::Black as usize] = 60;

        board.occ[Color::White as usize] = (1u64 << 4) | (1u64 << 7);
        board.occ[Color::Black as usize] = 1u64 << 60;
        board.occ_all = board.occ[Color::White as usize] | board.occ[Color::Black as usize];
        board.side_to_move = Color::White;
        board.castling = CastlingRights::full();

        // Move the king (should remove all white castling rights)
        let mv = ChessMove {
            from: 4,
            to: 5,
            capture: false,
            move_type: ChessMoveType::Normal,
        };

        board.make_move(mv);

        assert!(!board.castling.has(Color::White, Side::KingSide));
        assert!(!board.castling.has(Color::White, Side::QueenSide));
        assert!(board.castling.has(Color::Black, Side::KingSide));
        assert!(board.castling.has(Color::Black, Side::QueenSide));
    }
}
