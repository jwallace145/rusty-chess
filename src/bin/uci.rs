use rusty_chess::board::Board;
use rusty_chess::search::ChessEngine;
use std::io::{self, BufRead};

const ENGINE_NAME: &str = "Rusty Chess";
const ENGINE_AUTHOR: &str = "Jimmy Wallace";

struct UciEngine {
    engine: ChessEngine,
    board: Board,
    debug: bool,
}

impl UciEngine {
    fn new() -> Self {
        Self {
            engine: ChessEngine::new(),
            board: Board::new(),
            debug: false,
        }
    }

    fn run(&mut self) {
        let stdin = io::stdin();
        let mut lines = stdin.lock().lines();

        while let Some(Ok(line)) = lines.next() {
            let line = line.trim();

            if self.debug {
                eprintln!(">>> {}", line);
            }

            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "uci" => self.uci(),
                "isready" => self.isready(),
                "ucinewgame" => self.ucinewgame(),
                "position" => self.position(&parts[1..]),
                "go" => self.go(&parts[1..]),
                "stop" => {} // Currently we don't support stopping mid-search
                "quit" => break,
                "debug" => self.debug_cmd(&parts[1..]),
                "setoption" => self.setoption(&parts[1..]),
                "d" | "display" => self.display(), // Non-standard but useful for debugging
                _ => {
                    if self.debug {
                        eprintln!("Unknown command: {}", parts[0]);
                    }
                }
            }
        }
    }

    fn uci(&self) {
        println!("id name {}", ENGINE_NAME);
        println!("id author {}", ENGINE_AUTHOR);

        // Options that can be set
        println!("option name Hash type spin default 64 min 1 max 1024");
        println!("option name OwnBook type check default false");
        println!("option name BookFile type string default opening_book.bin");

        println!("uciok");
    }

    fn isready(&self) {
        println!("readyok");
    }

    fn ucinewgame(&mut self) {
        self.engine.new_game();
        self.board = Board::new();

        if self.debug {
            eprintln!("New game started");
        }
    }

    fn position(&mut self, args: &[&str]) {
        if args.is_empty() {
            return;
        }

        let move_index;

        // Parse the position
        match args[0] {
            "startpos" => {
                self.board = Board::new();
                move_index = args.iter().position(|&x| x == "moves");
            }
            "fen" => {
                // Find where "moves" keyword starts
                move_index = args.iter().position(|&x| x == "moves");

                let fen_end = move_index.unwrap_or(args.len());
                let fen_parts = &args[1..fen_end];
                let fen = fen_parts.join(" ");

                match Board::from_fen(&fen) {
                    Ok(board) => {
                        self.board = board;
                    }
                    Err(e) => {
                        if self.debug {
                            eprintln!("Error parsing FEN: {}", e);
                        }
                        return;
                    }
                }
            }
            _ => {
                if self.debug {
                    eprintln!("Invalid position command");
                }
                return;
            }
        }

        // Apply moves if any
        if let Some(idx) = move_index {
            for move_str in &args[idx + 1..] {
                match self.board.parse_uci(move_str) {
                    Ok(chess_move) => {
                        self.board.apply_move(chess_move);
                    }
                    Err(e) => {
                        if self.debug {
                            eprintln!("Error parsing move {}: {}", move_str, e);
                        }
                        return;
                    }
                }
            }
        }

        if self.debug {
            eprintln!("Position set: {}", self.board.to_fen());
        }
    }

    fn go(&mut self, args: &[&str]) {
        let mut depth = 5; // Default depth
        let mut _movetime: Option<u64> = None; // TODO: Implement time-based search

        let mut i = 0;
        while i < args.len() {
            match args[i] {
                "depth" => {
                    if i + 1 < args.len() {
                        if let Ok(d) = args[i + 1].parse::<u8>() {
                            depth = d.min(20); // Cap at 20
                        }
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "movetime" => {
                    if i + 1 < args.len() {
                        if let Ok(t) = args[i + 1].parse::<u64>() {
                            _movetime = Some(t);
                        }
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "infinite" => {
                    depth = 20; // Max depth for infinite search
                    i += 1;
                }
                "wtime" | "btime" | "winc" | "binc" | "movestogo" => {
                    // Skip time control for now
                    i += 2;
                }
                _ => {
                    i += 1;
                }
            }
        }

        if self.debug {
            eprintln!("Searching at depth {}", depth);
        }

        // Check for terminal positions
        if self.board.is_checkmate() {
            println!("info string Position is checkmate");
            println!("bestmove (none)");
            return;
        }

        if self.board.is_stalemate() {
            println!("info string Position is stalemate");
            println!("bestmove (none)");
            return;
        }

        // Search for best move
        let start_time = std::time::Instant::now();

        println!("info string Starting search at depth {}", depth);

        let best_move = self.engine.find_best_move(&self.board, depth);
        let elapsed = start_time.elapsed();

        // Print info about the search
        if let Some(metrics) = self.engine.get_last_search_metrics() {
            let time_ms = elapsed.as_millis();
            let nodes = metrics.nodes_explored;
            let nps = if time_ms > 0 {
                (nodes as u128 * 1000 / time_ms) as u64
            } else {
                0
            };

            println!(
                "info depth {} nodes {} time {} nps {}",
                metrics.max_depth_reached, nodes, time_ms, nps
            );
        }

        // Output the best move
        match best_move {
            Some(chess_move) => {
                println!("bestmove {}", chess_move.to_uci());
            }
            None => {
                println!("bestmove (none)");
            }
        }
    }

    fn debug_cmd(&mut self, args: &[&str]) {
        if args.is_empty() {
            return;
        }

        match args[0] {
            "on" => {
                self.debug = true;
                eprintln!("Debug mode enabled");
            }
            "off" => {
                self.debug = false;
            }
            _ => {}
        }
    }

    fn setoption(&mut self, args: &[&str]) {
        // Parse: setoption name <name> value <value>
        let mut name = String::new();
        let mut value = String::new();

        let mut i = 0;
        while i < args.len() {
            match args[i] {
                "name" => {
                    i += 1;
                    while i < args.len() && args[i] != "value" {
                        if !name.is_empty() {
                            name.push(' ');
                        }
                        name.push_str(args[i]);
                        i += 1;
                    }
                }
                "value" => {
                    i += 1;
                    while i < args.len() {
                        if !value.is_empty() {
                            value.push(' ');
                        }
                        value.push_str(args[i]);
                        i += 1;
                    }
                }
                _ => {
                    i += 1;
                }
            }
        }

        if self.debug {
            eprintln!("Set option: {} = {}", name, value);
        }

        // Handle options
        match name.as_str() {
            "Hash" => {
                // TODO: Implement hash table resizing
                if self.debug {
                    eprintln!("Hash table resizing not yet implemented");
                }
            }
            "OwnBook" => {
                let enabled = value == "true";
                self.engine.set_use_opening_book(enabled);
                if self.debug {
                    eprintln!(
                        "Opening book {}",
                        if enabled { "enabled" } else { "disabled" }
                    );
                }
            }
            "BookFile" => {
                // TODO: Load opening book from file
                if self.debug {
                    eprintln!("Opening book loading not yet implemented");
                }
            }
            _ => {
                if self.debug {
                    eprintln!("Unknown option: {}", name);
                }
            }
        }
    }

    fn display(&self) {
        self.board.print();
        println!("FEN: {}", self.board.to_fen());
    }
}

fn main() {
    let mut uci_engine = UciEngine::new();
    uci_engine.run();
}
