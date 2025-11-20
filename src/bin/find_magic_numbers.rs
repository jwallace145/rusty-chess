use rand::Rng;
use std::fs::File;
use std::io::Write;

/// Directions for sliding pieces
const ROOK_DIRS: [(i32, i32); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];
const BISHOP_DIRS: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

/// Convert (rank, file) to square index (A1=0, H8=63)
fn square_index(rank: i32, file: i32) -> Option<usize> {
    if (0..8).contains(&rank) && (0..8).contains(&file) {
        Some((rank * 8 + file) as usize)
    } else {
        None
    }
}

/// Generate mask for sliding piece
fn sliding_mask(square: usize, directions: &[(i32, i32)], exclude_edges: bool) -> u64 {
    let rank = (square / 8) as i32;
    let file = (square % 8) as i32;
    let mut mask = 0u64;

    for (dr, df) in directions.iter() {
        let mut r = rank + dr;
        let mut f = file + df;

        while let Some(sq) = square_index(r, f) {
            // exclude outer edges for occupancy mask
            let is_edge = r == 0 || r == 7 || f == 0 || f == 7;
            if exclude_edges && is_edge {
                break;
            }
            mask |= 1u64 << sq;
            r += dr;
            f += df;
        }
    }

    mask
}

/// Generate sliding attacks for a given occupancy
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

/// Generate all occupancies for given mask
fn generate_occupancies(mask: u64) -> Vec<u64> {
    let bits: Vec<usize> = (0..64).filter(|&i| (mask >> i) & 1 != 0).collect();
    let n = bits.len();
    let mut occupancies = Vec::with_capacity(1 << n);

    for i in 0..(1 << n) {
        let mut occ = 0u64;
        for (j, &bit) in bits.iter().enumerate() {
            if (i >> j) & 1 != 0 {
                occ |= 1u64 << bit;
            }
        }
        occupancies.push(occ);
    }

    occupancies
}

/// Generate a random 64-bit number with sparse bits
fn random_u64() -> u64 {
    let mut rng = rand::rng();
    rng.random::<u64>() & rng.random::<u64>() & rng.random::<u64>()
}

/// Calculate relevant bits for a square
fn calculate_relevant_bits(square: usize, directions: &[(i32, i32)]) -> u32 {
    let mask = sliding_mask(square, directions, true);
    mask.count_ones()
}

/// Brute-force search for magic number
fn find_magic(square: usize, relevant_bits: usize, directions: &[(i32, i32)]) -> u64 {
    let mask = sliding_mask(square, directions, true);
    let occupancies = generate_occupancies(mask);
    let attacks: Vec<u64> = occupancies
        .iter()
        .map(|&occ| sliding_attack(square, occ, directions))
        .collect();

    // Handle edge case where there are no relevant bits
    if relevant_bits == 0 || relevant_bits >= 64 {
        return 0;
    }

    loop {
        let magic = random_u64();
        let mut used = vec![0u64; 1 << relevant_bits];
        let mut fail = false;

        for (i, &occ) in occupancies.iter().enumerate() {
            let idx = ((occ.wrapping_mul(magic)) >> (64 - relevant_bits)) as usize;
            if used[idx] != 0 && used[idx] != attacks[i] {
                fail = true;
                break;
            }
            used[idx] = attacks[i];
        }

        if !fail {
            return magic;
        }
    }
}

/// Generate magic numbers for all squares and write to file
fn main() -> std::io::Result<()> {
    println!("Generating magic numbers for all 64 squares...");

    let mut rook_magics = [0u64; 64];
    let mut rook_relevant_bits = [0u32; 64];
    let mut bishop_magics = [0u64; 64];
    let mut bishop_relevant_bits = [0u32; 64];

    // Generate rook magic numbers
    for square in 0..64 {
        let relevant_bits = calculate_relevant_bits(square, &ROOK_DIRS);
        rook_relevant_bits[square] = relevant_bits;
        print!(
            "Finding rook magic for square {} ({} relevant bits)... ",
            square, relevant_bits
        );
        rook_magics[square] = find_magic(square, relevant_bits as usize, &ROOK_DIRS);
        println!("found!");
    }

    // Generate bishop magic numbers
    for square in 0..64 {
        let relevant_bits = calculate_relevant_bits(square, &BISHOP_DIRS);
        bishop_relevant_bits[square] = relevant_bits;
        print!(
            "Finding bishop magic for square {} ({} relevant bits)... ",
            square, relevant_bits
        );
        bishop_magics[square] = find_magic(square, relevant_bits as usize, &BISHOP_DIRS);
        println!("found!");
    }

    // Write to file
    let mut file = File::create("magic_numbers.rs")?;

    writeln!(
        file,
        "// Auto-generated magic numbers for sliding piece move generation"
    )?;
    writeln!(file, "// Generated by find_magic_numbers binary")?;
    writeln!(file)?;

    // Write rook magics
    writeln!(file, "pub const ROOK_MAGICS: [u64; 64] = [")?;
    for (i, &magic) in rook_magics.iter().enumerate() {
        if i % 4 == 0 {
            write!(file, "    ")?;
        }
        write!(file, "0x{:016X}", magic)?;
        if i < 63 {
            write!(file, ", ")?;
        }
        if i % 4 == 3 || i == 63 {
            writeln!(file)?;
        }
    }
    writeln!(file, "];")?;
    writeln!(file)?;

    // Write bishop magics
    writeln!(file, "pub const BISHOP_MAGICS: [u64; 64] = [")?;
    for (i, &magic) in bishop_magics.iter().enumerate() {
        if i % 4 == 0 {
            write!(file, "    ")?;
        }
        write!(file, "0x{:016X}", magic)?;
        if i < 63 {
            write!(file, ", ")?;
        }
        if i % 4 == 3 || i == 63 {
            writeln!(file)?;
        }
    }
    writeln!(file, "];")?;
    writeln!(file)?;

    // Write rook relevant bits
    writeln!(file, "pub const ROOK_RELEVANT_BITS: [u32; 64] = [")?;
    for (i, &bits) in rook_relevant_bits.iter().enumerate() {
        if i % 16 == 0 {
            write!(file, "    ")?;
        }
        write!(file, "{:2}", bits)?;
        if i < 63 {
            write!(file, ", ")?;
        }
        if i % 16 == 15 || i == 63 {
            writeln!(file)?;
        }
    }
    writeln!(file, "];")?;
    writeln!(file)?;

    // Write bishop relevant bits
    writeln!(file, "pub const BISHOP_RELEVANT_BITS: [u32; 64] = [")?;
    for (i, &bits) in bishop_relevant_bits.iter().enumerate() {
        if i % 16 == 0 {
            write!(file, "    ")?;
        }
        write!(file, "{:2}", bits)?;
        if i < 63 {
            write!(file, ", ")?;
        }
        if i % 16 == 15 || i == 63 {
            writeln!(file)?;
        }
    }
    writeln!(file, "];")?;

    println!("\nMagic numbers successfully written to magic_numbers.rs");
    Ok(())
}
