pub fn bitboard_from_algebraic(pieces: &[&str]) -> u64 {
    let mut bitboard = 0u64;

    for square_str in pieces {
        if let Some(square_idx) = parse_square(square_str) {
            bitboard |= 1u64 << square_idx;
        }
    }

    bitboard
}

fn parse_square(s: &str) -> Option<usize> {
    if s.len() != 2 {
        return None;
    }

    let bytes = s.as_bytes();
    let file = bytes[0];
    let rank = bytes[1];

    if !(b'a'..=b'h').contains(&file) {
        return None;
    }

    if !(b'1'..=b'8').contains(&rank) {
        return None;
    }

    let file_idx = (file - b'a') as usize;
    let rank_idx = (rank - b'1') as usize;

    Some(rank_idx * 8 + file_idx)
}
