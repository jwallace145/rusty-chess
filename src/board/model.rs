use super::{CastlingRights, Color};

#[derive(Copy, Clone)]
pub struct Board {
    // Pieces (represented as 64-bit Bitboards)
    // One for each type of piece and color (12 total)
    pub pieces: [[u64; 6]; 2], // [color][piece]

    // Occupancy (represented as 64-bit Bitboards)
    pub occ: [u64; 2], // [color]
    pub occ_all: u64,  // Occupancy regardless of color

    // Game state
    pub side_to_move: Color,
    pub castling: CastlingRights,
    pub en_passant: u8, // 0-63 or 64 if none
    pub halfmove_clock: u8,

    // King Positions (represented as 2 8-bit square indexes)
    // One for each color (2 total)
    pub king_sq: [u8; 2],

    // Hash (represented as 64-bit unsigned integer)
    // The hash of the board state
    pub hash: u64,
}
