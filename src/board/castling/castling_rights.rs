use crate::board::Color;

use super::CastlingRights;
use super::CastlingSide;

impl CastlingRights {
    /// Individual bit flags
    const WHITE_KING_SIDE: u8 = 0b0001;
    const WHITE_QUEEN_SIDE: u8 = 0b0010;
    const BLACK_KING_SIDE: u8 = 0b0100;
    const BLACK_QUEEN_SIDE: u8 = 0b1000;

    /// No castling rights
    pub const fn empty() -> Self {
        CastlingRights(0)
    }

    /// All castling rights
    pub const fn full() -> Self {
        CastlingRights(0b1111)
    }

    /// Check if a particular color + side is available
    pub fn has(&self, color: Color, side: CastlingSide) -> bool {
        let flag: u8 = match (color, side) {
            (Color::White, CastlingSide::KingSide) => Self::WHITE_KING_SIDE,
            (Color::White, CastlingSide::QueenSide) => Self::WHITE_QUEEN_SIDE,
            (Color::Black, CastlingSide::KingSide) => Self::BLACK_KING_SIDE,
            (Color::Black, CastlingSide::QueenSide) => Self::BLACK_QUEEN_SIDE,
        };
        (self.0 & flag) != 0
    }

    /// Remove a specific castling right
    pub fn remove(&mut self, color: Color, side: CastlingSide) {
        let flag: u8 = match (color, side) {
            (Color::White, CastlingSide::KingSide) => Self::WHITE_KING_SIDE,
            (Color::White, CastlingSide::QueenSide) => Self::WHITE_QUEEN_SIDE,
            (Color::Black, CastlingSide::KingSide) => Self::BLACK_KING_SIDE,
            (Color::Black, CastlingSide::QueenSide) => Self::BLACK_QUEEN_SIDE,
        };
        self.0 &= !flag;
    }

    /// Add a specific castling right
    pub fn add(&mut self, color: Color, side: CastlingSide) {
        let flag: u8 = match (color, side) {
            (Color::White, CastlingSide::KingSide) => Self::WHITE_KING_SIDE,
            (Color::White, CastlingSide::QueenSide) => Self::WHITE_QUEEN_SIDE,
            (Color::Black, CastlingSide::KingSide) => Self::BLACK_KING_SIDE,
            (Color::Black, CastlingSide::QueenSide) => Self::BLACK_QUEEN_SIDE,
        };
        self.0 |= flag;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── empty ────────────────────────────────────────────────────

    #[test]
    fn empty_has_no_rights() {
        let rights: CastlingRights = CastlingRights::empty();
        assert!(!rights.has(Color::White, CastlingSide::KingSide));
        assert!(!rights.has(Color::White, CastlingSide::QueenSide));
        assert!(!rights.has(Color::Black, CastlingSide::KingSide));
        assert!(!rights.has(Color::Black, CastlingSide::QueenSide));
    }

    // ── full ─────────────────────────────────────────────────────

    #[test]
    fn full_has_all_rights() {
        let rights: CastlingRights = CastlingRights::full();
        assert!(rights.has(Color::White, CastlingSide::KingSide));
        assert!(rights.has(Color::White, CastlingSide::QueenSide));
        assert!(rights.has(Color::Black, CastlingSide::KingSide));
        assert!(rights.has(Color::Black, CastlingSide::QueenSide));
    }

    // ── add ──────────────────────────────────────────────────────

    #[test]
    fn add_single_right() {
        let mut rights: CastlingRights = CastlingRights::empty();
        rights.add(Color::White, CastlingSide::KingSide);
        assert!(rights.has(Color::White, CastlingSide::KingSide));
        assert!(!rights.has(Color::White, CastlingSide::QueenSide));
        assert!(!rights.has(Color::Black, CastlingSide::KingSide));
        assert!(!rights.has(Color::Black, CastlingSide::QueenSide));
    }

    #[test]
    fn add_multiple_rights() {
        let mut rights: CastlingRights = CastlingRights::empty();
        rights.add(Color::White, CastlingSide::QueenSide);
        rights.add(Color::Black, CastlingSide::KingSide);
        assert!(!rights.has(Color::White, CastlingSide::KingSide));
        assert!(rights.has(Color::White, CastlingSide::QueenSide));
        assert!(rights.has(Color::Black, CastlingSide::KingSide));
        assert!(!rights.has(Color::Black, CastlingSide::QueenSide));
    }

    #[test]
    fn add_is_idempotent() {
        let mut rights: CastlingRights = CastlingRights::empty();
        rights.add(Color::White, CastlingSide::KingSide);
        rights.add(Color::White, CastlingSide::KingSide);
        assert!(rights.has(Color::White, CastlingSide::KingSide));
        assert_eq!(rights, {
            let mut r = CastlingRights::empty();
            r.add(Color::White, CastlingSide::KingSide);
            r
        });
    }

    // ── remove ───────────────────────────────────────────────────

    #[test]
    fn remove_single_right() {
        let mut rights: CastlingRights = CastlingRights::full();
        rights.remove(Color::White, CastlingSide::KingSide);
        assert!(!rights.has(Color::White, CastlingSide::KingSide));
        assert!(rights.has(Color::White, CastlingSide::QueenSide));
        assert!(rights.has(Color::Black, CastlingSide::KingSide));
        assert!(rights.has(Color::Black, CastlingSide::QueenSide));
    }

    #[test]
    fn remove_all_rights_individually() {
        let mut rights: CastlingRights = CastlingRights::full();
        rights.remove(Color::White, CastlingSide::KingSide);
        rights.remove(Color::White, CastlingSide::QueenSide);
        rights.remove(Color::Black, CastlingSide::KingSide);
        rights.remove(Color::Black, CastlingSide::QueenSide);
        assert_eq!(rights, CastlingRights::empty());
    }

    #[test]
    fn remove_from_empty_is_noop() {
        let mut rights: CastlingRights = CastlingRights::empty();
        rights.remove(Color::White, CastlingSide::KingSide);
        assert_eq!(rights, CastlingRights::empty());
    }

    // ── add + remove round-trip ──────────────────────────────────

    #[test]
    fn add_then_remove_returns_to_empty() {
        let mut rights: CastlingRights = CastlingRights::empty();
        rights.add(Color::Black, CastlingSide::QueenSide);
        rights.remove(Color::Black, CastlingSide::QueenSide);
        assert_eq!(rights, CastlingRights::empty());
    }

    // ── independence ─────────────────────────────────────────────

    #[test]
    fn each_right_is_independent() {
        let mut rights: CastlingRights = CastlingRights::empty();
        rights.add(Color::White, CastlingSide::KingSide);
        rights.add(Color::Black, CastlingSide::QueenSide);
        rights.remove(Color::White, CastlingSide::KingSide);
        assert!(!rights.has(Color::White, CastlingSide::KingSide));
        assert!(rights.has(Color::Black, CastlingSide::QueenSide));
    }
}
