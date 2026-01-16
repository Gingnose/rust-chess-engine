/// Here we define associated movements, captures or 
/// other traits with this piece, the almighty King !!
use crate::board::{Board, Square};

/// KingMoves is an Unit Struct, namespace to group related functions together.
pub struct KingMoves;

impl KingMoves {
    // Functions are grouped under KingMoves
    pub fn generate_moves(board: &Board, from: Square) -> Vec<Square> {
        let mut moves = Vec::new();
        
        // Get the color of the piece that's moving
        let piece = board.get_piece(from);
        let our_color = match piece {
            Some(p) => p.color,
            None => return moves, // No piece at 'from', return empty
        };

        // King's 8 directions
        let directions: [(i8, i8); 8] = [
            (-1, -1), (-1, 0), (-1, 1),
            ( 0, -1),          ( 0, 1),
            ( 1, -1), ( 1, 0), ( 1, 1),
        ];

        for (dr, dc) in directions {
            let new_row = from.0 as i8 + dr;
            let new_col = from.1 as i8 + dc;

            // Check 1: Is the square on the board?
            if new_row >= 0 && new_row < 8 && new_col >= 0 && new_col < 8 {
                let to = (new_row as u8, new_col as u8);

                // Check 2: Is the square occupied by our own pieces?
                match board.get_piece(to) {
                    None => moves.push(to), // Empty square
                    Some(p) => {
                        if p.color != our_color {
                            moves.push(to); // Enemy pieces can be captured
                        }
                        // Don't add when own piece
                    }
                }
            }
        }

        moves // Return all valid squares
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Color, Piece, PieceType};

    #[test]
    fn test_king_moves_center() {
        let mut board = Board::new();
        // Place King on e4 (row 4, col 4)
        board.set_piece((4, 4), Some(Piece::new(PieceType::King, Color::White)));

        let moves = KingMoves::generate_moves(&board, (4, 4));

        // King in center should have 8 moves
        assert_eq!(moves.len(), 8, "King in center should have 8 moves");
        
        // Check all 8 directions
        assert!(moves.contains(&(3, 3)), "Should move to d5");
        assert!(moves.contains(&(3, 4)), "Should move to e5");
        assert!(moves.contains(&(3, 5)), "Should move to f5");
        assert!(moves.contains(&(4, 3)), "Should move to d4");
        assert!(moves.contains(&(4, 5)), "Should move to f4");
        assert!(moves.contains(&(5, 3)), "Should move to d3");
        assert!(moves.contains(&(5, 4)), "Should move to e3");
        assert!(moves.contains(&(5, 5)), "Should move to f3");
    }

    #[test]
    fn test_king_moves_corner() {
        let mut board = Board::new();
        // Place King on a8 (row 0, col 0)
        board.set_piece((0, 0), Some(Piece::new(PieceType::King, Color::White)));

        let moves = KingMoves::generate_moves(&board, (0, 0));

        // King in corner should have 3 moves
        assert_eq!(moves.len(), 3, "King in corner should have 3 moves");
    }

    #[test]
    fn test_king_blocked_by_own_piece() {
        let mut board = Board::new();
        // Place King on e4
        board.set_piece((4, 4), Some(Piece::new(PieceType::King, Color::White)));
        // Place own piece on e5
        board.set_piece((3, 4), Some(Piece::new(PieceType::QNC, Color::White)));

        let moves = KingMoves::generate_moves(&board, (4, 4));

        // Should NOT include e5 (blocked by own piece)
        assert!(!moves.contains(&(3, 4)), "Should not capture own piece");
        assert_eq!(moves.len(), 7, "King should have 7 moves (one blocked)");
    }

    #[test]
    fn test_king_can_capture_enemy() {
        let mut board = Board::new();
        // Place King on e4
        board.set_piece((4, 4), Some(Piece::new(PieceType::King, Color::White)));
        // Place enemy piece on e5
        board.set_piece((3, 4), Some(Piece::new(PieceType::King, Color::Black)));

        let moves = KingMoves::generate_moves(&board, (4, 4));

        // Should include e5 (can capture enemy)
        assert!(moves.contains(&(3, 4)), "Should be able to capture enemy piece");
        assert_eq!(moves.len(), 8, "King should still have 8 moves (can capture)");
    }

    #[test]
    fn test_king_no_piece_returns_empty() {
        let board = Board::new();
        let moves = KingMoves::generate_moves(&board, (4, 4));
        assert!(moves.is_empty(), "No piece at square should return empty moves");
    }
}