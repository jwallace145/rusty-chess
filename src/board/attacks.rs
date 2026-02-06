use super::Board;
use crate::{
    attacks::database::ATTACKS_DB,
    board::{Color, Piece},
};

impl Board {
    // Attack generation

    #[inline]
    pub fn attacks_from(&self, piece: Piece, sq: u8, color: Color) -> u64 {
        let sq: usize = sq as usize;
        let db = &*ATTACKS_DB;

        match piece {
            Piece::Pawn => db.pawn_attacks(sq, color),
            Piece::Knight => db.knight_attacks(sq, self.occ_all),
            Piece::Bishop => db.bishop_attacks(sq, self.occ_all),
            Piece::Rook => db.rook_attacks(sq, self.occ_all),
            Piece::Queen => db.queen_attacks(sq, self.occ_all),
            Piece::King => db.king_attacks(sq, self.occ_all),
        }
    }

    /// Returns a bitboard of all squares attacked by the given color
    pub fn attacks(&self, color: Color) -> u64 {
        let mut attacks: u64 = 0u64;

        // Iterate through all piece types
        for piece in [
            Piece::Pawn,
            Piece::Knight,
            Piece::Bishop,
            Piece::Rook,
            Piece::Queen,
            Piece::King,
        ] {
            let mut pieces: u64 = self.pieces_of(color, piece);

            // Iterate through all pieces of this type
            while pieces != 0 {
                let sq: u8 = pieces.trailing_zeros() as u8;
                attacks |= self.attacks_from(piece, sq, color);
                pieces &= pieces - 1; // Clear the least significant bit
            }
        }

        attacks
    }

    /// Returns true if the given square `sq` is attacked by the given color `by`.
    pub fn is_square_attacked(&self, sq: u8, by: Color) -> bool {
        self.attackers_to(sq, by) != 0
    }

    /// Returns a bitboard of all pieces of color `by` that attack the square `sq`.
    ///
    /// In other words, "All the white pieces that attack f7" (returned as a u64 bitboard).
    pub fn attackers_to(&self, sq: u8, by: Color) -> u64 {
        let mut attackers = 0u64;

        // Pawns - use opponent's attack pattern since pawns attack in opposite directions
        attackers |=
            self.pieces_of(by, Piece::Pawn) & self.attacks_from(Piece::Pawn, sq, by.opponent());

        // Knights
        attackers |= self.pieces_of(by, Piece::Knight) & self.attacks_from(Piece::Knight, sq, by);

        // Bishops
        attackers |= self.pieces_of(by, Piece::Bishop) & self.attacks_from(Piece::Bishop, sq, by);

        // Rooks
        attackers |= self.pieces_of(by, Piece::Rook) & self.attacks_from(Piece::Rook, sq, by);

        // Queens (attack like both bishop and rook)
        attackers |= self.pieces_of(by, Piece::Queen) & self.attacks_from(Piece::Queen, sq, by);

        // King
        attackers |= self.pieces_of(by, Piece::King) & self.attacks_from(Piece::King, sq, by);

        attackers
    }

    /// Returns true if the given color `color` is in check
    pub fn in_check(&self, color: Color) -> bool {
        let king_sq: u8 = self.king_sq[color as usize];
        self.is_square_attacked(king_sq, color.opponent())
    }
}
