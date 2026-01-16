// UCI (Universal Chess Interface) Protocol Implementation
// Allows communication with chess GUIs and other engines

use crate::board::{Board, Square};
use crate::search::find_best_move;
use std::io::{self, BufRead, Write};

/// Convert a square to UCI notation (e.g., (7, 4) -> "e1")
fn square_to_uci(square: Square) -> String {
    let col = (b'a' + square.1) as char;
    let row = (b'8' - square.0) as char;
    format!("{}{}", col, row)
}

/// Parse UCI notation to square (e.g., "e1" -> (7, 4))
fn parse_square(s: &str) -> Option<Square> {
    if s.len() < 2 {
        return None;
    }
    let chars: Vec<char> = s.chars().collect();
    let col = (chars[0] as u8).checked_sub(b'a')?;
    let row = (b'8').checked_sub(chars[1] as u8)?;
    if col > 7 || row > 7 {
        return None;
    }
    Some((row, col))
}

/// Parse a UCI move string (e.g., "e2e4") to (from, to) squares
fn parse_uci_move(s: &str) -> Option<(Square, Square)> {
    if s.len() < 4 {
        return None;
    }
    let from = parse_square(&s[0..2])?;
    let to = parse_square(&s[2..4])?;
    Some((from, to))
}

/// Convert a move to UCI notation
fn move_to_uci(from: Square, to: Square) -> String {
    format!("{}{}", square_to_uci(from), square_to_uci(to))
}

/// Main UCI loop - reads commands from stdin and responds
pub fn uci_loop() {
    let stdin = io::stdin();
    let mut board = Board::setup_amazon_vs_rook();
    let mut default_depth = 4;

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "uci" => {
                println!("id name Amazon-vs-Rook Chess Engine");
                println!("id author Gingnose");
                println!("option name UCI_Variant type combo default amazon var amazon");
                println!("option name Depth type spin default 4 min 1 max 10");
                println!("uciok");
                io::stdout().flush().unwrap();
            }

            "isready" => {
                println!("readyok");
                io::stdout().flush().unwrap();
            }

            "ucinewgame" => {
                board = Board::setup_amazon_vs_rook();
                board.clear_history();
            }

            "position" => {
                parse_position(&mut board, &parts[1..]);
            }

            "go" => {
                let depth = parse_go_command(&parts[1..], default_depth);
                if let Some((best_move, _score)) = find_best_move(&mut board, depth) {
                    let uci_move = move_to_uci(best_move.from, best_move.to);
                    println!("bestmove {}", uci_move);
                } else {
                    println!("bestmove 0000"); // No legal move
                }
                io::stdout().flush().unwrap();
            }

            "setoption" => {
                // Parse: setoption name Depth value 6
                if parts.len() >= 5 && parts[1] == "name" && parts[3] == "value" {
                    if parts[2].to_lowercase() == "depth" {
                        if let Ok(d) = parts[4].parse::<i32>() {
                            default_depth = d.clamp(1, 10);
                        }
                    }
                }
            }

            "d" | "display" => {
                // Debug: display board (non-standard but useful)
                eprintln!("{}", board);
            }

            "quit" => {
                break;
            }

            _ => {
                // Unknown command - ignore
            }
        }
    }
}

/// Parse the "position" command
fn parse_position(board: &mut Board, args: &[&str]) {
    if args.is_empty() {
        return;
    }

    // Find where "moves" keyword is (if present)
    let moves_idx = args.iter().position(|&x| x == "moves");

    // Parse position type
    match args[0] {
        "startpos" => {
            *board = Board::setup_amazon_vs_rook();
            board.clear_history();
        }
        "fen" => {
            // Collect FEN parts (everything between "fen" and "moves" or end)
            let fen_end = moves_idx.unwrap_or(args.len());
            if fen_end > 1 {
                let fen_parts = &args[1..fen_end];
                let fen_string = fen_parts.join(" ");
                if let Some(parsed_board) = Board::from_fen(&fen_string) {
                    *board = parsed_board;
                    board.clear_history();
                } else {
                    // FEN parsing failed, use default position
                    *board = Board::setup_amazon_vs_rook();
                }
            } else {
                *board = Board::setup_amazon_vs_rook();
            }
        }
        _ => {
            return;
        }
    }

    // Apply moves if present
    if let Some(idx) = moves_idx {
        for move_str in &args[idx + 1..] {
            if let Some((from, to)) = parse_uci_move(move_str) {
                // Verify it's a legal move
                let legal_moves = board.generate_legal_moves();
                let is_legal = legal_moves.iter().any(|mv| mv.from == from && mv.to == to);
                if is_legal {
                    board.make_move(from, to);
                }
            }
        }
    }
}

/// Parse the "go" command and return the search depth
fn parse_go_command(args: &[&str], default_depth: i32) -> i32 {
    let mut depth = default_depth;

    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "depth" => {
                if i + 1 < args.len() {
                    if let Ok(d) = args[i + 1].parse::<i32>() {
                        depth = d.clamp(1, 20);
                    }
                    i += 1;
                }
            }
            "movetime" => {
                // For simplicity, ignore movetime and use depth
                if i + 1 < args.len() {
                    i += 1;
                }
            }
            "infinite" => {
                // Use max depth for infinite
                depth = 10;
            }
            _ => {}
        }
        i += 1;
    }

    depth
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_square_to_uci() {
        assert_eq!(square_to_uci((7, 4)), "e1");
        assert_eq!(square_to_uci((0, 0)), "a8");
        assert_eq!(square_to_uci((0, 7)), "h8");
        assert_eq!(square_to_uci((7, 0)), "a1");
    }

    #[test]
    fn test_parse_square() {
        assert_eq!(parse_square("e1"), Some((7, 4)));
        assert_eq!(parse_square("a8"), Some((0, 0)));
        assert_eq!(parse_square("h8"), Some((0, 7)));
        assert_eq!(parse_square("a1"), Some((7, 0)));
    }

    #[test]
    fn test_parse_uci_move() {
        assert_eq!(parse_uci_move("e2e4"), Some(((6, 4), (4, 4))));
        assert_eq!(parse_uci_move("d1d6"), Some(((7, 3), (2, 3))));
    }

    #[test]
    fn test_move_to_uci() {
        assert_eq!(move_to_uci((7, 3), (2, 3)), "d1d6");
        assert_eq!(move_to_uci((6, 4), (4, 4)), "e2e4");
    }
}
