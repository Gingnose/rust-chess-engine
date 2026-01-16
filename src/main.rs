use rust_chess_engine::board::{Board, Color, Square};
use rust_chess_engine::search::find_best_move;
use rust_chess_engine::uci::uci_loop;
use std::env;
use std::io::{self, Write};

/// Parse algebraic notation (e.g., "e2e4") to (from, to) squares
fn parse_move(input: &str) -> Option<(Square, Square)> {
    let input = input.trim().to_lowercase();
    if input.len() != 4 {
        return None;
    }

    let chars: Vec<char> = input.chars().collect();

    // Parse "from" square (e.g., "e2")
    let from_col = (chars[0] as u8).checked_sub(b'a')?;
    let from_row = (b'8').checked_sub(chars[1] as u8)?;

    // Parse "to" square (e.g., "e4")
    let to_col = (chars[2] as u8).checked_sub(b'a')?;
    let to_row = (b'8').checked_sub(chars[3] as u8)?;

    if from_col > 7 || from_row > 7 || to_col > 7 || to_row > 7 {
        return None;
    }

    Some(((from_row, from_col), (to_row, to_col)))
}

/// Convert a square to algebraic notation (e.g., (7, 4) -> "e1")
fn square_to_notation(square: Square) -> String {
    let col = (b'a' + square.1) as char;
    let row = (b'8' - square.0) as char;
    format!("{}{}", col, row)
}

/// Print game instructions
fn print_help() {
    println!("Commands:");
    println!("  <move>  - Enter move in format: e2e4 (from-to)");
    println!("  auto    - Let the engine play for current side");
    println!("  play    - Auto-play: engine vs engine until game ends");
    println!("  undo    - Undo last move");
    println!("  moves   - Show all legal moves");
    println!("  help    - Show this help");
    println!("  quit    - Exit the game");
    println!();
}

fn main() {
    // Check for UCI mode
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "--uci" || arg == "uci") {
        uci_loop();
        return;
    }

    // Interactive CLI mode
    println!("╔═══════════════════════════════════════╗");
    println!("║   Amazon + K vs R + K Chess Engine    ║");
    println!("║   Checkmate the defended King!        ║");
    println!("╚═══════════════════════════════════════╝");
    println!();
    println!("Run with --uci for UCI protocol mode.");
    println!();

    let mut board = Board::setup_amazon_vs_rook();
    let mut move_history: Vec<rust_chess_engine::board::Move> = Vec::new();
    let search_depth = 4;

    print_help();
    println!("{}", board);

    loop {
        let side = board.side_to_move();
        let side_name = match side {
            Color::White => "White",
            Color::Black => "Black",
        };

        // Check for game end
        if board.is_checkmate(side) {
            println!("*** CHECKMATE! {} loses. ***", side_name);
            break;
        }
        if board.is_stalemate(side) {
            println!("*** STALEMATE! Draw. ***");
            break;
        }
        if board.is_in_check(side) {
            println!("*** {} is in CHECK! ***", side_name);
        }

        print!("{} to move > ", side_name);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Error reading input");
            continue;
        }

        let input = input.trim().to_lowercase();

        match input.as_str() {
            "quit" | "exit" | "q" => {
                println!("Goodbye!");
                break;
            }
            "help" | "h" | "?" => {
                print_help();
            }
            "auto" | "a" => {
                println!("Engine thinking (depth {})...", search_depth);
                if let Some((best_move, score)) = find_best_move(&mut board, search_depth) {
                    let from_str = square_to_notation(best_move.from);
                    let to_str = square_to_notation(best_move.to);
                    println!(
                        "Engine plays: {}{} (score: {})",
                        from_str, to_str, score
                    );
                    let mv = board.make_move(best_move.from, best_move.to);
                    move_history.push(mv);
                    println!();
                    println!("{}", board);
                } else {
                    println!("No legal moves available!");
                }
            }
            "play" | "p" => {
                println!("=== Auto-play: Engine vs Engine ===");
                println!();
                let mut move_count = 0;
                let max_moves = 200;

                loop {
                    let current_side = board.side_to_move();
                    let current_side_name = match current_side {
                        Color::White => "White",
                        Color::Black => "Black",
                    };

                    // Check for game end
                    if board.is_checkmate(current_side) {
                        println!();
                        println!("*** CHECKMATE! {} loses. ***", current_side_name);
                        println!("Game finished in {} moves.", move_count);
                        println!();
                        println!("{}", board);
                        break;
                    }
                    if board.is_stalemate(current_side) {
                        println!();
                        println!("*** STALEMATE! Draw. ***");
                        println!("Game finished in {} moves.", move_count);
                        println!();
                        println!("{}", board);
                        break;
                    }

                    // Safety limit
                    if move_count >= max_moves {
                        println!();
                        println!("*** Draw by {} move limit. ***", max_moves);
                        println!();
                        println!("{}", board);
                        break;
                    }

                    // Engine plays
                    if let Some((best_move, score)) = find_best_move(&mut board, search_depth) {
                        move_count += 1;
                        let from_str = square_to_notation(best_move.from);
                        let to_str = square_to_notation(best_move.to);

                        // Make move first to check if it results in check
                        let mv = board.make_move(best_move.from, best_move.to);
                        move_history.push(mv);

                        let check_marker = if board.is_in_check(board.side_to_move()) {
                            "+"
                        } else {
                            ""
                        };

                        println!(
                            "{}. {} {}{}{} (score: {})",
                            move_count, current_side_name, from_str, to_str, check_marker, score
                        );
                    } else {
                        println!("No legal moves for {}!", current_side_name);
                        break;
                    }
                }
            }
            "undo" | "u" => {
                if let Some(mv) = move_history.pop() {
                    board.unmake_move(mv);
                    println!("Move undone.");
                    println!();
                    println!("{}", board);
                } else {
                    println!("No moves to undo.");
                }
            }
            "moves" | "m" => {
                let moves = board.generate_legal_moves();
                if moves.is_empty() {
                    println!("No legal moves!");
                } else {
                    println!("Legal moves ({}):", moves.len());
                    for mv in &moves {
                        let from_str = square_to_notation(mv.from);
                        let to_str = square_to_notation(mv.to);
                        print!("{}{} ", from_str, to_str);
                    }
                    println!();
                }
            }
            _ => {
                // Try to parse as a move
                if let Some((from, to)) = parse_move(&input) {
                    // Check if the move is legal
                    let legal_moves = board.generate_legal_moves();
                    let is_legal = legal_moves
                        .iter()
                        .any(|mv| mv.from == from && mv.to == to);

                    if is_legal {
                        let mv = board.make_move(from, to);
                        move_history.push(mv);
                        println!();
                        println!("{}", board);
                    } else {
                        println!("Illegal move! Type 'moves' to see legal moves.");
                    }
                } else if !input.is_empty() {
                    println!("Invalid input. Type 'help' for commands.");
                }
            }
        }
    }
}
