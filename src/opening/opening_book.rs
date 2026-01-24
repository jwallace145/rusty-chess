use crate::board::{Board, ChessMove, ChessMoveType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct OpeningBook {
    positions: HashMap<u64, Vec<(ChessMove, u32)>>,
}

impl Default for OpeningBook {
    fn default() -> Self {
        Self::new()
    }
}

impl OpeningBook {
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
        }
    }

    pub fn add_move(&mut self, hash: u64, chess_move: ChessMove) {
        let entry = self.positions.entry(hash).or_default();

        if let Some((_, count)) = entry.iter_mut().find(|(m, _)| *m == chess_move) {
            *count += 1;
        } else {
            entry.push((chess_move, 1));
        }
    }

    pub fn finalize(&mut self) {
        // Sort moves by frequency (most common first)
        for moves in self.positions.values_mut() {
            moves.sort_by(|a, b| b.1.cmp(&a.1));
        }
    }

    pub fn probe(&self, hash: u64) -> Option<ChessMove> {
        self.positions
            .get(&hash)
            .and_then(|moves| moves.first())
            .map(|(m, _)| *m)
    }

    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let encoded = bincode::serialize(self).unwrap();
        std::fs::write(path, encoded)
    }

    pub fn load(path: &str) -> std::io::Result<Self> {
        let data = std::fs::read(path)?;
        bincode::deserialize(&data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

/// Build an opening book featuring the London System for White.
///
/// The London System is characterized by:
/// 1. d4 - Queen's pawn opening
/// 2. Bf4 - Early bishop development (the signature London move)
/// 3. e3 - Supporting the bishop
/// 4. Nf3 - Knight development
/// 5. c3 - Solid pawn structure
/// 6. Bd3 - Bishop to d3
/// 7. Nbd2 - Knight to d2
/// 8. O-O - Castling kingside
pub fn create_basic_book() -> OpeningBook {
    let mut book = OpeningBook::new();

    // Helper to create a normal move
    let mv = |from: usize, to: usize| ChessMove {
        from,
        to,
        capture: false,
        move_type: ChessMoveType::Normal,
    };

    // Helper to create a capture move
    let capture = |from: usize, to: usize| ChessMove {
        from,
        to,
        capture: true,
        move_type: ChessMoveType::Normal,
    };

    // Square indices (rank * 8 + file, where a1 = 0)
    // Files: a=0, b=1, c=2, d=3, e=4, f=5, g=6, h=7
    // Ranks: 1=0, 2=8, 3=16, 4=24, 5=32, 6=40, 7=48, 8=56
    const D2: usize = 11; // d2
    const D4: usize = 27; // d4
    const C1: usize = 2; // c1 (dark-squared bishop)
    const F4: usize = 29; // f4
    const E2: usize = 12; // e2
    const E3: usize = 20; // e3
    const G1: usize = 6; // g1 (knight)
    const F3: usize = 21; // f3
    const F1: usize = 5; // f1 (light-squared bishop)
    const D3: usize = 19; // d3

    // Start from the starting position
    let mut board = Board::startpos();

    // === Move 1: d2d4 (London System starts with d4) ===
    // From starting position, White plays d4
    book.add_move(board.hash, mv(D2, D4));

    // Make the move to get the next position
    board.make_move(mv(D2, D4));

    // === After 1. d4, handle common Black responses ===
    // Black's common responses: d5, Nf6, e6, f5 (Dutch), c5, etc.

    // We need to add White's response (Bf4) for various Black moves
    // Let's handle the most common: 1...d5, 1...Nf6, 1...e6

    // --- 1...d5 (most common) ---
    let mut board_d5 = board;
    let d5_move = mv(51, 35); // d7-d5
    board_d5.make_move(d5_move);
    // After 1. d4 d5, White plays 2. Bf4
    book.add_move(board_d5.hash, mv(C1, F4));

    // Continue the London: 1. d4 d5 2. Bf4
    let mut board_d5_bf4 = board_d5;
    board_d5_bf4.make_move(mv(C1, F4));

    // After 2. Bf4, common Black moves: Nf6, e6, c5, Bf5
    // --- 1. d4 d5 2. Bf4 Nf6 ---
    let mut board_d5_bf4_nf6 = board_d5_bf4;
    board_d5_bf4_nf6.make_move(mv(62, 45)); // Ng8-f6
    book.add_move(board_d5_bf4_nf6.hash, mv(E2, E3)); // 3. e3

    // Continue: 1. d4 d5 2. Bf4 Nf6 3. e3
    let mut board_line1 = board_d5_bf4_nf6;
    board_line1.make_move(mv(E2, E3));
    // After 3. e3, Black plays e6 or c5 typically
    // 3...e6
    let mut board_line1_e6 = board_line1;
    board_line1_e6.make_move(mv(52, 44)); // e7-e6
    book.add_move(board_line1_e6.hash, mv(G1, F3)); // 4. Nf3

    // Continue: 4. Nf3
    let mut board_line1_nf3 = board_line1_e6;
    board_line1_nf3.make_move(mv(G1, F3));
    // 4...Bd6 or Be7
    let mut board_line1_bd6 = board_line1_nf3;
    board_line1_bd6.make_move(mv(61, 43)); // Bf8-d6
    book.add_move(board_line1_bd6.hash, mv(F1, D3)); // 5. Bd3

    // --- 1. d4 d5 2. Bf4 e6 ---
    let mut board_d5_bf4_e6 = board_d5_bf4;
    board_d5_bf4_e6.make_move(mv(52, 44)); // e7-e6
    book.add_move(board_d5_bf4_e6.hash, mv(E2, E3)); // 3. e3

    // --- 1. d4 d5 2. Bf4 c5 ---
    let mut board_d5_bf4_c5 = board_d5_bf4;
    board_d5_bf4_c5.make_move(mv(50, 34)); // c7-c5
    book.add_move(board_d5_bf4_c5.hash, mv(E2, E3)); // 3. e3

    // --- 1. d4 d5 2. Bf4 Bf5 (mirror) ---
    let mut board_d5_bf4_bf5 = board_d5_bf4;
    board_d5_bf4_bf5.make_move(mv(58, 37)); // Bc8-f5
    book.add_move(board_d5_bf4_bf5.hash, mv(E2, E3)); // 3. e3

    // --- 1...Nf6 (Indian Defense setup) ---
    let mut board_nf6 = board;
    board_nf6.make_move(mv(62, 45)); // Ng8-f6
    // After 1. d4 Nf6, White plays 2. Bf4
    book.add_move(board_nf6.hash, mv(C1, F4));

    // Continue: 1. d4 Nf6 2. Bf4
    let mut board_nf6_bf4 = board_nf6;
    board_nf6_bf4.make_move(mv(C1, F4));

    // After 2. Bf4, Black plays d5, e6, g6, etc.
    // --- 1. d4 Nf6 2. Bf4 d5 ---
    let mut board_nf6_bf4_d5 = board_nf6_bf4;
    board_nf6_bf4_d5.make_move(mv(51, 35)); // d7-d5
    book.add_move(board_nf6_bf4_d5.hash, mv(E2, E3)); // 3. e3

    // --- 1. d4 Nf6 2. Bf4 e6 ---
    let mut board_nf6_bf4_e6 = board_nf6_bf4;
    board_nf6_bf4_e6.make_move(mv(52, 44)); // e7-e6
    book.add_move(board_nf6_bf4_e6.hash, mv(E2, E3)); // 3. e3

    // --- 1. d4 Nf6 2. Bf4 g6 (King's Indian style) ---
    let mut board_nf6_bf4_g6 = board_nf6_bf4;
    board_nf6_bf4_g6.make_move(mv(54, 46)); // g7-g6
    book.add_move(board_nf6_bf4_g6.hash, mv(G1, F3)); // 3. Nf3

    // --- 1...e6 (can transpose to QGD or French-like) ---
    let mut board_e6 = board;
    board_e6.make_move(mv(52, 44)); // e7-e6
    book.add_move(board_e6.hash, mv(C1, F4)); // 2. Bf4 (London style)

    // --- 1...f5 (Dutch Defense) - still play Bf4! ---
    let mut board_f5 = board;
    board_f5.make_move(mv(53, 37)); // f7-f5
    book.add_move(board_f5.hash, mv(C1, F4)); // 2. Bf4

    // --- 1...c5 (Benoni-like) ---
    let mut board_c5 = board;
    board_c5.make_move(mv(50, 34)); // c7-c5
    book.add_move(board_c5.hash, mv(E2, E3)); // 2. e3 (more solid, or could play d5)

    // --- 1...g6 (Modern Defense) ---
    let mut board_g6 = board;
    board_g6.make_move(mv(54, 46)); // g7-g6
    book.add_move(board_g6.hash, mv(C1, F4)); // 2. Bf4

    // Add a few more depth in the main line
    // Main line: 1. d4 d5 2. Bf4 Nf6 3. e3 e6 4. Nf3 Bd6 5. Bd3
    let mut main_line = board_line1_bd6;
    main_line.make_move(mv(F1, D3)); // 5. Bd3

    // After 5. Bd3, if Black plays Bxf4
    let mut main_line_bxf4 = main_line;
    main_line_bxf4.make_move(capture(43, 29)); // Bd6xf4
    book.add_move(main_line_bxf4.hash, capture(E3, F4)); // 6. exf4 (recapture)

    // If Black castles instead
    let mut main_line_oo = main_line;
    // Black castles kingside (e8-g8)
    let castle_move = ChessMove {
        from: 60, // e8
        to: 62,   // g8
        capture: false,
        move_type: ChessMoveType::Castle,
    };
    main_line_oo.make_move(castle_move);
    book.add_move(main_line_oo.hash, mv(G1, F3)); // 6. Nf3 if not already played

    // Finalize the book (sort by frequency)
    book.finalize();

    book
}
