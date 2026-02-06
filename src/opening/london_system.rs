use crate::board::ChessMove;
use crate::opening::{
    B1, B8, C1, C2, C3, C5, C6, C7, C8, D2, D3, D4, D5, D6, D7, E2, E3, E5, E6, E7, F1, F3, F4, F5,
    F6, F7, F8, G1, G6, G7, G8, H2, H3, OpeningBook, capture, create_opening_book_from_lines, mv,
};

// =============================
// London System Opening (White)
// =============================

const LONDON_LINES: &[&[ChessMove]] = &[
    // ====================================
    // 1. d4 d5 2. Bf4 e5 3. Bxe5 f6 4. Bf4
    // ====================================
    &[
        mv(D2, D4),
        mv(D7, D5),
        mv(C1, F4),
        mv(E7, E5),
        capture(F4, E5),
        mv(F7, F6),
        mv(E5, F4),
    ],
    // Queen's Pawn Opening: Horwitz Defense (Dutch Defense too?)

    // ==============================================
    // 1. d4 f5 2. Bf4 e6 3. Nf3 Nf6 4. e3 Nc6 5. Bd3
    // ==============================================
    &[
        mv(D2, D4),
        mv(F7, F5),
        mv(C1, F4),
        mv(E7, E6),
        mv(G1, F3),
        mv(G8, F6),
        mv(E2, E3),
        mv(B8, C6),
        mv(F1, D3),
    ],
    // ===========================
    // 1. d4 d5 2. Bf4
    // ===========================

    // Main line: 1. d4 d5 2. Bf4 Nf6 3. e3 e6 4. Nf3 Bd6 5. Bd3
    &[
        mv(D2, D4),
        mv(D7, D5),
        mv(C1, F4),
        mv(G8, F6),
        mv(E2, E3),
        mv(E7, E6),
        mv(G1, F3),
        mv(F8, D6),
        mv(F1, D3),
    ],
    // 3. Nf3 first: 1. d4 d5 2. Bf4 Nf6 3. Nf3 e6 4. e3 Bd6 5. Bd3
    &[
        mv(D2, D4),
        mv(D7, D5),
        mv(C1, F4),
        mv(G8, F6),
        mv(G1, F3),
        mv(E7, E6),
        mv(E2, E3),
        mv(F8, D6),
        mv(F1, D3),
    ],
    // Nbd2 setup: 1. d4 d5 2. Bf4 Nf6 3. e3 e6 4. Nf3 Bd6 5. Nbd2
    &[
        mv(D2, D4),
        mv(D7, D5),
        mv(C1, F4),
        mv(G8, F6),
        mv(E2, E3),
        mv(E7, E6),
        mv(G1, F3),
        mv(F8, D6),
        mv(B1, D2),
    ],
    // vs ...c5: 1. d4 d5 2. Bf4 Nf6 3. e3 c5 4. c3 Nc6 5. Nd2
    &[
        mv(D2, D4),
        mv(D7, D5),
        mv(C1, F4),
        mv(G8, F6),
        mv(E2, E3),
        mv(C7, C5),
        mv(C2, C3),
        mv(B8, C6),
        mv(B1, D2),
    ],
    // vs ...c5 direct: 1. d4 d5 2. Bf4 c5 3. e3 Nc6 4. Nf3 Nf6 5. c3
    &[
        mv(D2, D4),
        mv(D7, D5),
        mv(C1, F4),
        mv(C7, C5),
        mv(E2, E3),
        mv(B8, C6),
        mv(G1, F3),
        mv(G8, F6),
        mv(C2, C3),
    ],
    // vs mirror London (...Bf5): 1. d4 d5 2. Bf4 Nf6 3. e3 Bf5 4. Nf3 e6 5. Bd3
    &[
        mv(D2, D4),
        mv(D7, D5),
        mv(C1, F4),
        mv(G8, F6),
        mv(E2, E3),
        mv(C8, F5),
        mv(G1, F3),
        mv(E7, E6),
        mv(F1, D3),
    ],
    // vs Slav setup (...c6): 1. d4 d5 2. Bf4 Nf6 3. e3 c6 4. Nf3 Bf5 5. Bd3
    &[
        mv(D2, D4),
        mv(D7, D5),
        mv(C1, F4),
        mv(G8, F6),
        mv(E2, E3),
        mv(C7, C6),
        mv(G1, F3),
        mv(C8, F5),
        mv(F1, D3),
    ],
    // vs ...e6 early: 1. d4 d5 2. Bf4 e6 3. e3 Nf6 4. Nf3 Bd6 5. Bd3
    &[
        mv(D2, D4),
        mv(D7, D5),
        mv(C1, F4),
        mv(E7, E6),
        mv(E2, E3),
        mv(G8, F6),
        mv(G1, F3),
        mv(F8, D6),
        mv(F1, D3),
    ],
    // vs ...e6 + ...c5: 1. d4 d5 2. Bf4 e6 3. e3 c5 4. c3 Nf6 5. Nf3
    &[
        mv(D2, D4),
        mv(D7, D5),
        mv(C1, F4),
        mv(E7, E6),
        mv(E2, E3),
        mv(C7, C5),
        mv(C2, C3),
        mv(G8, F6),
        mv(G1, F3),
    ],
    // ===========================
    // 1. d4 Nf6 2. Bf4
    // ===========================

    // 1. d4 Nf6 2. Bf4 d5 3. e3 e6 4. Nf3 Bd6 5. Bd3
    &[
        mv(D2, D4),
        mv(G8, F6),
        mv(C1, F4),
        mv(D7, D5),
        mv(E2, E3),
        mv(E7, E6),
        mv(G1, F3),
        mv(F8, D6),
        mv(F1, D3),
    ],
    // 1. d4 Nf6 2. Bf4 d5 3. e3 c5 4. c3 Nc6 5. Nf3
    &[
        mv(D2, D4),
        mv(G8, F6),
        mv(C1, F4),
        mv(D7, D5),
        mv(E2, E3),
        mv(C7, C5),
        mv(C2, C3),
        mv(B8, C6),
        mv(G1, F3),
    ],
    // 1. d4 Nf6 2. Bf4 e6 3. e3 d5 4. Nf3 Bd6 5. Bd3
    &[
        mv(D2, D4),
        mv(G8, F6),
        mv(C1, F4),
        mv(E7, E6),
        mv(E2, E3),
        mv(D7, D5),
        mv(G1, F3),
        mv(F8, D6),
        mv(F1, D3),
    ],
    // 1. d4 Nf6 2. Bf4 e6 3. e3 c5 4. Nf3 d5 5. c3
    &[
        mv(D2, D4),
        mv(G8, F6),
        mv(C1, F4),
        mv(E7, E6),
        mv(E2, E3),
        mv(C7, C5),
        mv(G1, F3),
        mv(D7, D5),
        mv(C2, C3),
    ],
    // vs King's Indian: 1. d4 Nf6 2. Bf4 g6 3. Nf3 Bg7 4. e3 d6 5. h3
    &[
        mv(D2, D4),
        mv(G8, F6),
        mv(C1, F4),
        mv(G7, G6),
        mv(G1, F3),
        mv(F8, G7),
        mv(E2, E3),
        mv(D7, D6),
        mv(H2, H3),
    ],
    // vs KID with ...d5: 1. d4 Nf6 2. Bf4 g6 3. Nf3 Bg7 4. e3 d5 5. Bd3
    &[
        mv(D2, D4),
        mv(G8, F6),
        mv(C1, F4),
        mv(G7, G6),
        mv(G1, F3),
        mv(F8, G7),
        mv(E2, E3),
        mv(D7, D5),
        mv(F1, D3),
    ],
    // ===========================
    // 1. d4 X 2. Nf3 — common transpositions
    // ===========================

    // 1. d4 d5 2. Nf3 Nf6 3. Bf4 e6 4. e3 Bd6 5. Bd3
    &[
        mv(D2, D4),
        mv(D7, D5),
        mv(G1, F3),
        mv(G8, F6),
        mv(C1, F4),
        mv(E7, E6),
        mv(E2, E3),
        mv(F8, D6),
        mv(F1, D3),
    ],
    // 1. d4 d5 2. Nf3 Nf6 3. Bf4 c5 4. e3 Nc6 5. c3
    &[
        mv(D2, D4),
        mv(D7, D5),
        mv(G1, F3),
        mv(G8, F6),
        mv(C1, F4),
        mv(C7, C5),
        mv(E2, E3),
        mv(B8, C6),
        mv(C2, C3),
    ],
    // 1. d4 Nf6 2. Nf3 d5 3. Bf4 e6 4. e3 Bd6 5. Bd3
    &[
        mv(D2, D4),
        mv(G8, F6),
        mv(G1, F3),
        mv(D7, D5),
        mv(C1, F4),
        mv(E7, E6),
        mv(E2, E3),
        mv(F8, D6),
        mv(F1, D3),
    ],
];

pub fn create_london_system_opening_book() -> OpeningBook {
    create_opening_book_from_lines(LONDON_LINES)
}
