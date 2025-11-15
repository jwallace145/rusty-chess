use rusty_chess::opening::create_basic_book;

fn main() {
    println!("Creating opening book...");

    // Create the opening book using the predefined openings
    let book = create_basic_book();

    // Save the book to the project root directory
    let output_path = "opening_book.bin";

    match book.save(output_path) {
        Ok(_) => {
            println!("Successfully created opening book at: {}", output_path);
            println!("Opening book is ready to use!");
        }
        Err(e) => {
            eprintln!("Error saving opening book: {}", e);
            std::process::exit(1);
        }
    }
}
