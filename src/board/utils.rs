use crate::board::{Board, Color, Piece};

/// Print the Chess board to the console
pub fn print_board(board: &Board) {
    // Unicode chess symbols
    fn unicode_symbol(piece: Piece, color: Color) -> char {
        match (piece, color) {
            (Piece::Pawn, Color::White) => '♙',
            (Piece::Knight, Color::White) => '♘',
            (Piece::Bishop, Color::White) => '♗',
            (Piece::Rook, Color::White) => '♖',
            (Piece::Queen, Color::White) => '♕',
            (Piece::King, Color::White) => '♔',
            (Piece::Pawn, Color::Black) => '♟',
            (Piece::Knight, Color::Black) => '♞',
            (Piece::Bishop, Color::Black) => '♝',
            (Piece::Rook, Color::Black) => '♜',
            (Piece::Queen, Color::Black) => '♛',
            (Piece::King, Color::Black) => '♚',
        }
    }

    // ANSI color codes for board squares
    const LIGHT_SQUARE: &str = "\x1b[48;5;230m"; // beige
    const DARK_SQUARE: &str = "\x1b[48;5;94m"; // brown
    const RESET: &str = "\x1b[0m";

    println!("\n  a b c d e f g h");
    for rank in (0..8).rev() {
        print!("{} ", rank + 1);
        for file in 0..8 {
            let sq = rank * 8 + file;
            let is_light = (rank + file) % 2 == 1;
            let bg_color = if is_light { LIGHT_SQUARE } else { DARK_SQUARE };

            if let Some((color, piece)) = board.piece_on(sq) {
                let symbol = unicode_symbol(piece, color);
                print!("{}{} {}", bg_color, symbol, RESET);
            } else {
                print!("{}  {}", bg_color, RESET);
            }
        }
        println!(" {}", rank + 1);
    }
    println!("  a b c d e f g h\n");
}
