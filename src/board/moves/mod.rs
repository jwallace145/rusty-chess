mod make_move;

use super::{CastlingRights, Piece};
use serde::{Deserialize, Serialize};
use std::fmt;

// ── Bit layout ──────────────────────────────────────────────────────
//
//   15 14 | 13 12 | 11 10  9  8  7  6 |  5  4  3  2  1  0
//   promo | flags |       to (6)       |      from (6)
//
//   flags:  00 = Normal,  01 = Castle,  10 = EnPassant,  11 = Promotion
//   promo:  00 = Knight,  01 = Bishop,  10 = Rook,       11 = Queen
//           (only meaningful when flags == 11)

const FROM_MASK: u16 = 0b0000_0000_0011_1111;
const TO_MASK: u16 = 0b0000_1111_1100_0000;
const FLAG_MASK: u16 = 0b0011_0000_0000_0000;
const PROMO_MASK: u16 = 0b1100_0000_0000_0000;

const TO_SHIFT: u16 = 6;
const FLAG_SHIFT: u16 = 12;
const PROMO_SHIFT: u16 = 14;

const FLAG_NORMAL: u16 = 0b00;
const FLAG_CASTLE: u16 = 0b01;
const FLAG_EN_PASSANT: u16 = 0b10;
const FLAG_PROMOTION: u16 = 0b11;

const PROMO_KNIGHT: u16 = 0b00;
const PROMO_BISHOP: u16 = 0b01;
const PROMO_ROOK: u16 = 0b10;
const PROMO_QUEEN: u16 = 0b11;

// ── ChessMove ───────────────────────────────────────────────────────

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct ChessMove(u16);

impl ChessMove {
    // ── Constructors ────────────────────────────────────────────────

    /// Normal (quiet or capture) move.
    #[inline(always)]
    pub const fn new(from: usize, to: usize) -> Self {
        Self(
            (from as u16 & FROM_MASK)
                | ((to as u16) << TO_SHIFT & TO_MASK)
                | (FLAG_NORMAL << FLAG_SHIFT),
        )
    }

    /// Castling move.
    #[inline(always)]
    pub const fn new_castle(from: usize, to: usize) -> Self {
        Self(
            (from as u16 & FROM_MASK)
                | ((to as u16) << TO_SHIFT & TO_MASK)
                | (FLAG_CASTLE << FLAG_SHIFT),
        )
    }

    /// En passant capture.
    #[inline(always)]
    pub const fn new_en_passant(from: usize, to: usize) -> Self {
        Self(
            (from as u16 & FROM_MASK)
                | ((to as u16) << TO_SHIFT & TO_MASK)
                | (FLAG_EN_PASSANT << FLAG_SHIFT),
        )
    }

    /// Promotion move (may or may not be a capture — caller decides via board state).
    #[inline(always)]
    pub const fn new_promotion(from: usize, to: usize, piece: Piece) -> Self {
        let promo = match piece {
            Piece::Knight => PROMO_KNIGHT,
            Piece::Bishop => PROMO_BISHOP,
            Piece::Rook => PROMO_ROOK,
            Piece::Queen => PROMO_QUEEN,
            // Only promotable pieces are valid; King/Pawn are unreachable.
            _ => PROMO_QUEEN,
        };
        Self(
            (from as u16 & FROM_MASK)
                | ((to as u16) << TO_SHIFT & TO_MASK)
                | (FLAG_PROMOTION << FLAG_SHIFT)
                | (promo << PROMO_SHIFT),
        )
    }

    // ── Accessors ───────────────────────────────────────────────────

    /// Source square (0-63).
    #[inline(always)]
    pub const fn from(self) -> usize {
        (self.0 & FROM_MASK) as usize
    }

    /// Destination square (0-63).
    #[inline(always)]
    pub const fn to(self) -> usize {
        ((self.0 & TO_MASK) >> TO_SHIFT) as usize
    }

    /// Raw 2-bit flag field.
    #[inline(always)]
    const fn flags(self) -> u16 {
        (self.0 & FLAG_MASK) >> FLAG_SHIFT
    }

    #[inline(always)]
    pub const fn is_castle(self) -> bool {
        self.flags() == FLAG_CASTLE
    }

    #[inline(always)]
    pub const fn is_en_passant(self) -> bool {
        self.flags() == FLAG_EN_PASSANT
    }

    #[inline(always)]
    pub const fn is_promotion(self) -> bool {
        self.flags() == FLAG_PROMOTION
    }

    /// True when the move is Normal (not castle, en passant, or promotion).
    #[inline(always)]
    pub const fn is_quiet(self) -> bool {
        self.flags() == FLAG_NORMAL
    }

    /// The promotion piece, or `None` if this is not a promotion.
    #[inline(always)]
    pub const fn promotion_piece(self) -> Option<Piece> {
        if !self.is_promotion() {
            return None;
        }
        let bits = (self.0 & PROMO_MASK) >> PROMO_SHIFT;
        Some(match bits {
            PROMO_KNIGHT => Piece::Knight,
            PROMO_BISHOP => Piece::Bishop,
            PROMO_ROOK => Piece::Rook,
            _ => Piece::Queen,
        })
    }

    /// Access the raw underlying `u16` (useful for serialisation).
    #[inline(always)]
    pub const fn raw(self) -> u16 {
        self.0
    }

    /// Construct from a raw `u16` (useful for deserialisation).
    #[inline(always)]
    pub const fn from_raw(raw: u16) -> Self {
        Self(raw)
    }

    // ── Display helpers ─────────────────────────────────────────────

    /// UCI notation (e.g. "e2e4", "e7e8q").
    pub fn to_uci(&self) -> String {
        let from_file = (self.from() % 8) as u8;
        let from_rank = (self.from() / 8) as u8;
        let to_file = (self.to() % 8) as u8;
        let to_rank = (self.to() / 8) as u8;

        let mut uci = format!(
            "{}{}{}{}",
            (b'a' + from_file) as char,
            (b'1' + from_rank) as char,
            (b'a' + to_file) as char,
            (b'1' + to_rank) as char,
        );

        if let Some(piece) = self.promotion_piece() {
            let ch = match piece {
                Piece::Queen => 'q',
                Piece::Rook => 'r',
                Piece::Bishop => 'b',
                Piece::Knight => 'n',
                _ => unreachable!(),
            };
            uci.push(ch);
        }

        uci
    }

    /// Standard algebraic castling notation ("O-O" or "O-O-O"), or `None`.
    pub fn to_castling_notation(&self) -> Option<&'static str> {
        if !self.is_castle() {
            return None;
        }
        let to_file = self.to() % 8;
        match to_file {
            6 => Some("O-O"),
            2 => Some("O-O-O"),
            _ => None,
        }
    }

    /// Display-friendly string: castling notation when applicable, UCI otherwise.
    pub fn to_display(&self) -> String {
        if let Some(castling) = self.to_castling_notation() {
            castling.to_string()
        } else {
            self.to_uci()
        }
    }
}

// ── Debug ───────────────────────────────────────────────────────────

impl fmt::Debug for ChessMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ChessMove({})", self.to_uci())
    }
}

// ── Serde ───────────────────────────────────────────────────────────

impl Serialize for ChessMove {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ChessMove {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        u16::deserialize(deserializer).map(Self)
    }
}

// ── MoveUndo ────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
pub struct MoveUndo {
    pub chess_move: ChessMove,
    pub captured_piece: Option<Piece>,
    pub previous_castling: CastlingRights,
    pub previous_en_passant: u8,
    pub previous_halfmove_clock: u8,
    pub previous_zobrist_hash: u64,
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Encoding round-trips ────────────────────────────────────────

    #[test]
    fn normal_move_round_trip() {
        let mv = ChessMove::new(12, 28); // e2 → e4
        assert_eq!(mv.from(), 12);
        assert_eq!(mv.to(), 28);
        assert!(mv.is_quiet());
        assert!(!mv.is_castle());
        assert!(!mv.is_en_passant());
        assert!(!mv.is_promotion());
        assert_eq!(mv.promotion_piece(), None);
    }

    #[test]
    fn castle_move_round_trip() {
        let mv = ChessMove::new_castle(4, 6); // e1 → g1 (white kingside)
        assert_eq!(mv.from(), 4);
        assert_eq!(mv.to(), 6);
        assert!(mv.is_castle());
        assert!(!mv.is_quiet());
    }

    #[test]
    fn en_passant_round_trip() {
        let mv = ChessMove::new_en_passant(33, 40); // b5 → a6
        assert_eq!(mv.from(), 33);
        assert_eq!(mv.to(), 40);
        assert!(mv.is_en_passant());
        assert!(!mv.is_quiet());
    }

    #[test]
    fn promotion_queen_round_trip() {
        let mv = ChessMove::new_promotion(48, 56, Piece::Queen); // a7 → a8
        assert_eq!(mv.from(), 48);
        assert_eq!(mv.to(), 56);
        assert!(mv.is_promotion());
        assert_eq!(mv.promotion_piece(), Some(Piece::Queen));
    }

    #[test]
    fn promotion_knight_round_trip() {
        let mv = ChessMove::new_promotion(52, 60, Piece::Knight); // e7 → e8
        assert_eq!(mv.from(), 52);
        assert_eq!(mv.to(), 60);
        assert!(mv.is_promotion());
        assert_eq!(mv.promotion_piece(), Some(Piece::Knight));
    }

    #[test]
    fn promotion_bishop_round_trip() {
        let mv = ChessMove::new_promotion(48, 57, Piece::Bishop); // a7 → b8
        assert!(mv.is_promotion());
        assert_eq!(mv.promotion_piece(), Some(Piece::Bishop));
    }

    #[test]
    fn promotion_rook_round_trip() {
        let mv = ChessMove::new_promotion(55, 63, Piece::Rook); // h7 → h8
        assert!(mv.is_promotion());
        assert_eq!(mv.promotion_piece(), Some(Piece::Rook));
    }

    // ── Edge cases ──────────────────────────────────────────────────

    #[test]
    fn corner_to_corner() {
        let mv = ChessMove::new(0, 63); // a1 → h8
        assert_eq!(mv.from(), 0);
        assert_eq!(mv.to(), 63);
    }

    #[test]
    fn same_square() {
        let mv = ChessMove::new(0, 0);
        assert_eq!(mv.from(), 0);
        assert_eq!(mv.to(), 0);
    }

    // ── UCI notation ────────────────────────────────────────────────

    #[test]
    fn uci_normal_move() {
        let mv = ChessMove::new(12, 28); // e2 → e4
        assert_eq!(mv.to_uci(), "e2e4");
    }

    #[test]
    fn uci_promotion() {
        let mv = ChessMove::new_promotion(52, 60, Piece::Queen); // e7 → e8
        assert_eq!(mv.to_uci(), "e7e8q");
    }

    #[test]
    fn uci_knight_promotion() {
        let mv = ChessMove::new_promotion(48, 56, Piece::Knight); // a7 → a8
        assert_eq!(mv.to_uci(), "a7a8n");
    }

    // ── Castling notation ───────────────────────────────────────────

    #[test]
    fn castling_notation_kingside() {
        let mv = ChessMove::new_castle(4, 6); // e1 → g1
        assert_eq!(mv.to_castling_notation(), Some("O-O"));
    }

    #[test]
    fn castling_notation_queenside() {
        let mv = ChessMove::new_castle(4, 2); // e1 → c1
        assert_eq!(mv.to_castling_notation(), Some("O-O-O"));
    }

    #[test]
    fn castling_notation_non_castle() {
        let mv = ChessMove::new(4, 5);
        assert_eq!(mv.to_castling_notation(), None);
    }

    // ── Display ─────────────────────────────────────────────────────

    #[test]
    fn display_castle_uses_notation() {
        let mv = ChessMove::new_castle(4, 6);
        assert_eq!(mv.to_display(), "O-O");
    }

    #[test]
    fn display_normal_uses_uci() {
        let mv = ChessMove::new(12, 28);
        assert_eq!(mv.to_display(), "e2e4");
    }

    // ── Debug ───────────────────────────────────────────────────────

    #[test]
    fn debug_shows_uci() {
        let mv = ChessMove::new(12, 28);
        assert_eq!(format!("{:?}", mv), "ChessMove(e2e4)");
    }

    // ── Serde round-trip ────────────────────────────────────────────

    #[test]
    fn serde_round_trip() {
        let mv = ChessMove::new_promotion(52, 60, Piece::Queen);
        let bytes = bincode::serialize(&mv).unwrap();
        assert_eq!(bytes.len(), 2); // u16 = 2 bytes
        let decoded: ChessMove = bincode::deserialize(&bytes).unwrap();
        assert_eq!(decoded, mv);
    }

    // ── Equality ────────────────────────────────────────────────────

    #[test]
    fn different_flags_not_equal() {
        let normal = ChessMove::new(4, 6);
        let castle = ChessMove::new_castle(4, 6);
        assert_ne!(normal, castle);
    }

    // ── Size assertions ─────────────────────────────────────────────

    #[test]
    fn chess_move_is_two_bytes() {
        assert_eq!(std::mem::size_of::<ChessMove>(), 2);
    }

    #[test]
    fn move_undo_is_compact() {
        assert!(std::mem::size_of::<MoveUndo>() <= 16);
    }
}
