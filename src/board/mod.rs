#[allow(clippy::module_inception)]
mod board;
mod board2;
pub mod castling;
pub mod chess_move;
pub mod color;
pub mod move_generator;
pub mod piece;

pub use board::Board;
pub use castling::CastlingRights;
pub use chess_move::{ChessMove, ChessMoveState};
pub use color::Color;
pub use move_generator::MoveGenerator;
pub use piece::Piece;
