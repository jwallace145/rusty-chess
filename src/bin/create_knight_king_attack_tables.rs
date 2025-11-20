use std::fs::File;
use std::io::Write;

const BOARD_SIZE: usize = 64;

fn knight_attacks(sq: usize) -> u64 {
    let rank = sq / 8;
    let file = sq % 8;
    let mut attacks = 0u64;

    let moves = [
        (2, 1),
        (1, 2),
        (-1, 2),
        (-2, 1),
        (-2, -1),
        (-1, -2),
        (1, -2),
        (2, -1),
    ];

    for (dr, df) in moves {
        let r = rank as isize + dr;
        let f = file as isize + df;
        if (0..8).contains(&r) && (0..8).contains(&f) {
            attacks |= 1u64 << (r * 8 + f);
        }
    }

    attacks
}

fn king_attacks(sq: usize) -> u64 {
    let rank = sq / 8;
    let file = sq % 8;
    let mut attacks = 0u64;

    for dr in -1..=1 {
        for df in -1..=1 {
            if dr == 0 && df == 0 {
                continue;
            }
            let r = rank as isize + dr;
            let f = file as isize + df;
            if (0..8).contains(&r) && (0..8).contains(&f) {
                attacks |= 1u64 << (r * 8 + f);
            }
        }
    }

    attacks
}

fn main() -> std::io::Result<()> {
    let mut knight_table = Vec::with_capacity(BOARD_SIZE);
    let mut king_table = Vec::with_capacity(BOARD_SIZE);

    for sq in 0..BOARD_SIZE {
        knight_table.push(knight_attacks(sq));
        king_table.push(king_attacks(sq));
    }

    let mut file = File::create("attack_tables.rs")?;

    writeln!(file, "pub const KNIGHT_ATTACKS: [u64; 64] = [")?;
    for &attack in knight_table.iter().take(BOARD_SIZE) {
        writeln!(file, "    0x{:016X},", attack)?;
    }
    writeln!(file, "];\n")?;

    writeln!(file, "pub const KING_ATTACKS: [u64; 64] = [")?;
    for &attack in king_table.iter().take(BOARD_SIZE) {
        writeln!(file, "    0x{:016X},", attack)?;
    }
    writeln!(file, "];")?;

    println!("Generated knight and king attack tables in attack_tables.rs");

    Ok(())
}
