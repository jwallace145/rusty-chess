use std::fs::File;
use std::io::{BufWriter, Write};

const BOARD_SIZE: usize = 64;

// Example directions
const ROOK_DIRS: [(i32, i32); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];
const BISHOP_DIRS: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

// === Replace these with your precomputed numbers ===
pub const ROOK_MAGICS: [u64; 64] = [
    0x0000000000000000,
    0x8208D04002204420,
    0x41200208014C1044,
    0x1088132400900200,
    0x04012804024A0080,
    0x0025007044020080,
    0x080A030845822050,
    0x0000000000000000,
    0x0041004000200000,
    0x1621400050032001,
    0x0000806000500082,
    0x10820021100A0042,
    0x0003004410080100,
    0x020F000300040008,
    0x0302005A0088010C,
    0x800200050A802000,
    0x00004A0000100002,
    0x0070004000200841,
    0x0000120020420080,
    0x0010C200200A0010,
    0x0181010008001104,
    0x0D0080800C000200,
    0x1040010100020004,
    0x0208220A00000410,
    0x10011C4604001A90,
    0x0040004080200084,
    0x8800A00480100084,
    0x4028100500210008,
    0x0002000600201009,
    0x20A6008080020400,
    0x2212002200410448,
    0x0002020A00342040,
    0x0000000242000300,
    0x002000D002400420,
    0x1004100280802000,
    0x4225002009001000,
    0x08400C0081800800,
    0x4004008004800200,
    0x8000300804004201,
    0x8200814041009010,
    0x0040200040020211,
    0x2800201000C04009,
    0x0000200411010040,
    0x8030000900510020,
    0x0801002488010010,
    0x000300220C010018,
    0x8000118810140006,
    0x4921402218208000,
    0x4400148024040200,
    0x210082A000400080,
    0x00C0600B10008080,
    0x9004080082100080,
    0x020C000800048080,
    0x0904004100220040,
    0x00A1000A00248100,
    0x1504008034908200,
    0x0000000000000000,
    0x0820104044286200,
    0x9221208410080200,
    0x4209180890140204,
    0x0043880100640200,
    0x114040208C020101,
    0x40410021CA018010,
    0x0000000000000000,
];

pub const BISHOP_MAGICS: [u64; 64] = [
    0x00400424018C2104,
    0x4248100102202000,
    0x109808A10607A200,
    0x01041C0093008010,
    0x882206100000E202,
    0x0001010842700045,
    0x0004010858944038,
    0x200100410C884002,
    0x0010081010288310,
    0x000008060801C500,
    0x060A040302021200,
    0x9140190401080108,
    0x0020045040082404,
    0x0841418210400000,
    0x2008040211100800,
    0x40000200CC040403,
    0x0210000810010810,
    0x0050008524058400,
    0x01100048002440D4,
    0x018C800802004004,
    0x4042001012100028,
    0x00010002004A0600,
    0x140084040C090800,
    0x0202204042080C00,
    0x0022400489080800,
    0x0824201491010104,
    0x1024880190004B11,
    0x0142080084004008,
    0x11020C0002008203,
    0x0048020040404A10,
    0x0000810202013002,
    0x0906061080809080,
    0x0571301002082000,
    0x1200889420081012,
    0x0004042C00020400,
    0x1100020082080180,
    0x0000420020220280,
    0x0450100080083200,
    0x0030420052060108,
    0x20210400910108C1,
    0x00280128A0000A01,
    0x1510880508001012,
    0x1050402C01001001,
    0x2000004208010380,
    0x0000100202000890,
    0x0040104410200040,
    0x1004440800401200,
    0x0210140040400080,
    0x0886020260052201,
    0x0102241248040420,
    0x0020010084900000,
    0x8000008084110801,
    0x0001004405040000,
    0x2C92112005810300,
    0xA320440348110000,
    0x0108100280810000,
    0x020A202104104040,
    0x0000410403448580,
    0x0160A02100493008,
    0x0040000200228801,
    0x2000C41462524400,
    0xA495024042840102,
    0x0608102208011401,
    0x0120060200440082,
];

pub const ROOK_RELEVANT_BITS: [u32; 64] = [
    0, 6, 6, 6, 6, 6, 6, 0, 6, 10, 10, 10, 10, 10, 10, 6, 6, 10, 10, 10, 10, 10, 10, 6, 6, 10, 10,
    10, 10, 10, 10, 6, 6, 10, 10, 10, 10, 10, 10, 6, 6, 10, 10, 10, 10, 10, 10, 6, 6, 10, 10, 10,
    10, 10, 10, 6, 0, 6, 6, 6, 6, 6, 6, 0,
];

pub const BISHOP_RELEVANT_BITS: [u32; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 6,
];

// === Generate sliding attack masks (exclude edges) ===
fn square_index(rank: i32, file: i32) -> Option<usize> {
    if (0..8).contains(&rank) && (0..8).contains(&file) {
        Some((rank * 8 + file) as usize)
    } else {
        None
    }
}

fn sliding_mask(square: usize, directions: &[(i32, i32)], exclude_edges: bool) -> u64 {
    let rank = (square / 8) as i32;
    let file = (square % 8) as i32;
    let mut mask = 0u64;
    for (dr, df) in directions.iter() {
        let mut r = rank + dr;
        let mut f = file + df;
        while let Some(sq) = square_index(r, f) {
            if exclude_edges && (r == 0 || r == 7 || f == 0 || f == 7) {
                break;
            }
            mask |= 1u64 << sq;
            r += dr;
            f += df;
        }
    }
    mask
}

// Generate all occupancy variations of mask
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
    let rank = (square / 8) as i32;
    let file = (square % 8) as i32;
    let mut attacks = 0u64;
    for (dr, df) in directions.iter() {
        let mut r = rank;
        let mut f = file;
        loop {
            r += dr;
            f += df;
            if let Some(sq) = square_index(r, f) {
                attacks |= 1u64 << sq;
                if (blockers >> sq) & 1 != 0 {
                    break;
                }
            } else {
                break;
            }
        }
    }
    attacks
}

// Build attack table for a single square
fn build_attack_table(
    square: usize,
    mask: u64,
    magic: u64,
    relevant_bits: usize,
    directions: &[(i32, i32)],
) -> Vec<u64> {
    let occupancies = generate_occupancies(mask);
    let mut table = vec![0u64; 1 << relevant_bits];
    for occ in occupancies.iter() {
        let index = ((*occ & mask).wrapping_mul(magic)) >> (64 - relevant_bits);
        table[index as usize] = sliding_attack(square, *occ, directions);
    }
    table
}

fn main() -> std::io::Result<()> {
    let mut rook_tables: Vec<Vec<u64>> = Vec::with_capacity(64);
    let mut bishop_tables: Vec<Vec<u64>> = Vec::with_capacity(64);

    // Generate tables
    for square in 0..BOARD_SIZE {
        let rook_mask = sliding_mask(square, &ROOK_DIRS, true);
        let bishop_mask = sliding_mask(square, &BISHOP_DIRS, true);

        rook_tables.push(build_attack_table(
            square,
            rook_mask,
            ROOK_MAGICS[square],
            ROOK_RELEVANT_BITS[square] as usize,
            &ROOK_DIRS,
        ));
        bishop_tables.push(build_attack_table(
            square,
            bishop_mask,
            BISHOP_MAGICS[square],
            BISHOP_RELEVANT_BITS[square] as usize,
            &BISHOP_DIRS,
        ));
    }

    // Write to binary file
    let file = File::create("attack_tables.bin")?;
    let mut writer = BufWriter::new(file);

    for table in rook_tables.iter().chain(bishop_tables.iter()) {
        for &bb in table {
            writer.write_all(&bb.to_le_bytes())?;
        }
    }

    writer.flush()?;
    println!("Attack tables written to attack_tables.bin");
    Ok(())
}
