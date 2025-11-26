use crate::{
    board::{Board2, Color, Piece},
    eval::evaluator::BoardEvaluator,
};

/// Direction indices for ray masks
const NORTH: usize = 0;
const SOUTH: usize = 1;
const EAST: usize = 2;
const WEST: usize = 3;
const NORTH_EAST: usize = 4;
const NORTH_WEST: usize = 5;
const SOUTH_EAST: usize = 6;
const SOUTH_WEST: usize = 7;

/// Ray masks for each square and direction.
/// RAY_MASKS[sq][dir] gives the bitboard of all squares in direction `dir` from square `sq`.
/// These are "pure" rays not including the source square, used for pin/x-ray detection.
const RAY_MASKS: [[u64; 8]; 64] = generate_ray_masks();

/// Generate all ray masks at compile time
const fn generate_ray_masks() -> [[u64; 8]; 64] {
    let mut masks = [[0u64; 8]; 64];
    let mut sq: usize = 0;
    while sq < 64 {
        let rank = sq / 8;
        let file = sq % 8;

        // North (+8)
        masks[sq][NORTH] = generate_ray(sq, 8, 7, rank);
        // South (-8)
        masks[sq][SOUTH] = generate_ray_neg(sq, 8, rank);
        // East (+1)
        masks[sq][EAST] = generate_ray(sq, 1, 7, file);
        // West (-1)
        masks[sq][WEST] = generate_ray_neg(sq, 1, file);
        // NorthEast (+9) - limited by both rank and file
        masks[sq][NORTH_EAST] = generate_ray_diagonal(sq, 9, 7 - rank, 7 - file);
        // NorthWest (+7) - limited by rank going up and file going left
        masks[sq][NORTH_WEST] = generate_ray_diagonal(sq, 7, 7 - rank, file);
        // SouthEast (-7) - limited by rank going down and file going right
        masks[sq][SOUTH_EAST] = generate_ray_diagonal_neg(sq, 7, rank, 7 - file);
        // SouthWest (-9) - limited by rank going down and file going left
        masks[sq][SOUTH_WEST] = generate_ray_diagonal_neg(sq, 9, rank, file);

        sq += 1;
    }
    masks
}

/// Generate ray in positive direction with step size, limited by max_steps
const fn generate_ray(sq: usize, step: usize, max_edge: usize, current: usize) -> u64 {
    let mut mask = 0u64;
    let max_steps = max_edge - current;
    let mut i = 1;
    while i <= max_steps {
        let target = sq + i * step;
        if target < 64 {
            mask |= 1u64 << target;
        }
        i += 1;
    }
    mask
}

/// Generate ray in negative direction with step size
const fn generate_ray_neg(sq: usize, step: usize, current: usize) -> u64 {
    let mut mask = 0u64;
    let max_steps = current;
    let mut i = 1;
    while i <= max_steps {
        if sq >= i * step {
            let target = sq - i * step;
            mask |= 1u64 << target;
        }
        i += 1;
    }
    mask
}

/// Generate diagonal ray in positive direction
const fn generate_ray_diagonal(sq: usize, step: usize, max_rank: usize, max_file: usize) -> u64 {
    let mut mask = 0u64;
    let max_steps = if max_rank < max_file {
        max_rank
    } else {
        max_file
    };
    let mut i = 1;
    while i <= max_steps {
        let target = sq + i * step;
        if target < 64 {
            mask |= 1u64 << target;
        }
        i += 1;
    }
    mask
}

/// Generate diagonal ray in negative direction
const fn generate_ray_diagonal_neg(
    sq: usize,
    step: usize,
    max_rank: usize,
    max_file: usize,
) -> u64 {
    let mut mask = 0u64;
    let max_steps = if max_rank < max_file {
        max_rank
    } else {
        max_file
    };
    let mut i = 1;
    while i <= max_steps {
        if sq >= i * step {
            let target = sq - i * step;
            mask |= 1u64 << target;
        }
        i += 1;
    }
    mask
}

/// Rook directions: North, South, East, West
const ROOK_DIRS: [usize; 4] = [NORTH, SOUTH, EAST, WEST];
/// Bishop directions: NE, NW, SE, SW
const BISHOP_DIRS: [usize; 4] = [NORTH_EAST, NORTH_WEST, SOUTH_EAST, SOUTH_WEST];

/// Scoring constants for line-based tactical patterns
const ABSOLUTE_PIN_BASE: i32 = 40;
const ABSOLUTE_PIN_QUEEN: i32 = 100;
const ABSOLUTE_PIN_ROOK: i32 = 70;
const ABSOLUTE_PIN_BISHOP: i32 = 60;
const ABSOLUTE_PIN_KNIGHT: i32 = 60;
const ABSOLUTE_PIN_PAWN: i32 = 40;

const RELATIVE_PIN_BASE: i32 = 20;
const RELATIVE_PIN_MAX: i32 = 40;

const XRAY_BONUS: i32 = 15;
const DISCOVERED_ATTACK_BASE: i32 = 20;
const DISCOVERED_ATTACK_MAX: i32 = 50;
const SKEWER_BONUS: i32 = 15;

/// Context for ray evaluation to reduce function argument count
struct RayEvalContext {
    own_occ: u64,
    enemy_occ: u64,
    enemy_king_bb: u64,
    enemy_queen_bb: u64,
}

/// LinePressureEvaluator detects line-based tactical patterns:
/// - Absolute pins (piece pinned to king)
/// - Relative pins (piece pinned to high-value piece)
/// - X-rays (attack through a piece to a valuable target)
/// - Discovered attack potential
/// - Skewer potential
pub struct LinePressureEvaluator;

impl LinePressureEvaluator {
    /// Get piece value for scoring calculations
    #[inline]
    fn piece_value(piece: Piece) -> i32 {
        match piece {
            Piece::Pawn => 100,
            Piece::Knight => 320,
            Piece::Bishop => 330,
            Piece::Rook => 500,
            Piece::Queen => 900,
            Piece::King => 20000,
        }
    }

    /// Evaluate line pressure for one side's sliding pieces
    fn evaluate_side(board: &Board2, color: Color) -> i32 {
        let mut score = 0;
        let enemy = color.opponent();
        let enemy_king_sq = board.king_sq[enemy as usize];

        let ctx = RayEvalContext {
            own_occ: board.occ[color as usize],
            enemy_occ: board.occ[enemy as usize],
            enemy_king_bb: 1u64 << enemy_king_sq,
            enemy_queen_bb: board.pieces[enemy as usize][Piece::Queen as usize],
        };

        // Process rooks
        let mut rooks = board.pieces[color as usize][Piece::Rook as usize];
        while rooks != 0 {
            let sq = rooks.trailing_zeros() as usize;
            score += Self::evaluate_piece_rays(board, sq, &ROOK_DIRS, &ctx);
            rooks &= rooks - 1;
        }

        // Process bishops
        let mut bishops = board.pieces[color as usize][Piece::Bishop as usize];
        while bishops != 0 {
            let sq = bishops.trailing_zeros() as usize;
            score += Self::evaluate_piece_rays(board, sq, &BISHOP_DIRS, &ctx);
            bishops &= bishops - 1;
        }

        // Process queens (all 8 directions)
        let mut queens = board.pieces[color as usize][Piece::Queen as usize];
        while queens != 0 {
            let sq = queens.trailing_zeros() as usize;
            score += Self::evaluate_piece_rays(board, sq, &ROOK_DIRS, &ctx);
            score += Self::evaluate_piece_rays(board, sq, &BISHOP_DIRS, &ctx);
            queens &= queens - 1;
        }

        score
    }

    /// Evaluate all rays from a single piece
    #[inline]
    fn evaluate_piece_rays(
        board: &Board2,
        sq: usize,
        directions: &[usize],
        ctx: &RayEvalContext,
    ) -> i32 {
        let mut score = 0;

        for &dir in directions {
            let ray_mask = RAY_MASKS[sq][dir];
            if ray_mask == 0 {
                continue;
            }

            let occ_on_ray = board.occ_all & ray_mask;
            if occ_on_ray == 0 {
                continue;
            }

            // Find first and second blockers using direction-aware bit scanning
            let (first_sq, second_sq) = Self::find_blockers(occ_on_ray, dir);

            let Some(first_sq) = first_sq else {
                continue;
            };

            let first_bb = 1u64 << first_sq;
            let first_is_enemy = (first_bb & ctx.enemy_occ) != 0;
            let first_is_own = (first_bb & ctx.own_occ) != 0;

            // Get piece info for first blocker
            let first_piece = board.piece_on(first_sq as u8);

            // Check second blocker if exists
            if let Some(second_sq) = second_sq {
                let second_bb = 1u64 << second_sq;
                let second_is_enemy_king = (second_bb & ctx.enemy_king_bb) != 0;
                let second_is_enemy_queen = (second_bb & ctx.enemy_queen_bb) != 0;
                let second_is_enemy = (second_bb & ctx.enemy_occ) != 0;

                // Pattern classification with priority:
                // absolute pin > relative pin > discovered attack > x-ray > skewer

                if first_is_enemy && second_is_enemy_king {
                    // ABSOLUTE PIN: enemy piece pinned to enemy king
                    if let Some((_, first_piece_type)) = first_piece {
                        score += Self::absolute_pin_score(first_piece_type);
                    }
                } else if first_is_enemy && second_is_enemy_queen {
                    // RELATIVE PIN: enemy piece pinned to enemy queen
                    if let Some((_, first_piece_type)) = first_piece {
                        let pinned_value = Self::piece_value(first_piece_type);
                        // Score based on value difference
                        let bonus = RELATIVE_PIN_BASE
                            + ((900 - pinned_value).max(0) * RELATIVE_PIN_MAX / 900);
                        score += bonus.min(RELATIVE_PIN_MAX);
                    }
                } else if first_is_own && second_is_enemy {
                    // DISCOVERED ATTACK POTENTIAL: own piece blocks attack on enemy piece
                    if let Some((_, second_piece_type)) = board.piece_on(second_sq as u8) {
                        let target_value = Self::piece_value(second_piece_type);
                        // Higher score for more valuable targets
                        let bonus = DISCOVERED_ATTACK_BASE
                            + (target_value * (DISCOVERED_ATTACK_MAX - DISCOVERED_ATTACK_BASE)
                                / 900);
                        score += bonus.min(DISCOVERED_ATTACK_MAX);
                    }
                } else if first_is_enemy
                    && second_is_enemy
                    && let Some((_, second_piece_type)) = board.piece_on(second_sq as u8)
                    && matches!(second_piece_type, Piece::King | Piece::Queen | Piece::Rook)
                {
                    // X-RAY: attack through enemy piece to another enemy piece
                    score += XRAY_BONUS;
                }

                // SKEWER: first piece is high-value, second is lower value
                if first_is_enemy
                    && let (Some((_, first_type)), Some((_, second_type))) =
                        (first_piece, board.piece_on(second_sq as u8))
                    && matches!(first_type, Piece::King | Piece::Queen)
                    && second_is_enemy
                    && Self::piece_value(first_type) > Self::piece_value(second_type)
                {
                    score += SKEWER_BONUS;
                }
            }
        }

        score
    }

    /// Find the first and second blockers on a ray using direction-aware bit scanning.
    /// For positive directions (North, East, NE, NW), scan from LSB.
    /// For negative directions (South, West, SE, SW), scan from MSB.
    #[inline]
    fn find_blockers(occ_on_ray: u64, dir: usize) -> (Option<usize>, Option<usize>) {
        if occ_on_ray == 0 {
            return (None, None);
        }

        // Positive directions scan from LSB (closest to source in positive direction)
        // Negative directions scan from MSB (closest to source in negative direction)
        let scan_from_lsb = matches!(dir, NORTH | EAST | NORTH_EAST | NORTH_WEST);

        if scan_from_lsb {
            // Scan from LSB (smallest square number first)
            let first = occ_on_ray.trailing_zeros() as usize;
            let remaining = occ_on_ray & (occ_on_ray - 1); // Clear LSB
            let second = if remaining != 0 {
                Some(remaining.trailing_zeros() as usize)
            } else {
                None
            };
            (Some(first), second)
        } else {
            // Scan from MSB (largest square number first)
            let first = 63 - occ_on_ray.leading_zeros() as usize;
            let remaining = occ_on_ray & !(1u64 << first); // Clear MSB
            let second = if remaining != 0 {
                Some(63 - remaining.leading_zeros() as usize)
            } else {
                None
            };
            (Some(first), second)
        }
    }

    /// Score for absolute pin based on pinned piece type
    #[inline]
    fn absolute_pin_score(pinned_piece: Piece) -> i32 {
        match pinned_piece {
            Piece::Queen => ABSOLUTE_PIN_QUEEN,
            Piece::Rook => ABSOLUTE_PIN_ROOK,
            Piece::Bishop => ABSOLUTE_PIN_BISHOP,
            Piece::Knight => ABSOLUTE_PIN_KNIGHT,
            Piece::Pawn => ABSOLUTE_PIN_PAWN,
            Piece::King => ABSOLUTE_PIN_BASE, // Shouldn't happen, but handle gracefully
        }
    }
}

impl BoardEvaluator for LinePressureEvaluator {
    fn evaluate(&self, board: &Board2) -> i32 {
        let white_score = Self::evaluate_side(board, Color::White);
        let black_score = Self::evaluate_side(board, Color::Black);
        white_score - black_score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_masks_north() {
        // From a1 (sq 0), north ray should include a2-a8
        let ray = RAY_MASKS[0][NORTH];
        assert_eq!(ray, 0x0101010101010100);
    }

    #[test]
    fn test_ray_masks_south() {
        // From a8 (sq 56), south ray should include a1-a7
        let ray = RAY_MASKS[56][SOUTH];
        assert_eq!(ray, 0x0001010101010101);
    }

    #[test]
    fn test_ray_masks_east() {
        // From a1 (sq 0), east ray should include b1-h1
        let ray = RAY_MASKS[0][EAST];
        assert_eq!(ray, 0x00000000000000FE);
    }

    #[test]
    fn test_ray_masks_diagonal_ne() {
        // From a1 (sq 0), NE ray should include b2, c3, d4, e5, f6, g7, h8
        let ray = RAY_MASKS[0][NORTH_EAST];
        let expected = (1u64 << 9)
            | (1u64 << 18)
            | (1u64 << 27)
            | (1u64 << 36)
            | (1u64 << 45)
            | (1u64 << 54)
            | (1u64 << 63);
        assert_eq!(ray, expected);
    }

    #[test]
    fn test_find_blockers_lsb() {
        // Occupancy on squares 8, 16, 24 (a2, a3, a4)
        let occ = (1u64 << 8) | (1u64 << 16) | (1u64 << 24);
        let (first, second) = LinePressureEvaluator::find_blockers(occ, NORTH);
        assert_eq!(first, Some(8));
        assert_eq!(second, Some(16));
    }

    #[test]
    fn test_find_blockers_msb() {
        // Occupancy on squares 8, 16, 24 (a2, a3, a4), scanning from MSB
        let occ = (1u64 << 8) | (1u64 << 16) | (1u64 << 24);
        let (first, second) = LinePressureEvaluator::find_blockers(occ, SOUTH);
        assert_eq!(first, Some(24));
        assert_eq!(second, Some(16));
    }
}
