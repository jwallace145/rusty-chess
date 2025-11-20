use once_cell::sync::Lazy;
use std::sync::Arc;

use crate::board::Color;

use super::attacks::{BLACK_PAWN_ATTACKS, KING_ATTACKS, KNIGHT_ATTACKS, WHITE_PAWN_ATTACKS};
use super::magic::{
    BISHOP_MAGICS, BISHOP_MASKS, BISHOP_RELEVANT_BITS, BOARD_SIZE, ROOK_MAGICS, ROOK_MASKS,
    ROOK_RELEVANT_BITS,
};

pub struct AttackTables {
    pub rook: Vec<Vec<u64>>,
    pub bishop: Vec<Vec<u64>>,
    pub white_pawn: [u64; 64],
    pub black_pawn: [u64; 64],
    pub knight: [u64; 64],
    pub king: [u64; 64],
}

// ---------------------------
// GLOBAL STATIC INSTANCE
// ---------------------------

pub static ATTACK_TABLES: Lazy<Arc<AttackTables>> = Lazy::new(|| {
    let tables =
        AttackTables::load_from_bin("attack_tables.bin").expect("Failed to load attack tables");
    Arc::new(tables)
});

// ---------------------------
// IMPLEMENTATION
// ---------------------------

impl AttackTables {
    pub fn load_from_bin(path: &str) -> std::io::Result<Self> {
        use std::fs::File;
        use std::io::{BufReader, Read};

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let mut rook = Vec::with_capacity(BOARD_SIZE);
        let mut bishop = Vec::with_capacity(BOARD_SIZE);

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
        let index = ((occ_all & mask).wrapping_mul(magic)) >> (64 - relevant_bits);
        table[square][index as usize]
    }
}
