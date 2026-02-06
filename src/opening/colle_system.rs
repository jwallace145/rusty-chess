use crate::board::ChessMove;
use crate::opening::{
    B8, C6, D2, D3, D4, D5, D6, D7, E2, E3, E6, E7, F1, F3, F6, G1, G6, G7, G8, OpeningBook,
    create_opening_book_from_lines, mv,
};

// =============================
// London System Opening (White)
// =============================

const COLLE_LINES: &[&[ChessMove]] = &[
    // ====================================
    // 1. d4 d5 2. Nf3 Nc6 3. e3 e6
    // ====================================
    &[
        mv(D2, D4),
        mv(D7, D5),
        mv(G1, F3),
        mv(B8, C6),
        mv(E2, E3),
        mv(E7, E6),
    ],
    // ====================================
    // 1. d4 Nf6 2. Nf3 g6 3. e3 d6 4. Bd3
    // ====================================
    &[
        mv(D2, D4),
        mv(G8, F6),
        mv(G1, F3),
        mv(G7, G6),
        mv(E2, E3),
        mv(D7, D6),
        mv(F1, D3),
    ],
];

pub fn create_colle_system_opening_book() -> OpeningBook {
    create_opening_book_from_lines(COLLE_LINES)
}
