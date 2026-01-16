/// Rook move generation
/// Moves horizontally and vertically (orthogonally)
use crate::board::{Board, Square};

pub struct RookMoves;

impl RookMoves {
    /// Generate all pseudo-legal moves for a Rook
    /// Rook slides horizontally and vertically
    pub fn generate_moves(board: &Board, from: Square) -> Vec<Square> {
        let mut moves = Vec::with_capacity(14); // Rook can have up to 14 moves

        // Get the color of the piece that's moving
        let piece = board.get_piece(from);
        let our_color = match piece {
            Some(p) => p.color,
            None => return moves, // No piece at 'from', return empty
        };

        // 4 orthogonal directions: up, down, left, right
        let directions: [(i8, i8); 4] = [
            (-1, 0), // up
            ( 1, 0), // down
            ( 0, -1), // left
            ( 0, 1), // right
        ];

        for (dr, dc) in directions {
            let mut distance = 1;
            loop {
                let new_row = from.0 as i8 + dr * distance;
                let new_col = from.1 as i8 + dc * distance;

                // Check: Is the square on the board?
                if new_row < 0 || new_row >= 8 || new_col < 0 || new_col >= 8 {
                    break; // Off the board, stop this direction
                }

                let to = (new_row as u8, new_col as u8);

                match board.get_piece(to) {
                    None => {
                        // Empty square - can move here, continue searching
                        moves.push(to);
                        distance += 1;
                    }
                    Some(p) => {
                        if p.color != our_color {
                            // Enemy piece - can capture
                            moves.push(to);
                        }
                        // Blocked by a piece (own or enemy), stop this direction
                        break;
                    }
                }
            }
        }

        moves
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
    fn test_rook_on_empty_board_center() {
        let mut board = Board::new();
        // Place Rook on d4 (row 4, col 3) - center of board
        board.set_piece((4, 3), Some(Piece::new(PieceType::Rook, Color::White)));

        let moves = RookMoves::generate_moves(&board, (4, 3));

        // Rook should have 14 moves from center (7 vertical + 7 horizontal)
        assert_eq!(moves.len(), 14, "Rook should have 14 moves from center");
        
        // Vertical moves
        assert!(moves.contains(&(0, 3)), "Rook move to d8 should be possible");
        assert!(moves.contains(&(7, 3)), "Rook move to d1 should be possible");
        
        // Horizontal moves
        assert!(moves.contains(&(4, 0)), "Rook move to a4 should be possible");
        assert!(moves.contains(&(4, 7)), "Rook move to h4 should be possible");
        
        // Should NOT have diagonal moves
        assert!(!moves.contains(&(3, 2)), "Rook should not move diagonally");
        assert!(!moves.contains(&(5, 4)), "Rook should not move diagonally");
    }

    #[test]
    fn test_rook_blocked_by_own_piece() {
        let mut board = Board::new();
        // Place Rook on d4
        board.set_piece((4, 3), Some(Piece::new(PieceType::Rook, Color::White)));
        // Place own piece on d5 (blocking sliding north)
        board.set_piece((3, 3), Some(Piece::new(PieceType::King, Color::White)));

        let moves = RookMoves::generate_moves(&board, (4, 3));

        // Should NOT be able to move to d5 (blocked by own piece)
        assert!(!moves.contains(&(3, 3)), "Should not capture own piece");
        // Should NOT be able to slide through to d6, d7, d8
        assert!(!moves.contains(&(2, 3)), "Should not slide through own piece");
    }

    #[test]
    fn test_rook_can_capture_enemy() {
        let mut board = Board::new();
        // Place Rook on d4
        board.set_piece((4, 3), Some(Piece::new(PieceType::Rook, Color::White)));
        // Place enemy piece on d5
        board.set_piece((3, 3), Some(Piece::new(PieceType::King, Color::Black)));

        let moves = RookMoves::generate_moves(&board, (4, 3));

        // CAN capture enemy on d5
        assert!(moves.contains(&(3, 3)), "Should be able to capture enemy piece");
        // Should NOT slide through to d6, d7, d8 (blocked after capture)
        assert!(!moves.contains(&(2, 3)), "Should not slide through enemy piece");
    }

    #[test]
    fn test_rook_corner() {
        let mut board = Board::new();
        // Place Rook on a1 (row 7, col 0) - corner
        board.set_piece((7, 0), Some(Piece::new(PieceType::Rook, Color::White)));

        let moves = RookMoves::generate_moves(&board, (7, 0));

        // Rook in corner should have 14 moves (7 up + 7 right)
        assert_eq!(moves.len(), 14, "Rook should have 14 moves from corner");
    }

    #[test]
    fn test_rook_no_piece_returns_empty() {
        let board = Board::new(); // Empty board
        let moves = RookMoves::generate_moves(&board, (4, 3));
        assert!(moves.is_empty(), "No piece at square should return empty moves");
    }
}
