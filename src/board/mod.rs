#[allow(clippy::module_inception)]
mod board;
mod board2;
pub mod castling;
pub mod chess_move;
pub mod color;
pub mod move_generator;
pub mod move_generator2;
pub mod piece;

pub use board::Board;
pub use board2::Board2;
pub use castling::CastlingRights;
pub use chess_move::{ChessMove, ChessMoveState};
pub use color::Color;
pub use move_generator::MoveGenerator;
pub use move_generator2::MoveGenerator2;
pub use piece::Piece;
