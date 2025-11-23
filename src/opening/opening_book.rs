use crate::board::Board2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct OpeningBook {
    positions: HashMap<u64, Vec<(String, u32)>>,
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

    pub fn add_move(&mut self, hash: u64, move_str: String) {
        let entry = self.positions.entry(hash).or_default();

        if let Some((_, count)) = entry.iter_mut().find(|(m, _)| m == &move_str) {
            *count += 1;
        } else {
            entry.push((move_str, 1));
        }
    }

    pub fn finalize(&mut self) {
        // Sort moves by frequency (most common first)
        for moves in self.positions.values_mut() {
            moves.sort_by(|a, b| b.1.cmp(&a.1));
        }
    }

    pub fn probe(&self, hash: u64) -> Option<&str> {
        self.positions
            .get(&hash)
            .and_then(|moves| moves.first())
            .map(|(m, _)| m.as_str())
    }

    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let encoded = bincode::serialize(self).unwrap();
        std::fs::write(path, encoded)
    }

    pub fn load(path: &str) -> std::io::Result<Self> {
        let data = std::fs::read(path)?;
        Ok(bincode::deserialize(&data).unwrap())
    }
}

// Build book manually from common openings
pub fn create_basic_book() -> OpeningBook {
    let mut book = OpeningBook::new();
    let mut board = Board2::new_standard();

    // Helper to add a move sequence
    let add_line = |book: &mut OpeningBook, board: &mut Board2, moves: &[&str]| {
        *board = Board2::new_standard();
        for move_uci in moves {
            let hash = board.hash;
            book.add_move(hash, move_uci.to_string());
            let m = board.parse_uci(move_uci).unwrap();
            board.apply_move(m);
        }
    };

    // ========================================
    // WHITE OPENINGS
    // ========================================

    // Main line: 1.e4
    add_line(&mut book, &mut board, &["e2e4"]);

    // Main line: 1.d4
    add_line(&mut book, &mut board, &["d2d4"]);

    // ========================================
    // AGAINST 1.e4 (AS BLACK)
    // ========================================

    // Italian Game (Giuoco Piano)
    add_line(
        &mut book,
        &mut board,
        &[
            "e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "f8c5", "c2c3", "g8f6", "d2d4", "e5d4", "c3d4",
        ],
    );

    // Ruy Lopez - Main Line (Closed)
    add_line(
        &mut book,
        &mut board,
        &[
            "e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "a7a6", "b5a4", "g8f6", "e1g1", "f8e7", "f1e1",
            "b7b5", "a4b3", "d7d6", "c2c3", "e8g8",
        ],
    );

    // Ruy Lopez - Berlin Defense
    add_line(
        &mut book,
        &mut board,
        &[
            "e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "g8f6", "e1g1", "f6e4", "d2d4", "e4d6", "b5c6",
            "d7c6", "d4e5", "d6f5",
        ],
    );

    // Sicilian Defense - Open Sicilian (Najdorf)
    add_line(
        &mut book,
        &mut board,
        &[
            "e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "a7a6", "c1e3",
            "e7e5", "d4b3",
        ],
    );

    // Sicilian Defense - Dragon Variation
    add_line(
        &mut book,
        &mut board,
        &[
            "e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "g7g6", "c1e3",
            "f8g7", "f2f3", "e8g8",
        ],
    );

    // French Defense - Advance Variation
    add_line(
        &mut book,
        &mut board,
        &[
            "e2e4", "e7e6", "d2d4", "d7d5", "e4e5", "c7c5", "c2c3", "b8c6", "g1f3", "d8b6", "a2a3",
        ],
    );

    // French Defense - Classical
    add_line(
        &mut book,
        &mut board,
        &[
            "e2e4", "e7e6", "d2d4", "d7d5", "b1c3", "g8f6", "c1g5", "f8e7", "e4e5", "f6d7", "g5e7",
            "d8e7",
        ],
    );

    // Caro-Kann Defense - Classical
    add_line(
        &mut book,
        &mut board,
        &[
            "e2e4", "c7c6", "d2d4", "d7d5", "b1c3", "d5e4", "c3e4", "c8f5", "e4g3", "f5g6", "h2h4",
            "h7h6",
        ],
    );

    // Caro-Kann Defense - Advance
    add_line(
        &mut book,
        &mut board,
        &[
            "e2e4", "c7c6", "d2d4", "d7d5", "e4e5", "c8f5", "g1f3", "e7e6", "f1e2", "c6c5", "e1g1",
        ],
    );

    // Scandinavian Defense
    add_line(
        &mut book,
        &mut board,
        &[
            "e2e4", "d7d5", "e4d5", "d8d5", "b1c3", "d5a5", "d2d4", "g8f6", "g1f3", "c7c6",
        ],
    );

    // Pirc Defense
    add_line(
        &mut book,
        &mut board,
        &[
            "e2e4", "d7d6", "d2d4", "g8f6", "b1c3", "g7g6", "g1f3", "f8g7", "f1e2", "e8g8", "e1g1",
        ],
    );

    // Modern Defense - Main Line
    add_line(
        &mut book,
        &mut board,
        &[
            "e2e4", "g7g6", "d2d4", "f8g7", "b1c3", "d7d6", "c1e3", "g8f6", "d1d2", "e8g8",
        ],
    );

    // Modern Defense - Solid setup
    add_line(
        &mut book,
        &mut board,
        &[
            "e2e4", "g7g6", "d2d4", "f8g7", "b1c3", "d7d6", "g1f3", "g8f6", "f1e2", "e8g8",
        ],
    );

    // ========================================
    // AGAINST 1.d4 (AS BLACK)
    // ========================================

    // Queen's Gambit Declined - Orthodox
    add_line(
        &mut book,
        &mut board,
        &[
            "d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "g8f6", "c1g5", "f8e7", "e2e3", "e8g8", "g1f3",
            "b8d7", "a1c1",
        ],
    );

    // Queen's Gambit Accepted
    add_line(
        &mut book,
        &mut board,
        &[
            "d2d4", "d7d5", "c2c4", "d5c4", "g1f3", "g8f6", "e2e3", "e7e6", "f1c4", "c7c5", "e1g1",
            "a7a6",
        ],
    );

    // Slav Defense
    add_line(
        &mut book,
        &mut board,
        &[
            "d2d4", "d7d5", "c2c4", "c7c6", "g1f3", "g8f6", "b1c3", "d5c4", "a2a4", "c8f5", "e2e3",
        ],
    );

    // King's Indian Defense - Classical
    add_line(
        &mut book,
        &mut board,
        &[
            "d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "f8g7", "e2e4", "d7d6", "g1f3", "e8g8", "f1e2",
            "e7e5", "e1g1",
        ],
    );

    // Nimzo-Indian Defense - Classical
    add_line(
        &mut book,
        &mut board,
        &[
            "d2d4", "g8f6", "c2c4", "e7e6", "b1c3", "f8b4", "d1c2", "e8g8", "a2a3", "b4c3", "c2c3",
        ],
    );

    // Queen's Indian Defense
    add_line(
        &mut book,
        &mut board,
        &[
            "d2d4", "g8f6", "c2c4", "e7e6", "g1f3", "b7b6", "g2g3", "c8a6", "b2b3", "f8b4", "c1d2",
            "b4e7",
        ],
    );

    // Grunfeld Defense
    add_line(
        &mut book,
        &mut board,
        &[
            "d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "d7d5", "c4d5", "f6d5", "e2e4", "d5c3", "b2c3",
            "f8g7",
        ],
    );

    // ========================================
    // ALTERNATIVE WHITE OPENINGS
    // ========================================

    // English Opening - Symmetrical
    add_line(
        &mut book,
        &mut board,
        &[
            "c2c4", "c7c5", "b1c3", "b8c6", "g2g3", "g7g6", "f1g2", "f8g7", "g1f3",
        ],
    );

    // English Opening - vs 1...e5
    add_line(
        &mut book,
        &mut board,
        &[
            "c2c4", "e7e5", "b1c3", "g8f6", "g1f3", "b8c6", "g2g3", "d7d5", "c4d5", "f6d5",
        ],
    );

    // Reti Opening
    add_line(
        &mut book,
        &mut board,
        &[
            "g1f3", "d7d5", "c2c4", "e7e6", "g2g3", "g8f6", "f1g2", "f8e7", "e1g1", "e8g8",
        ],
    );

    // London System
    add_line(
        &mut book,
        &mut board,
        &[
            "d2d4", "g8f6", "g1f3", "e7e6", "c1f4", "c7c5", "e2e3", "b8c6", "b1d2", "d7d5", "c2c3",
        ],
    );

    book.finalize();
    book
}
