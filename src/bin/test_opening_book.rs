use rusty_chess::board::Board;
use rusty_chess::opening::{OpeningBook, create_basic_book};

// Test that a specific position in an opening is present in the book
fn test_position_in_book(book: &OpeningBook, name: &str, moves: &[&str]) -> bool {
    let mut board = Board::new();

    print!("{}: ", name);

    // Play all moves in the sequence
    for move_uci in moves {
        match board.parse_uci(move_uci) {
            Ok(chess_move) => {
                board.apply_move(chess_move);
            }
            Err(e) => {
                println!("✗ Error parsing move {}: {}", move_uci, e);
                return false;
            }
        }
    }

    // Check if the position is in the book
    let hash = board.zobrist_hash;
    if let Some(book_move) = book.probe(hash) {
        // Verify the book move is legal
        match board.parse_uci(book_move) {
            Ok(_) => {
                println!("✓ Position found, suggests: {}", book_move);
                true
            }
            Err(e) => {
                println!("✗ Book suggests illegal move: {} ({})", book_move, e);
                false
            }
        }
    } else {
        println!("✗ Position not found in book");
        false
    }
}

fn main() {
    println!("=== Opening Book Test ===\n");

    // Create a basic opening book
    println!("Creating opening book...");
    let book = create_basic_book();
    println!("Opening book created successfully!\n");

    // Save the book to a file
    let book_path = "opening_book.bin";
    println!("Saving opening book to {}...", book_path);
    match book.save(book_path) {
        Ok(_) => println!("Opening book saved successfully!\n"),
        Err(e) => {
            eprintln!("Failed to save opening book: {}", e);
            return;
        }
    }

    // Load the book from the file
    println!("Loading opening book from {}...", book_path);
    let loaded_book = match OpeningBook::load(book_path) {
        Ok(book) => {
            println!("Opening book loaded successfully!\n");
            book
        }
        Err(e) => {
            eprintln!("Failed to load opening book: {}", e);
            return;
        }
    };

    println!("Testing opening book positions:\n");
    println!("--- AGAINST 1.e4 (AS BLACK) ---");

    let mut passed = 0;
    let mut total = 0;

    // Italian Game (Giuoco Piano) - test characteristic position
    total += 1;
    if test_position_in_book(
        &loaded_book,
        "Italian Game",
        &["e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "f8c5"],
    ) {
        passed += 1;
    }

    // Ruy Lopez - Main Line (Closed) - test Morphy Defense position
    total += 1;
    if test_position_in_book(
        &loaded_book,
        "Ruy Lopez - Closed",
        &[
            "e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "a7a6", "b5a4", "g8f6", "e1g1", "f8e7",
        ],
    ) {
        passed += 1;
    }

    // Ruy Lopez - Berlin Defense
    total += 1;
    if test_position_in_book(
        &loaded_book,
        "Ruy Lopez - Berlin",
        &["e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "g8f6"],
    ) {
        passed += 1;
    }

    // Sicilian Defense - Najdorf characteristic position
    total += 1;
    if test_position_in_book(
        &loaded_book,
        "Sicilian - Najdorf",
        &[
            "e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "a7a6",
        ],
    ) {
        passed += 1;
    }

    // Sicilian Defense - Dragon characteristic position
    total += 1;
    if test_position_in_book(
        &loaded_book,
        "Sicilian - Dragon",
        &[
            "e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3", "g7g6",
        ],
    ) {
        passed += 1;
    }

    // French Defense - Advance
    total += 1;
    if test_position_in_book(
        &loaded_book,
        "French - Advance",
        &["e2e4", "e7e6", "d2d4", "d7d5", "e4e5", "c7c5"],
    ) {
        passed += 1;
    }

    // French Defense - Classical
    total += 1;
    if test_position_in_book(
        &loaded_book,
        "French - Classical",
        &["e2e4", "e7e6", "d2d4", "d7d5", "b1c3", "g8f6", "c1g5"],
    ) {
        passed += 1;
    }

    // Caro-Kann - Classical
    total += 1;
    if test_position_in_book(
        &loaded_book,
        "Caro-Kann - Classical",
        &["e2e4", "c7c6", "d2d4", "d7d5", "b1c3", "d5e4", "c3e4"],
    ) {
        passed += 1;
    }

    // Caro-Kann - Advance
    total += 1;
    if test_position_in_book(
        &loaded_book,
        "Caro-Kann - Advance",
        &["e2e4", "c7c6", "d2d4", "d7d5", "e4e5"],
    ) {
        passed += 1;
    }

    // Scandinavian Defense
    total += 1;
    if test_position_in_book(
        &loaded_book,
        "Scandinavian Defense",
        &["e2e4", "d7d5", "e4d5", "d8d5"],
    ) {
        passed += 1;
    }

    // Pirc Defense
    total += 1;
    if test_position_in_book(
        &loaded_book,
        "Pirc Defense",
        &["e2e4", "d7d6", "d2d4", "g8f6", "b1c3", "g7g6"],
    ) {
        passed += 1;
    }

    println!("\n--- AGAINST 1.d4 (AS BLACK) ---");

    // Queen's Gambit Declined
    total += 1;
    if test_position_in_book(
        &loaded_book,
        "QGD - Orthodox",
        &["d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "g8f6", "c1g5"],
    ) {
        passed += 1;
    }

    // Queen's Gambit Accepted
    total += 1;
    if test_position_in_book(&loaded_book, "QGA", &["d2d4", "d7d5", "c2c4", "d5c4"]) {
        passed += 1;
    }

    // Slav Defense
    total += 1;
    if test_position_in_book(
        &loaded_book,
        "Slav Defense",
        &["d2d4", "d7d5", "c2c4", "c7c6", "g1f3", "g8f6"],
    ) {
        passed += 1;
    }

    // King's Indian Defense
    total += 1;
    if test_position_in_book(
        &loaded_book,
        "King's Indian - Classical",
        &[
            "d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "f8g7", "e2e4", "d7d6",
        ],
    ) {
        passed += 1;
    }

    // Nimzo-Indian Defense
    total += 1;
    if test_position_in_book(
        &loaded_book,
        "Nimzo-Indian - Classical",
        &["d2d4", "g8f6", "c2c4", "e7e6", "b1c3", "f8b4"],
    ) {
        passed += 1;
    }

    // Queen's Indian Defense
    total += 1;
    if test_position_in_book(
        &loaded_book,
        "Queen's Indian",
        &["d2d4", "g8f6", "c2c4", "e7e6", "g1f3", "b7b6"],
    ) {
        passed += 1;
    }

    // Grunfeld Defense
    total += 1;
    if test_position_in_book(
        &loaded_book,
        "Grunfeld Defense",
        &["d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "d7d5"],
    ) {
        passed += 1;
    }

    println!("\n--- ALTERNATIVE WHITE OPENINGS ---");

    // English Opening - Symmetrical
    total += 1;
    if test_position_in_book(
        &loaded_book,
        "English - Symmetrical",
        &["c2c4", "c7c5", "b1c3", "b8c6"],
    ) {
        passed += 1;
    }

    // English Opening - vs e5
    total += 1;
    if test_position_in_book(
        &loaded_book,
        "English - vs e5",
        &["c2c4", "e7e5", "b1c3", "g8f6"],
    ) {
        passed += 1;
    }

    // Reti Opening
    total += 1;
    if test_position_in_book(
        &loaded_book,
        "Reti Opening",
        &["g1f3", "d7d5", "c2c4", "e7e6"],
    ) {
        passed += 1;
    }

    // London System
    total += 1;
    if test_position_in_book(
        &loaded_book,
        "London System",
        &["d2d4", "g8f6", "g1f3", "e7e6", "c1f4"],
    ) {
        passed += 1;
    }

    println!("\n=== Test Summary ===");
    println!("Passed: {}/{}", passed, total);

    if passed == total {
        println!("✓ All tests passed!");
    } else {
        println!("✗ Some tests failed");
    }
}
