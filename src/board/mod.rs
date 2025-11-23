#[allow(clippy::module_inception)]
pub mod board2;
pub mod castling;
pub mod chess_move;
pub mod color;
pub mod piece;

pub use board2::Board2;
pub use castling::CastlingRights;
pub use chess_move::{ChessMove, ChessMoveState};
pub use color::Color;
pub use piece::Piece;
