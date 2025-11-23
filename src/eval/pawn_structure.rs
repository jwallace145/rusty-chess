use crate::{
    board::{Board2, Color, Piece},
    eval::evaluator::BoardEvaluator,
};

pub struct PawnStructureEvaluator;

impl BoardEvaluator for PawnStructureEvaluator {
    fn evaluate(&self, board: &Board2) -> i32 {
        let mut score = 0;

        // Build pawn maps for fast lookups
        let (white_pawns, black_pawns) = Self::build_pawn_maps(board);

        // Evaluate White pawns
        for position in &white_pawns {
            score += Self::evaluate_pawn(*position, Color::White, &white_pawns, &black_pawns);
        }

        // Evaluate Black pawns
        for position in &black_pawns {
            score -= Self::evaluate_pawn(*position, Color::Black, &black_pawns, &white_pawns);
        }

        score
    }
}

impl PawnStructureEvaluator {
    fn evaluate_pawn(
        position: usize,
        color: Color,
        friendly_pawns: &[usize],
        enemy_pawns: &[usize],
    ) -> i32 {
        let mut score: i32 = 0;

        let rank: usize = Self::rank(position);
        let file: usize = Self::file(position);

        // Check for passed pawn
        if Self::is_passed_pawn(position, color, enemy_pawns) {
            score += Self::passed_pawn_bonus(rank, color);
        }

        // Check for isolated pawn
        if Self::is_isolated_pawn(file, friendly_pawns) {
            score -= 20;
        }

        // Check for doubled pawn
        if Self::is_doubled_pawn(position, color, friendly_pawns) {
            score -= 10;
        }

        score
    }

    fn build_pawn_maps(board: &Board2) -> (Vec<usize>, Vec<usize>) {
        let mut white_pawns: Vec<usize> = Vec::new();
        let mut black_pawns: Vec<usize> = Vec::new();

        // Iterate through white pawns bitboard
        let mut white_bb = board.pieces[Color::White as usize][Piece::Pawn as usize];
        while white_bb != 0 {
            let sq = white_bb.trailing_zeros() as usize;
            white_pawns.push(sq);
            white_bb &= white_bb - 1; // Clear the least significant bit
        }

        // Iterate through black pawns bitboard
        let mut black_bb = board.pieces[Color::Black as usize][Piece::Pawn as usize];
        while black_bb != 0 {
            let sq = black_bb.trailing_zeros() as usize;
            black_pawns.push(sq);
            black_bb &= black_bb - 1; // Clear the least significant bit
        }

        (white_pawns, black_pawns)
    }

    fn is_passed_pawn(position: usize, color: Color, enemy_pawns: &[usize]) -> bool {
        let rank: usize = Self::rank(position);
        let file: usize = Self::file(position);

        // Files to check: same file and adjacent files
        let files_to_check = [file.saturating_sub(1), file, (file + 1).min(7)];

        for &enemy_pos in enemy_pawns {
            let enemy_rank: usize = Self::rank(enemy_pos);
            let enemy_file: usize = Self::file(enemy_pos);

            // Check if enemy pawn is on a relevant file
            if !files_to_check.contains(&enemy_file) {
                continue;
            }

            // Check if enemy pawn is ahead of our pawn
            match color {
                Color::White => {
                    // White pawns move up (increasing rank)
                    if enemy_rank > rank {
                        return false; // Enemy pawn blocks this one
                    }
                }
                Color::Black => {
                    // Black pawns move down (decreasing rank)
                    if enemy_rank < rank {
                        return false; // Enemy pawn blocks this one
                    }
                }
            }
        }

        true
    }

    fn passed_pawn_bonus(rank: usize, color: Color) -> i32 {
        // Bonus increases dramatically as pawn advances
        // Rank is 0-7, with 0=rank1, 7=rank8
        let advancement = match color {
            Color::White => rank,     // Rank 0-7, want high rank
            Color::Black => 7 - rank, // Rank 7-0, want low rank
        };

        // Exponential bonus: 2nd rank = 10, 7th rank = 160
        match advancement {
            0 => 0,  // Starting rank (shouldn't happen)
            1 => 10, // Advanced one square
            2 => 20,
            3 => 40,
            4 => 60,
            5 => 90,
            6 => 160, // Nearly promoting!
            _ => 0,
        }
    }

    fn is_isolated_pawn(file: usize, friendly_pawns: &[usize]) -> bool {
        for &friendly_pos in friendly_pawns {
            let friendly_file: usize = Self::file(friendly_pos);

            // Check if on adjacent file (not same file)
            if friendly_file != file && friendly_file.abs_diff(file) == 1 {
                return false; // Has a neighboring pawn
            }
        }

        true
    }

    fn is_doubled_pawn(position: usize, color: Color, friendly_pawns: &[usize]) -> bool {
        let rank: usize = Self::rank(position);
        let file: usize = Self::file(position);

        for &friendly_pos in friendly_pawns {
            if friendly_pos == position {
                continue; // Skip self
            }

            let friendly_file: usize = Self::file(friendly_pos);
            let friendly_rank: usize = Self::rank(friendly_pos);

            // Check if another friendly pawn is on the same file
            if friendly_file == file {
                // Check if it's "behind" this pawn (doubled pawns are a weakness)
                match color {
                    Color::White => {
                        if friendly_rank < rank {
                            return true;
                        }
                    }
                    Color::Black => {
                        if friendly_rank > rank {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    fn rank(position: usize) -> usize {
        position / 8
    }

    fn file(position: usize) -> usize {
        position % 8
    }
}
