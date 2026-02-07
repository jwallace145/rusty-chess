mod castling_rights;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CastlingSide {
    KingSide,
    QueenSide,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CastlingRights(u8);
