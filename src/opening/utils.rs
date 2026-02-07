use super::OpeningBook;
use crate::board::{Board, ChessMove};

/// Creates a normal (non-capturing) move from source to destination square.
pub const fn mv(from: usize, to: usize) -> ChessMove {
    ChessMove::new(from, to)
}

/// Creates a capture move from source to destination square.
/// Note: In the packed ChessMove encoding, captures and non-captures
/// share the same constructor. This helper documents capture intent.
pub const fn capture(from: usize, to: usize) -> ChessMove {
    ChessMove::new(from, to)
}

// Square constants for all 64 chess squares
pub const A1: usize = 0;
pub const B1: usize = 1;
pub const C1: usize = 2;
pub const D1: usize = 3;
pub const E1: usize = 4;
pub const F1: usize = 5;
pub const G1: usize = 6;
pub const H1: usize = 7;

pub const A2: usize = 8;
pub const B2: usize = 9;
pub const C2: usize = 10;
pub const D2: usize = 11;
pub const E2: usize = 12;
pub const F2: usize = 13;
pub const G2: usize = 14;
pub const H2: usize = 15;

pub const A3: usize = 16;
pub const B3: usize = 17;
pub const C3: usize = 18;
pub const D3: usize = 19;
pub const E3: usize = 20;
pub const F3: usize = 21;
pub const G3: usize = 22;
pub const H3: usize = 23;

pub const A4: usize = 24;
pub const B4: usize = 25;
pub const C4: usize = 26;
pub const D4: usize = 27;
pub const E4: usize = 28;
pub const F4: usize = 29;
pub const G4: usize = 30;
pub const H4: usize = 31;

pub const A5: usize = 32;
pub const B5: usize = 33;
pub const C5: usize = 34;
pub const D5: usize = 35;
pub const E5: usize = 36;
pub const F5: usize = 37;
pub const G5: usize = 38;
pub const H5: usize = 39;

pub const A6: usize = 40;
pub const B6: usize = 41;
pub const C6: usize = 42;
pub const D6: usize = 43;
pub const E6: usize = 44;
pub const F6: usize = 45;
pub const G6: usize = 46;
pub const H6: usize = 47;

pub const A7: usize = 48;
pub const B7: usize = 49;
pub const C7: usize = 50;
pub const D7: usize = 51;
pub const E7: usize = 52;
pub const F7: usize = 53;
pub const G7: usize = 54;
pub const H7: usize = 55;

pub const A8: usize = 56;
pub const B8: usize = 57;
pub const C8: usize = 58;
pub const D8: usize = 59;
pub const E8: usize = 60;
pub const F8: usize = 61;
pub const G8: usize = 62;
pub const H8: usize = 63;

/// Creates an opening book from a collection of move sequences (lines).
///
/// Each line is a sequence of moves starting from the standard position.
/// Common positions across different lines will naturally accumulate moves,
/// handling transpositions automatically.
///
/// # Example
/// ```
/// use rusty_chess::board::ChessMove;
/// use rusty_chess::opening::{mv, create_opening_book_from_lines};
/// use rusty_chess::opening::{D2, D4, D7, D5, C1, F4, G8, F6};
///
/// let lines: &[&[ChessMove]] = &[
///     &[mv(D2, D4), mv(D7, D5), mv(C1, F4)],  // 1. d4 d5 2. Bf4
///     &[mv(D2, D4), mv(G8, F6), mv(C1, F4)],  // 1. d4 Nf6 2. Bf4
/// ];
/// let book = create_opening_book_from_lines(lines);
/// ```
pub fn create_opening_book_from_lines(lines: &[&[ChessMove]]) -> OpeningBook {
    let mut book = OpeningBook::new();

    for line in lines {
        let mut board = Board::startpos();
        for &chess_move in *line {
            book.add_move(board.hash, chess_move);
            board.make_move(chess_move);
        }
    }

    book.finalize();
    book
}
