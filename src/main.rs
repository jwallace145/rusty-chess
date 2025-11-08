mod board;
use board::{Board, ChessMove};

fn main() {
    let mut board = Board::new();
    println!("Initial board:");
    board.print();

    println!("Side to move: {:?}\n", board.side_to_move);

    // Generate initial moves
    let moves = board.generate_moves();
    println!("Possible moves for {:?}:", board.side_to_move);
    for m in &moves {
        println!("{}", format_move(m));
    }
    println!("Total moves: {}\n", moves.len());

    // Apply a few moves to demonstrate
    let move_sequence = [
        ChessMove {
            from: 12,
            to: 28,
            capture: false,
        }, // e2 -> e4
        ChessMove {
            from: 52,
            to: 36,
            capture: false,
        }, // e7 -> e5
    ];

    for mv in move_sequence {
        println!("Applying move: {}", format_move(&mv));
        board.apply_move(mv);
        board.print();
        println!("Side to move: {:?}\n", board.side_to_move);

        let next_moves = board.generate_moves();
        println!(
            "Possible moves for {:?}: {}",
            board.side_to_move,
            next_moves.len()
        );
    }
}

fn format_move(m: &ChessMove) -> String {
    format!("{} -> {}", index_to_coord(m.from), index_to_coord(m.to))
}

fn index_to_coord(index: usize) -> String {
    let file = (index % 8) as u8;
    let rank = (index / 8) as u8;
    let file_char = (b'a' + file) as char;
    let rank_char = (b'1' + rank) as char;
    format!("{}{}", file_char, rank_char)
}
