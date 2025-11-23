use rand::Rng;
use rand::rngs::ThreadRng;
use std::fs::File;
use std::io::Write;

// =========
// Constants
// =========

pub const OUTPUT_FILE: &str = "magic_numbers.rs";

pub const ROOK_RELEVANT_BITS: [u32; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11, 12, 11, 11, 11, 11, 11, 11, 12,
];
pub const BISHOP_RELEVANT_BITS: [u32; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 6,
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

const ROOK_DIRS: [(i32, i32); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];
const BISHOP_DIRS: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

// =====================
// Bitboard Helpers
// =====================

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
    (0..=7).contains(&rank) && (0..=7).contains(&file)
}

// =====================
// Blocker & Attack Generation
// =====================

fn generate_blockers(mask: u64) -> Vec<u64> {
    let bits: Vec<usize> = (0..64).filter(|i| (mask >> i) & 1 != 0).collect();
    let num_blocker_combinations = 1 << bits.len();
    let mut blockers = Vec::with_capacity(num_blocker_combinations);

    for index in 0..num_blocker_combinations {
        let mut b: u64 = 0;
        for (i, &bit) in bits.iter().enumerate() {
            if (index >> i) & 1 != 0 {
                b |= 1u64 << bit;
            }
        }
        blockers.push(b);
    }

    blockers
}

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

// =====================
// Magic Number Finder
// =====================

fn find_magic(square: usize, relevant_bits: u32, directions: &[(i32, i32)], mask: u64) -> u64 {
    let blockers = generate_blockers(mask);
    let attacks: Vec<u64> = blockers
        .iter()
        .map(|&b| sliding_attack(square, b, directions))
        .collect();
    let shift = 64 - relevant_bits;

    let mut rng: ThreadRng = rand::rng();

    for attempt in 0..100_000_000 {
        let magic: u64 = rng.random::<u64>() & rng.random::<u64>() & rng.random::<u64>();
        println!("Testing Magic: {:#018X}", magic);

        if (mask.wrapping_mul(magic) & 0xFF00000000000000).count_ones() < 6 {
            println!(
                "Magic did not have enough one's (>= 6) in first 8 bits: {:#018X}",
                magic
            );
            continue;
        }

        let mut table = vec![0u64; 1 << relevant_bits];
        let mut collision = false;

        for (&blocker, &attack) in blockers.iter().zip(attacks.iter()) {
            let index = (((blocker & mask).wrapping_mul(magic)) >> shift) as usize;
            if table[index] == 0 {
                table[index] = attack;
            } else {
                println!("Attack table index collision!");
                collision = true;
                break;
            }
        }

        if !collision {
            // Verify the magic is truly collision-free
            let mut verify_table = vec![0u64; 1 << relevant_bits];
            for (&blocker, &attack) in blockers.iter().zip(attacks.iter()) {
                let index = (((blocker & mask).wrapping_mul(magic)) >> shift) as usize;
                if verify_table[index] != 0 && verify_table[index] != attack {
                    panic!(
                        "BUG: Collision detected in verification for square {} with magic 0x{:016X}\n  \
                        blocker1 produced attack 0x{:016X}, blocker2=0x{:016X} produced attack 0x{:016X}",
                        square, magic, verify_table[index], blocker, attack
                    );
                }
                verify_table[index] = attack;
            }

            // Extra validation for square 0
            if square == 0 {
                let test_occ = 0x000101000101015A_u64;
                if blockers.contains(&test_occ) {
                    let idx = blockers.iter().position(|&b| b == test_occ).unwrap();
                    println!(
                        "  Square 0: Found problematic occupancy 0x{:016X} with attack 0x{:016X}",
                        test_occ, attacks[idx]
                    );
                    let test_index = ((test_occ.wrapping_mul(magic)) >> shift) as usize;
                    println!(
                        "  Maps to index {} with table value 0x{:016X}",
                        test_index, verify_table[test_index]
                    );
                } else {
                    println!(
                        "  Square 0: Occupancy 0x{:016X} NOT in blockers list!",
                        test_occ
                    );
                }
            }

            if attempt > 1_000_000 {
                println!("  Found magic after {} attempts", attempt);
            }
            return magic;
        }
    }

    panic!(
        "Failed to find magic for square {} after 100M attempts",
        square
    );
}

// =====================
// File Writer
// =====================

fn write_magic_numbers(file: &mut File, name: &str, magics: &[u64; 64]) -> std::io::Result<()> {
    writeln!(file, "pub const {}: [u64; 64] = [", name)?;
    for rank in (0..8).rev() {
        write!(file, "    ")?;
        for file_idx in 0..8 {
            let sq = rank * 8 + file_idx;
            write!(file, "{:#018X}", magics[sq])?;
            if file_idx < 7 || rank > 0 {
                write!(file, ", ")?;
            }
        }
        writeln!(file)?;
    }
    writeln!(file, "];")
}

// =====================
// Main
// =====================

fn main() -> std::io::Result<()> {
    let mut rook_magic: [u64; 64] = [0; 64];
    let mut bishop_magic: [u64; 64] = [0; 64];

    for sq in 0..64 {
        println!("Finding rook magic for square {}", sq);
        rook_magic[sq] = find_magic(sq, ROOK_RELEVANT_BITS[sq], &ROOK_DIRS, ROOK_MASKS[sq]);

        println!("Finding bishop magic for square {}", sq);
        bishop_magic[sq] = find_magic(sq, BISHOP_RELEVANT_BITS[sq], &BISHOP_DIRS, BISHOP_MASKS[sq]);
    }

    let mut file = File::create(OUTPUT_FILE)?;
    writeln!(file, "// Auto-generated magic numbers")?;
    write_magic_numbers(&mut file, "ROOK_MAGIC_NUMBERS", &rook_magic)?;
    write_magic_numbers(&mut file, "BISHOP_MAGIC_NUMBERS", &bishop_magic)?;

    println!("Magic numbers written to {}", OUTPUT_FILE);
    Ok(())
}
