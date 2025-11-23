use once_cell::sync::Lazy;
use std::sync::Arc;

use crate::board::Color;

use super::kings::KING_ATTACKS;
use super::knights::KNIGHT_ATTACKS;
use super::magics::{BISHOP_MAGICS, ROOK_MAGICS};
use super::masks::{BISHOP_MASKS, ROOK_MASKS};
use super::pawns::{BLACK_PAWN_ATTACKS, WHITE_PAWN_ATTACKS};
use super::relevant_bits::{BISHOP_RELEVANT_BITS, ROOK_RELEVANT_BITS};
use std::fs::File;
use std::io::{BufReader, Read};

// =========
// CONSTANTS
// =========

pub const ATTACKS_DB_BIN: &str = "attack_tables.bin";

pub const BOARD_SIZE: usize = 64;

// ================
// ATTACKS DATABASE
// ================

pub struct AttacksDB {
    pub rook: Vec<Vec<u64>>,
    pub bishop: Vec<Vec<u64>>,
    pub white_pawn: [u64; 64],
    pub black_pawn: [u64; 64],
    pub knight: [u64; 64],
    pub king: [u64; 64],
}

// The AttacksDB global static reference for attack table queries
pub static ATTACKS_DB: Lazy<Arc<AttacksDB>> = Lazy::new(|| {
    let tables = AttacksDB::load_from_bin(ATTACKS_DB_BIN).expect("Failed to load AttacksDB bin!");
    Arc::new(tables)
});

// ===============================
// ATTACKS DATABASE IMPLEMENTATION
// ===============================

impl AttacksDB {
    pub fn load_from_bin(path: &str) -> std::io::Result<Self> {
        let file: File = File::open(path)?;
        let mut reader: BufReader<File> = BufReader::new(file);

        let mut rook: Vec<Vec<u64>> = Vec::with_capacity(BOARD_SIZE);
        let mut bishop: Vec<Vec<u64>> = Vec::with_capacity(BOARD_SIZE);

        for &relevant_bits in ROOK_RELEVANT_BITS.iter().take(BOARD_SIZE) {
            let size = 1 << relevant_bits;
            let mut table = Vec::with_capacity(size);
            for _ in 0..size {
                let mut buf = [0u8; 8];
                reader.read_exact(&mut buf)?;
                table.push(u64::from_le_bytes(buf));
            }
            rook.push(table);
        }

        for &relevant_bits in BISHOP_RELEVANT_BITS.iter().take(BOARD_SIZE) {
            let size = 1 << relevant_bits;
            let mut table = Vec::with_capacity(size);
            for _ in 0..size {
                let mut buf = [0u8; 8];
                reader.read_exact(&mut buf)?;
                table.push(u64::from_le_bytes(buf));
            }
            bishop.push(table);
        }

        Ok(Self {
            rook,
            bishop,
            white_pawn: WHITE_PAWN_ATTACKS,
            black_pawn: BLACK_PAWN_ATTACKS,
            knight: KNIGHT_ATTACKS,
            king: KING_ATTACKS,
        })
    }

    #[inline]
    pub fn pawn_attacks(&self, square: usize, color: Color) -> u64 {
        match color {
            Color::White => self.white_pawn[square],
            Color::Black => self.black_pawn[square],
        }
    }

    #[inline]
    pub fn rook_attacks(&self, square: usize, occ_all: u64) -> u64 {
        Self::lookup(
            square,
            occ_all,
            ROOK_MASKS[square],
            ROOK_MAGICS[square],
            ROOK_RELEVANT_BITS[square],
            &self.rook,
        )
    }

    #[inline]
    pub fn knight_attacks(&self, square: usize, _occ_all: u64) -> u64 {
        self.knight[square]
    }

    #[inline]
    pub fn bishop_attacks(&self, square: usize, occ_all: u64) -> u64 {
        Self::lookup(
            square,
            occ_all,
            BISHOP_MASKS[square],
            BISHOP_MAGICS[square],
            BISHOP_RELEVANT_BITS[square],
            &self.bishop,
        )
    }

    #[inline]
    pub fn queen_attacks(&self, square: usize, occ_all: u64) -> u64 {
        self.rook_attacks(square, occ_all) | self.bishop_attacks(square, occ_all)
    }

    #[inline]
    pub fn king_attacks(&self, square: usize, _occ_all: u64) -> u64 {
        self.king[square]
    }

    #[inline]
    fn lookup(
        square: usize,
        occ_all: u64,
        mask: u64,
        magic: u64,
        relevant_bits: u32,
        table: &[Vec<u64>],
    ) -> u64 {
        let index: u64 = ((occ_all & mask).wrapping_mul(magic)) >> (64 - relevant_bits);
        table[square][index as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attack_tables_default_board_rook_a1() {
        // Load the attacks database
        let tables = ATTACKS_DB.clone();

        // Setup default board occupancy (starting position)
        let mut occupancy: u64 = 0xFFFF00000000FFFF;
        occupancy &= !(1u64 << 0); // Remove the rook from A1

        // Square A1 is index 0
        let square_a1 = 0;

        // Get rook attacks from A1
        let attacks = tables.rook_attacks(square_a1, occupancy);

        // On the starting position, a rook on A1 can attack:
        // - B1 (square 1): blocked by white knight
        // - A2 (square 8): blocked by white pawn
        let expected = (1u64 << 1) | (1u64 << 8);

        assert_eq!(
            attacks,
            expected,
            "Rook on A1 should attack B1 (square 1) and A2 (square 8) on default board.\n\
             Expected: {:?}\n\
             Got:      {:?}",
            print_bitboard(expected),
            print_bitboard(attacks)
        );
    }

    #[test]
    fn test_attack_tables_default_board_bishop_c1() {
        // Load the attacks database
        let tables = ATTACKS_DB.clone();

        // Setup default board occupancy (starting position)
        let mut occupancy: u64 = 0xFFFF00000000FFFF;
        occupancy &= !(1u64 << 2); // Remove the Bishop from C1

        // Square C1 is index 0
        let square_c1 = 2;

        // Get rook attacks from A1
        let attacks = tables.bishop_attacks(square_c1, occupancy);

        // On the starting position, a Bishop on C1 can attack:
        // - B2 (square 9): blocked by white pawn
        // - D2 (square 11): blocked by white pawn
        let expected = (1u64 << 9) | (1u64 << 11);

        assert_eq!(
            attacks,
            expected,
            "Bishop on C1 should attack B2 (square 9) and D2 (square 11) on default board.\n\
             Expected: {:?}\n\
             Got:      {:?}",
            print_bitboard(expected),
            print_bitboard(attacks)
        );
    }

    #[test]
    fn test_attack_tables_default_board_white_queen_d1() {
        // Load the attacks database
        let tables: Arc<AttacksDB> = ATTACKS_DB.clone();

        // Setup default board occupancy (starting position)
        let mut occupancy: u64 = 0xFFFF00000000FFFF;
        occupancy &= !(1u64 << 3); // Remove the Queen from D1

        // Square D1 is index 3
        let square_d1: usize = 3;

        let queen_attacks: u64 = tables.queen_attacks(square_d1, occupancy);
        let expected_bishop_attacks: u64 = (1u64 << 10) | (1u64 << 12);
        let expected_rook_attacks: u64 = (1u64 << 2) | (1u64 << 11) | (1u64 << 4);
        let expected_queen_attacks: u64 = expected_bishop_attacks | expected_rook_attacks;

        assert_eq!(
            queen_attacks,
            expected_queen_attacks,
            "Actual Queen attacks do not equal expected Queen attacks!.\n\
             Expected: {:?}\n\
             Got:      {:?}",
            print_bitboard(expected_queen_attacks),
            print_bitboard(queen_attacks)
        );
    }

    #[test]
    fn test_attack_tables_empty_board_white_rook_a1() {
        // Load the attack tables
        let tables: Arc<AttacksDB> = ATTACKS_DB.clone();

        // Setup empty board occupancy
        let mut occupancy: u64 = 0x0000000000000001;
        occupancy &= !(1u64 << 0); // Remove the White Rook from A1

        // Square A1 is index 0
        let square_a1: usize = 0;

        let rook_attacks: u64 = tables.rook_attacks(square_a1, occupancy);
        let expected_rook_attacks: u64 = (1u64 << 8)
            | (1u64 << 16)
            | (1u64 << 24)
            | (1u64 << 32)
            | (1u64 << 40)
            | (1u64 << 48)
            | (1u64 << 56)
            | (1u64 << 1)
            | (1u64 << 2)
            | (1u64 << 3)
            | (1u64 << 4)
            | (1u64 << 5)
            | (1u64 << 6)
            | (1u64 << 7);
        assert_eq!(
            rook_attacks,
            expected_rook_attacks,
            "Actual Rook attacks do not equal expected Rook attacks!\n\
             Expected: {:?}\n\
             Got:      {:?}",
            print_bitboard(expected_rook_attacks),
            print_bitboard(rook_attacks)
        )
    }

    #[test]
    fn test_attack_tables_empty_board_white_bishop_c1() {
        // Load the attack tables
        let tables: Arc<AttacksDB> = ATTACKS_DB.clone();

        // Setup empty board occupancy
        let mut occupancy: u64 = 0x0000000000000004;
        occupancy &= !(1u64 << 2); // Remove the White Rook from A1

        // Square C1 is index 2
        let square_c1: usize = 2;

        let bishop_attacks: u64 = tables.bishop_attacks(square_c1, occupancy);
        let expected_bishop_attacks: u64 = (1u64 << 9)
            | (1u64 << 11)
            | (1u64 << 16)
            | (1u64 << 20)
            | (1u64 << 29)
            | (1u64 << 38)
            | (1u64 << 47);
        assert_eq!(
            bishop_attacks,
            expected_bishop_attacks,
            "Actual Bishop attacks do not equal expected Bishop attacks!\n\
             Expected: {:?}\n\
             Got:      {:?}",
            print_bitboard(expected_bishop_attacks),
            print_bitboard(bishop_attacks)
        )
    }

    #[test]
    fn test_attack_tables_empty_board_white_queen_d1() {
        // Load the attack tables
        let tables: Arc<AttacksDB> = ATTACKS_DB.clone();

        // Square D1 is index 3
        let square: usize = 3;

        // Setup empty board occupancy
        let mut occupancy: u64 = 0x0000000000000008;
        occupancy &= !(1u64 << square); // Remove the White Queen from D1

        let queen_attacks: u64 = tables.queen_attacks(square, occupancy);
        let expected_queen_attacks: u64 = (1u64 << 0)
            | (1u64 << 1)
            | (1u64 << 2)
            | (1u64 << 4)
            | (1u64 << 5)
            | (1u64 << 6)
            | (1u64 << 7)
            | (1u64 << 10)
            | (1u64 << 11)
            | (1u64 << 12)
            | (1u64 << 17)
            | (1u64 << 19)
            | (1u64 << 21)
            | (1u64 << 24)
            | (1u64 << 27)
            | (1u64 << 30)
            | (1u64 << 35)
            | (1u64 << 39)
            | (1u64 << 43)
            | (1u64 << 51)
            | (1u64 << 59);
        assert_eq!(
            queen_attacks,
            expected_queen_attacks,
            "Actual Queen attacks do not equal expected Queen attacks!\n\
             Expected: {:?}\n\
             Got:      {:?}",
            print_bitboard(expected_queen_attacks),
            print_bitboard(queen_attacks)
        )
    }

    /// Print a bitboard visually for debugging
    fn print_bitboard(bb: u64) {
        println!("Bitboard: 0x{:016X}", bb);
        for rank in (0..8).rev() {
            // print from rank 8 to rank 1
            for file in 0..8 {
                let sq = rank * 8 + file;
                if (bb >> sq) & 1 != 0 {
                    print!("1 ");
                } else {
                    print!(". ");
                }
            }
            println!();
        }
        println!();
    }
}
