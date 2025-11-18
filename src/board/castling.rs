use super::Color;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Side {
    KingSide,
    QueenSide,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CastlingRights(u8);

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
    pub fn has(&self, color: Color, side: Side) -> bool {
        let flag = match (color, side) {
            (Color::White, Side::KingSide) => Self::WHITE_KING_SIDE,
            (Color::White, Side::QueenSide) => Self::WHITE_QUEEN_SIDE,
            (Color::Black, Side::KingSide) => Self::BLACK_KING_SIDE,
            (Color::Black, Side::QueenSide) => Self::BLACK_QUEEN_SIDE,
        };
        (self.0 & flag) != 0
    }

    /// Remove a specific castling right
    pub fn remove(&mut self, color: Color, side: Side) {
        let flag = match (color, side) {
            (Color::White, Side::KingSide) => Self::WHITE_KING_SIDE,
            (Color::White, Side::QueenSide) => Self::WHITE_QUEEN_SIDE,
            (Color::Black, Side::KingSide) => Self::BLACK_KING_SIDE,
            (Color::Black, Side::QueenSide) => Self::BLACK_QUEEN_SIDE,
        };
        self.0 &= !flag;
    }

    /// Add a specific castling right
    pub fn add(&mut self, color: Color, side: Side) {
        let flag = match (color, side) {
            (Color::White, Side::KingSide) => Self::WHITE_KING_SIDE,
            (Color::White, Side::QueenSide) => Self::WHITE_QUEEN_SIDE,
            (Color::Black, Side::KingSide) => Self::BLACK_KING_SIDE,
            (Color::Black, Side::QueenSide) => Self::BLACK_QUEEN_SIDE,
        };
        self.0 |= flag;
    }
}
