use super::Board;
use crate::board::{Color, Piece};

impl Board {
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

    // King helpers

    pub fn king_square(&self, color: Color) -> u8 {
        self.king_sq[color as usize]
    }

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

    // Piece counts

    pub fn count_pieces(&self, color: Color, piece: Piece) -> u32 {
        self.pieces[color as usize][piece as usize].count_ones()
    }
}
