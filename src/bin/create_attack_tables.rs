use std::fs::File;
use std::io::{BufWriter, Write};

// =========
// Constants
// =========

pub const ATTACK_TABLES_OUTPUT_FILE: &str = "attack_tables.bin";

const BOARD_SIZE: usize = 64;

pub const MIN_RANK: i32 = 0;
pub const MAX_RANK: i32 = 7;
pub const MIN_FILE: i32 = 0;
pub const MAX_FILE: i32 = 7;

const ROOK_DIRECTIONS: [(i32, i32); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];
pub const ROOK_MAGIC_NUMBERS: [u64; 64] = [
    0x5080008011400020,
    0x0140001000402000,
    0x0280091000200480,
    0x0700081001002084,
    0x0300024408010030,
    0x510004004E480100,
    0x0400044128020090,
    0x8080004100012080,
    0x0220800480C00124,
    0x0020401001C02000,
    0x000A002204428050,
    0x004E002040100A00,
    0x0102000A00041020,
    0x0A0880040080C200,
    0x0002000600018408,
    0x0025001200518100,
    0x8900328001400080,
    0x0848810020400100,
    0xC001410020010153,
    0x4110C90020100101,
    0x00A0808004004800,
    0x401080801C000601,
    0x0100040028104221,
    0x840002000900A054,
    0x1000348280004000,
    0x001000404000E008,
    0x0424410300200035,
    0x2008C22200085200,
    0x0005304D00080100,
    0x000C040080120080,
    0x8404058400080210,
    0x0001848200010464,
    0x6000204001800280,
    0x2410004003C02010,
    0x0181200A80801000,
    0x000C60400A001200,
    0x0B00040180802800,
    0xC00A000280804C00,
    0x4040080504005210,
    0x0000208402000041,
    0xA200400080628000,
    0x0021020240820020,
    0x1020027000848022,
    0x0020500018008080,
    0x10000D0008010010,
    0x0100020004008080,
    0x0008020004010100,
    0x12241C0880420003,
    0x4000420024810200,
    0x0103004000308100,
    0x008C200010410300,
    0x2410008050A80480,
    0x0820880080040080,
    0x0044220080040080,
    0x2040100805120400,
    0x0129000080C20100,
    0x0010402010800101,
    0x0648A01040008101,
    0x0006084102A00033,
    0x0002000870C06006,
    0x0082008820100402,
    0x0012008410050806,
    0x2009408802100144,
    0x821080440020810A,
];
pub const BISHOP_MAGIC_NUMBERS: [u64; 64] = [
    0x2020420401002200,
    0x05210A020A002118,
    0x1110040454C00484,
    0x1008095104080000,
    0xC409104004000000,
    0x0002901048080200,
    0x0044040402084301,
    0x2002030188040200,
    0x0000C8084808004A,
    0x1040040808010028,
    0x40040C0114090051,
    0x40004820802004C4,
    0x0010042420260012,
    0x10024202300C010A,
    0x000054013D101000,
    0x0100020482188A0A,
    0x0120090421020200,
    0x1022204444040C00,
    0x0008000400440288,
    0x0008060082004040,
    0x0044040081A00800,
    0x021200014308A010,
    0x8604040080880809,
    0x0000802D46009049,
    0x00500E8040080604,
    0x0024030030100320,
    0x2004100002002440,
    0x02090C0008440080,
    0x0205010000104000,
    0x0410820405004A00,
    0x8004140261012100,
    0x0A00460000820100,
    0x201004A40A101044,
    0x840C024220208440,
    0x000C002E00240401,
    0x2220A00800010106,
    0x88C0080820060020,
    0x0818030B00A81041,
    0xC091280200110900,
    0x08A8114088804200,
    0x228929109000C001,
    0x1230480209205000,
    0x0A43040202000102,
    0x1011284010444600,
    0x0003041008864400,
    0x0115010901000200,
    0x01200402C0840201,
    0x001A009400822110,
    0x2002111128410000,
    0x8420410288203000,
    0x0041210402090081,
    0x8220002442120842,
    0x0140004010450000,
    0xC0408860086488A0,
    0x0090203E00820002,
    0x0820020083090024,
    0x1040440210900C05,
    0x0818182101082000,
    0x0200800080D80800,
    0x32A9220510209801,
    0x0000901010820200,
    0x0000014064080180,
    0xA001204204080186,
    0xC04010040258C048,
];

pub const ROOK_RELEVANT_BITS: [u32; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11, 12, 11, 11, 11, 11, 11, 11, 12,
];
pub const ROOK_MASKS: [u64; 64] = [
    0x000101010101017E,
    0x000202020202027C,
    0x000404040404047A,
    0x0008080808080876,
    0x001010101010106E,
    0x002020202020205E,
    0x004040404040403E,
    0x008080808080807E,
    0x0001010101017E00,
    0x0002020202027C00,
    0x0004040404047A00,
    0x0008080808087600,
    0x0010101010106E00,
    0x0020202020205E00,
    0x0040404040403E00,
    0x0080808080807E00,
    0x00010101017E0100,
    0x00020202027C0200,
    0x00040404047A0400,
    0x0008080808760800,
    0x00101010106E1000,
    0x00202020205E2000,
    0x00404040403E4000,
    0x00808080807E8000,
    0x000101017E010100,
    0x000202027C020200,
    0x000404047A040400,
    0x0008080876080800,
    0x001010106E101000,
    0x002020205E202000,
    0x004040403E404000,
    0x008080807E808000,
    0x0001017E01010100,
    0x0002027C02020200,
    0x0004047A04040400,
    0x0008087608080800,
    0x0010106E10101000,
    0x0020205E20202000,
    0x0040403E40404000,
    0x0080807E80808000,
    0x00017E0101010100,
    0x00027C0202020200,
    0x00047A0404040400,
    0x0008760808080800,
    0x00106E1010101000,
    0x00205E2020202000,
    0x00403E4040404000,
    0x00807E8080808000,
    0x007E010101010100,
    0x007C020202020200,
    0x007A040404040400,
    0x0076080808080800,
    0x006E101010101000,
    0x005E202020202000,
    0x003E404040404000,
    0x007E808080808000,
    0x7E01010101010100,
    0x7C02020202020200,
    0x7A04040404040400,
    0x7608080808080800,
    0x6E10101010101000,
    0x5E20202020202000,
    0x3E40404040404000,
    0x7E80808080808000,
];
const BISHOP_DIRECTIONS: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
pub const BISHOP_RELEVANT_BITS: [u32; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 6,
];
pub const BISHOP_MASKS: [u64; 64] = [
    0x0040201008040200,
    0x0000402010080400,
    0x0000004020100A00,
    0x0000000040221400,
    0x0000000002442800,
    0x0000000204085000,
    0x0000020408102000,
    0x0002040810204000,
    0x0020100804020000,
    0x0040201008040000,
    0x00004020100A0000,
    0x0000004022140000,
    0x0000000244280000,
    0x0000020408500000,
    0x0002040810200000,
    0x0004081020400000,
    0x0010080402000200,
    0x0020100804000400,
    0x004020100A000A00,
    0x0000402214001400,
    0x0000024428002800,
    0x0002040850005000,
    0x0004081020002000,
    0x0008102040004000,
    0x0008040200020400,
    0x0010080400040800,
    0x0020100A000A1000,
    0x0040221400142200,
    0x0002442800284400,
    0x0004085000500800,
    0x0008102000201000,
    0x0010204000402000,
    0x0004020002040800,
    0x0008040004081000,
    0x00100A000A102000,
    0x0022140014224000,
    0x0044280028440200,
    0x0008500050080400,
    0x0010200020100800,
    0x0020400040201000,
    0x0002000204081000,
    0x0004000408102000,
    0x000A000A10204000,
    0x0014001422400000,
    0x0028002844020000,
    0x0050005008040200,
    0x0020002010080400,
    0x0040004020100800,
    0x0000020408102000,
    0x0000040810204000,
    0x00000A1020400000,
    0x0000142240000000,
    0x0000284402000000,
    0x0000500804020000,
    0x0000201008040200,
    0x0000402010080400,
    0x0002040810204000,
    0x0004081020400000,
    0x000A102040000000,
    0x0014224000000000,
    0x0028440200000000,
    0x0050080402000000,
    0x0020100804020000,
    0x0040201008040200,
];

// ================
// Bitboard Helpers
// ================

#[inline]
fn rank(square: usize) -> i32 {
    (square / 8) as i32
}
#[inline]
fn file(square: usize) -> i32 {
    (square % 8) as i32
}
#[inline]
fn square_index(rank: i32, file: i32) -> usize {
    (rank * 8 + file) as usize
}
#[inline]
fn is_valid_square(rank: i32, file: i32) -> bool {
    (MIN_RANK..=MAX_RANK).contains(&rank) && (MIN_FILE..=MAX_FILE).contains(&file)
}

// Generate all occupancy variations of a mask
fn generate_occupancies(mask: u64) -> Vec<u64> {
    let bits: Vec<usize> = (0..64).filter(|&i| (mask >> i) & 1 != 0).collect();
    let n = bits.len();
    let mut occs = Vec::with_capacity(1 << n);

    for i in 0..(1 << n) {
        let mut occ = 0u64;
        for (j, &bit) in bits.iter().enumerate().take(n) {
            if (i >> j) & 1 != 0 {
                occ |= 1u64 << bit;
            }
        }
        occs.push(occ);
    }
    occs
}

// Compute attack bitboard for a given occupancy
fn sliding_attack(square: usize, blockers: u64, directions: &[(i32, i32)]) -> u64 {
    let mut attack: u64 = 0;
    let r0 = rank(square);
    let f0 = file(square);

    for &(dr, df) in directions {
        let mut r = r0 + dr;
        let mut f = f0 + df;
        while is_valid_square(r, f) {
            let sq = square_index(r, f);
            attack |= 1u64 << sq;
            if blockers & (1u64 << sq) != 0 {
                break;
            }
            r += dr;
            f += df;
        }
    }

    attack
}

// Build attack table for a single square using precomputed mask
fn build_attack_table(
    square: usize,
    mask: u64,
    magic: u64,
    relevant_bits: usize,
    directions: &[(i32, i32)],
) -> Vec<u64> {
    let occupancies = generate_occupancies(mask);
    let mut table = vec![0u64; 1 << relevant_bits];

    for &occ in occupancies.iter() {
        let index = ((occ & mask).wrapping_mul(magic)) >> (64 - relevant_bits);
        let attack = sliding_attack(square, occ, directions);

        // Debug for square 0, index 0
        if square == 0 && (index == 0 || occ == 0) {
            println!(
                "Square 0: occ=0x{:016X} -> index={} -> attack=0x{:016X}",
                occ, index, attack
            );
        }

        table[index as usize] = attack;
    }

    // Final verification for square 0
    if square == 0 {
        println!("Square 0: table[0] = 0x{:016X}", table[0]);
    }

    table
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

// =======================
// Main
// =======================

fn main() -> std::io::Result<()> {
    let test_sq: usize = 3;
    let test_blockers: u64 = 0xFFFF00000000FFFF;
    let test_directions: [(i32, i32); 4] = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
    let test_attack_mask: u64 = sliding_attack(test_sq, test_blockers, &test_directions);
    println!(
        "Square {}: \nBlockers: {:#018X}\nAttack Mask: {:#018X}",
        test_sq, test_blockers, test_attack_mask
    );
    print_bitboard(test_blockers);
    print_bitboard(test_attack_mask);

    let mut rook_tables = Vec::with_capacity(BOARD_SIZE);
    let mut bishop_tables = Vec::with_capacity(BOARD_SIZE);

    for square in 0..BOARD_SIZE {
        rook_tables.push(build_attack_table(
            square,
            ROOK_MASKS[square],
            ROOK_MAGIC_NUMBERS[square],
            ROOK_RELEVANT_BITS[square] as usize,
            &ROOK_DIRECTIONS,
        ));

        bishop_tables.push(build_attack_table(
            square,
            BISHOP_MASKS[square],
            BISHOP_MAGIC_NUMBERS[square],
            BISHOP_RELEVANT_BITS[square] as usize,
            &BISHOP_DIRECTIONS,
        ));
    }

    // Write to binary file
    let file = File::create(ATTACK_TABLES_OUTPUT_FILE)?;
    let mut writer = BufWriter::new(file);

    for table in rook_tables.iter().chain(bishop_tables.iter()) {
        for &bb in table {
            writer.write_all(&bb.to_le_bytes())?;
        }
    }

    writer.flush()?;
    println!("Attack tables written to '{}'", ATTACK_TABLES_OUTPUT_FILE);
    Ok(())
}
