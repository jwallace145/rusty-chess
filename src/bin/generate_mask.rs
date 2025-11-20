use std::fs::File;
use std::io::Write;

fn rook_mask(square: usize) -> u64 {
    let rank = square / 8;
    let file = square % 8;
    let mut mask = 0u64;

    // Horizontal (ignore edges)
    for f in 1..7 {
        if f != file {
            mask |= 1u64 << (rank * 8 + f);
        }
    }

    // Vertical (ignore edges)
    for r in 1..7 {
        if r != rank {
            mask |= 1u64 << (r * 8 + file);
        }
    }

    mask
}

fn bishop_mask(square: usize) -> u64 {
    let rank = square / 8;
    let file = square % 8;
    let mut mask = 0u64;

    for r in 1..7 {
        for f in 1..7 {
            if (r != rank || f != file)
                && ((r as isize - rank as isize).abs() == (f as isize - file as isize).abs())
            {
                mask |= 1u64 << (r * 8 + f);
            }
        }
    }

    mask
}

fn main() -> std::io::Result<()> {
    let mut file = File::create("src/generated_masks.rs")?;

    writeln!(file, "pub const ROOK_MASKS: [u64; 64] = [")?;
    for square in 0..64 {
        writeln!(file, "    0x{:016X},", rook_mask(square))?;
    }
    writeln!(file, "];\n")?;

    writeln!(file, "pub const BISHOP_MASKS: [u64; 64] = [")?;
    for square in 0..64 {
        writeln!(file, "    0x{:016X},", bishop_mask(square))?;
    }
    writeln!(file, "];")?;

    println!("Masks generated successfully in src/generated_masks.rs");

    Ok(())
}
