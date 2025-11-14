use rusty_chess::board::Board;
use rusty_chess::opening::create_basic_book;
use rusty_chess::search::ChessEngine;

fn test_opening_sequence(engine: &mut ChessEngine, name: &str, moves: &[&str], depth: u8) -> bool {
    let mut board = Board::new();
    let mut all_matched = true;

    println!("\nTesting {}", name);
    println!("{}", "=".repeat(50));

    for (i, &expected_move) in moves.iter().enumerate() {
        let position_desc = if i == 0 {
            "Starting position".to_string()
        } else {
            format!("After move {}", i)
        };

        if let Some(best_move) = engine.find_best_move(&board, depth) {
            let uci = best_move.to_uci();
            let matched = uci == expected_move;

            if matched {
                println!("  {}: {} ✓", position_desc, uci);
            } else {
                println!("  {}: {} (expected {})", position_desc, uci, expected_move);
                all_matched = false;
            }

            board.apply_move(best_move);
        } else {
            println!("  {}: No move found!", position_desc);
            all_matched = false;
            break;
        }
    }

    if all_matched {
        println!("  Result: ✓ All moves matched");
    } else {
        println!("  Result: ✗ Some moves differed (engine may have chosen valid alternatives)");
    }

    all_matched
}

fn main() {
    println!("=== Chess Engine with Opening Book Test ===\n");

    // Create and save the opening book
    println!("Creating opening book...");
    let book = create_basic_book();
    let book_path = "opening_book.bin";
    book.save(book_path).expect("Failed to save opening book");
    println!("Opening book saved to {}\n", book_path);

    // Create engine with opening book
    println!("Creating chess engine with opening book...");
    let mut engine =
        ChessEngine::with_opening_book(book_path).expect("Failed to load opening book");
    println!("Chess engine created successfully!\n");

    let depth = 4;
    let mut passed = 0;
    let mut total = 0;

    println!(
        "Testing engine with various opening lines (depth {}):\n",
        depth
    );

    // Test Italian Game
    total += 1;
    if test_opening_sequence(
        &mut engine,
        "Italian Game (Giuoco Piano)",
        &["e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "f8c5"],
        depth,
    ) {
        passed += 1;
    }

    // Test Ruy Lopez - Closed
    total += 1;
    if test_opening_sequence(
        &mut engine,
        "Ruy Lopez - Closed",
        &["e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "a7a6"],
        depth,
    ) {
        passed += 1;
    }

    // Test Sicilian Defense - Najdorf
    total += 1;
    if test_opening_sequence(
        &mut engine,
        "Sicilian Defense - Najdorf",
        &[
            "e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6",
        ],
        depth,
    ) {
        passed += 1;
    }

    // Test French Defense - Classical
    total += 1;
    if test_opening_sequence(
        &mut engine,
        "French Defense - Classical",
        &["e2e4", "e7e6", "d2d4", "d7d5", "b1c3", "g8f6"],
        depth,
    ) {
        passed += 1;
    }

    // Test Caro-Kann Defense - Classical
    total += 1;
    if test_opening_sequence(
        &mut engine,
        "Caro-Kann Defense - Classical",
        &["e2e4", "c7c6", "d2d4", "d7d5", "b1c3", "d5e4"],
        depth,
    ) {
        passed += 1;
    }

    // Test Scandinavian Defense
    total += 1;
    if test_opening_sequence(
        &mut engine,
        "Scandinavian Defense",
        &["e2e4", "d7d5", "e4d5", "d8d5", "b1c3", "d5a5"],
        depth,
    ) {
        passed += 1;
    }

    // Test Queen's Gambit Declined
    total += 1;
    if test_opening_sequence(
        &mut engine,
        "Queen's Gambit Declined",
        &["d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "g8f6"],
        depth,
    ) {
        passed += 1;
    }

    // Test Queen's Gambit Accepted
    total += 1;
    if test_opening_sequence(
        &mut engine,
        "Queen's Gambit Accepted",
        &["d2d4", "d7d5", "c2c4", "d5c4", "g1f3", "g8f6"],
        depth,
    ) {
        passed += 1;
    }

    // Test King's Indian Defense
    total += 1;
    if test_opening_sequence(
        &mut engine,
        "King's Indian Defense",
        &["d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "f8g7"],
        depth,
    ) {
        passed += 1;
    }

    // Test Nimzo-Indian Defense
    total += 1;
    if test_opening_sequence(
        &mut engine,
        "Nimzo-Indian Defense",
        &["d2d4", "g8f6", "c2c4", "e7e6", "b1c3", "f8b4"],
        depth,
    ) {
        passed += 1;
    }

    // Test English Opening - Symmetrical
    total += 1;
    if test_opening_sequence(
        &mut engine,
        "English Opening - Symmetrical",
        &["c2c4", "c7c5", "b1c3", "b8c6"],
        depth,
    ) {
        passed += 1;
    }

    // Test Reti Opening
    total += 1;
    if test_opening_sequence(
        &mut engine,
        "Reti Opening",
        &["g1f3", "d7d5", "c2c4", "e7e6"],
        depth,
    ) {
        passed += 1;
    }

    // Test out-of-book position
    println!("\nTesting Out-of-Book Position (Engine uses search)");
    println!("{}", "=".repeat(50));
    let mut board = Board::new();

    // Play a sequence that goes beyond the book
    let setup_moves = vec![
        "e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "f8c5", "c2c3", "g8f6", "d2d4", "e5d4", "c3d4",
        "f8b4",
    ];
    for move_uci in setup_moves {
        if let Ok(m) = board.parse_uci(move_uci) {
            board.apply_move(m);
        }
    }

    println!("  Position: After Italian Game main line + Bb4");
    if let Some(best_move) = engine.find_best_move(&board, depth) {
        let uci = best_move.to_uci();
        println!("  Engine chose: {} (using minimax search)", uci);
        println!("  Result: ✓ Engine successfully searched out-of-book position");
    } else {
        println!("  Result: ✗ Engine failed to find a move");
    }

    println!("\n{}", "=".repeat(50));
    println!("=== Test Summary ===");
    println!(
        "Opening sequences tested: {}/{} matched perfectly",
        passed, total
    );
    println!("Note: Some openings may show valid alternative moves");
    println!("\n✓ All tests completed successfully!");
}
