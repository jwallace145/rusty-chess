use std::fs::File;
use std::io::Write;

// =======================
// Constants
// =======================

pub const RELEVANT_BITS_OUTPUT_FILE: &str = "relevant_bits.rs";

pub const MIN_RANK: i32 = 0;
pub const MAX_RANK: i32 = 7;
pub const MIN_FILE: i32 = 0;
pub const MAX_FILE: i32 = 7;

const ROOK_DIRS: [(i32, i32); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];
const BISHOP_DIRS: [(i32, i32); 4] = [(-1, -1), (-1, 1), (1, -1), (1, 1)];

// =======================
// Helpers
// =======================

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
fn is_valid_rank(rank: i32) -> bool {
    (MIN_RANK..=MAX_RANK).contains(&rank)
}
#[inline]
fn is_valid_file(file: i32) -> bool {
    (MIN_FILE..=MAX_FILE).contains(&file)
}
#[inline]
fn is_valid_square(rank: i32, file: i32) -> bool {
    is_valid_rank(rank) && is_valid_file(file)
}

// ========================
// Sliding mask calculation
// ========================

fn sliding_mask(square: usize, directions: &[(i32, i32)]) -> u64 {
    let r0 = rank(square);
    let f0 = file(square);
    let mut mask: u64 = 0;

    for &(dr, df) in directions {
        let mut r = r0 + dr;
        let mut f = f0 + df;

        while is_valid_square(r, f) {
            if !is_valid_square(r + dr, f + df) {
                break;
            }

            mask |= 1u64 << square_index(r, f);
            r += dr;
            f += df;
        }
    }

    mask
}

// ============================
// Generic relevant bits writer
// ============================

fn write_relevant_bits(file: &mut File, name: &str, bits: &[u32; 64]) -> std::io::Result<()> {
    writeln!(file, "pub const {}: [u32; 64] = [", name)?;

    for rank in 0..8 {
        // Print from top rank down to bottom rank
        write!(file, "    ")?; // Indent each rank
        for file_idx in 0..8 {
            let square = rank * 8 + file_idx;
            write!(file, "{:2}", bits[square])?;
            if square < 63 {
                write!(file, ", ")?;
            } // comma between numbers
        }
        writeln!(file)?; // end of rank line
    }

    writeln!(file, "];")
}

fn write_masks(file: &mut File, name: &str, masks: &[u64; 64]) -> std::io::Result<()> {
    writeln!(file, "pub const {}: [u64; 64] = [", name)?;

    for rank in 0..8 {
        // Print from top rank down to bottom rank
        write!(file, "    ")?; // Indent each rank
        for file_idx in 0..8 {
            let square = rank * 8 + file_idx;
            write!(file, "{:#018X}", masks[square])?;
            if square < 63 {
                write!(file, ", ")?;
            }
        }
        writeln!(file)?; // end of rank line
    }

    writeln!(file, "];")
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

// ======================
// Main generation script
// ======================

fn main() -> std::io::Result<()> {
    let mut rook_masks: [u64; 64] = [0; 64];
    let mut rook_bits: [u32; 64] = [0; 64];
    let mut bishop_masks: [u64; 64] = [0; 64];
    let mut bishop_bits: [u32; 64] = [0; 64];

    for square in 0..64 {
        // Generate rook mask and count relevant bits
        let rook_mask: u64 = sliding_mask(square, &ROOK_DIRS);
        rook_masks[square] = rook_mask;
        println!("Square: {} with Rook Mask: {:#018X}", square, rook_mask);
        print_bitboard(rook_mask);
        rook_bits[square] = rook_mask.count_ones();

        // Generate bishop mask and count relevant bits
        let bishop_mask: u64 = sliding_mask(square, &BISHOP_DIRS);
        bishop_masks[square] = bishop_mask;
        println!("Square: {} with Bishop Mask: {:#018X}", square, bishop_mask);
        print_bitboard(bishop_mask);
        bishop_bits[square] = bishop_mask.count_ones();
    }

    let mut file = File::create(RELEVANT_BITS_OUTPUT_FILE)?;

    writeln!(file, "// Auto-generated relevant bits and masks")?;

    // Write relevant bits
    write_relevant_bits(&mut file, "ROOK_RELEVANT_BITS", &rook_bits)?;
    write_relevant_bits(&mut file, "BISHOP_RELEVANT_BITS", &bishop_bits)?;

    // Write masks as u64 hex arrays
    write_masks(&mut file, "ROOK_MASKS", &rook_masks)?;
    write_masks(&mut file, "BISHOP_MASKS", &bishop_masks)?;

    println!(
        "Relevant bits and masks successfully written to {}",
        RELEVANT_BITS_OUTPUT_FILE
    );
    Ok(())
}
