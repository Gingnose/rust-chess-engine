/// Amazon (Queen + Knight) move generation
/// A powerful fairy chess piece combining Queen and Knight movements
use crate::board::{Board, Color, Square};

pub struct AmazonMoves;

impl AmazonMoves {
    /// Generate all pseudo-legal moves for an Amazon piece
    /// Amazon combines: Queen (sliding) + Knight (2,1 jump)
    pub fn generate_moves(board: &Board, from: Square) -> Vec<Square> {
        let mut moves = Vec::with_capacity(48); // Amazon can have many moves

        // Get the color of the piece that's moving
        let piece = board.get_piece(from);
        let our_color = match piece {
            Some(p) => p.color,
            None => return moves, // No piece at 'from', return empty
        };

        // 1. Queen moves (sliding in 8 directions)
        Self::generate_sliding_moves(&mut moves, board, from, our_color);

        // 2. Knight moves (8 jump patterns)
        Self::generate_knight_moves(&mut moves, board, from, our_color);

        moves
    }

    /// Generate Queen-like sliding moves (8 directions, any distance)
    fn generate_sliding_moves(
        moves: &mut Vec<Square>,
        board: &Board,
        from: Square,
        our_color: Color,
    ) {
        // 8 directions: diagonals + orthogonals
        let directions: [(i8, i8); 8] = [
            (-1, -1), (-1, 0), (-1, 1),
            ( 0, -1),          ( 0, 1),
            ( 1, -1), ( 1, 0), ( 1, 1),
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
    }

    /// Generate Knight-like jump moves (L-shape: 2+1)
    fn generate_knight_moves(
        moves: &mut Vec<Square>,
        board: &Board,
        from: Square,
        our_color: Color,
    ) {
        // Knight offsets: (±2, ±1) and (±1, ±2)
        let knight_offsets: [(i8, i8); 8] = [
            (-2, -1), (-2, 1), (-1, -2), (-1, 2),
            ( 1, -2), ( 1, 2), ( 2, -1), ( 2, 1),
        ];

        for (dr, dc) in knight_offsets {
            let new_row = from.0 as i8 + dr;
            let new_col = from.1 as i8 + dc;

            // Check: Is the square on the board?
            if new_row >= 0 && new_row < 8 && new_col >= 0 && new_col < 8 {
                let to = (new_row as u8, new_col as u8);

                match board.get_piece(to) {
                    None => moves.push(to),                        // Empty - can move
                    Some(p) if p.color != our_color => moves.push(to), // Enemy - can capture
                    _ => {}                                        // Own piece - blocked
                }
            }
        }
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Piece, PieceType};

    #[test]
    fn test_amazon_on_empty_board_center() {
        let mut board = Board::new();
        // Place Amazon on d4 (row 4, col 3) - center of board
        board.set_piece((4, 3), Some(Piece::new(PieceType::Amazon, Color::White)));

        let moves = AmazonMoves::generate_moves(&board, (4, 3));

        // Amazon should have many moves from the center
        assert!(!moves.is_empty(), "Amazon should have moves");
        
        // Knight moves from d4: c2, e2, b3, f3, b5, f5, c6, e6
        assert!(moves.contains(&(6, 2)), "Knight move to c2 should be possible");
        assert!(moves.contains(&(6, 4)), "Knight move to e2 should be possible");
        
        // Queen sliding should work
        assert!(moves.contains(&(0, 3)), "Queen move to d8 should be possible");
        assert!(moves.contains(&(4, 7)), "Queen move to h4 should be possible");
    }

    #[test]
    fn test_amazon_blocked_by_own_piece() {
        let mut board = Board::new();
        // Place Amazon on d4
        board.set_piece((4, 3), Some(Piece::new(PieceType::Amazon, Color::White)));
        // Place own piece on d5 (blocking sliding north)
        board.set_piece((3, 3), Some(Piece::new(PieceType::King, Color::White)));

        let moves = AmazonMoves::generate_moves(&board, (4, 3));

        // Should NOT be able to move to d5 (blocked by own piece)
        assert!(!moves.contains(&(3, 3)), "Should not capture own piece");
        // Should NOT be able to slide through to d6, d7, d8
        assert!(!moves.contains(&(2, 3)), "Should not slide through own piece");
    }

    #[test]
    fn test_amazon_can_capture_enemy() {
        let mut board = Board::new();
        // Place Amazon on d4
        board.set_piece((4, 3), Some(Piece::new(PieceType::Amazon, Color::White)));
        // Place enemy piece on d5
        board.set_piece((3, 3), Some(Piece::new(PieceType::King, Color::Black)));

        let moves = AmazonMoves::generate_moves(&board, (4, 3));

        // CAN capture enemy on d5
        assert!(moves.contains(&(3, 3)), "Should be able to capture enemy piece");
        // Should NOT slide through to d6, d7, d8 (blocked after capture)
        assert!(!moves.contains(&(2, 3)), "Should not slide through enemy piece");
    }

    #[test]
    fn test_amazon_no_piece_returns_empty() {
        let board = Board::new(); // Empty board
        let moves = AmazonMoves::generate_moves(&board, (4, 3));
        assert!(moves.is_empty(), "No piece at square should return empty moves");
    }
}
