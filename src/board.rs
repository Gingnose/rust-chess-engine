// Board representation and piece logic
// Using Mailbox (8x8 array) approach for clarity and extensibility

use crate::pieces::king::KingMoves;
use crate::pieces::qnc::QncMoves;

// =============================================================================
// Type Definitions
// =============================================================================

/// Square coordinate (row, col) where 0-7
/// Row 0 = rank 8 (black's back rank)
/// Row 7 = rank 1 (white's back rank)
/// Col 0 = file a, Col 7 = file h
pub type Square = (u8, u8);

/// Color of a piece or side to move
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Color {
    White,
    Black,
}

impl Color {
    /// Returns the opposite color
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

/// Type of a chess piece
/// For now, only King and QNC (Actress) are needed for K vs QNC endgame
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PieceType {
    King,
    /// QNC = Queen + Knight + Camel (Actress)
    /// Moves like Queen (sliding), Knight (2,1 jump), and Camel (3,1 jump)
    QNC,
}

/// A chess piece with type and color
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

impl Piece {
    /// Create a new piece
    pub fn new(piece_type: PieceType, color: Color) -> Self {
        Piece { piece_type, color }
    }
}

/// Represents a chess move
#[derive(Copy, Clone, Debug)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub captured: Option<Piece>, // For unmake_move restoration
}

impl Move {
    pub fn new(from: Square, to: Square) -> Self {
        Move {
            from,
            to,
            captured: None,
        }
    }
}

// =============================================================================
// Board Structure
// =============================================================================

/// Chess board using Mailbox representation (8x8 array)
///
/// Coordinate system:
/// - squares[0][0] = a8 (top-left from white's perspective)
/// - squares[7][7] = h1 (bottom-right from white's perspective)
/// - squares[row][col] where row = 7 - rank, col = file
pub struct Board {
    /// 8x8 array of squares, each containing an optional piece
    squares: [[Option<Piece>; 8]; 8],
    /// Which side is to move
    side_to_move: Color,
}

impl Board {
    /// Create an empty board
    pub fn new() -> Self {
        Board {
            squares: [[None; 8]; 8],
            side_to_move: Color::White,
        }
    }

    /// Get the piece at a given square
    pub fn get_piece(&self, square: Square) -> Option<Piece> {
        let (row, col) = square;
        if row < 8 && col < 8 {
            self.squares[row as usize][col as usize]
        } else {
            None
        }
    }

    /// Set a piece at a given square
    pub fn set_piece(&mut self, square: Square, piece: Option<Piece>) {
        let (row, col) = square;
        if row < 8 && col < 8 {
            self.squares[row as usize][col as usize] = piece;
        }
    }

    /// Get the current side to move
    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    /// Set the side to move
    pub fn set_side_to_move(&mut self, color: Color) {
        self.side_to_move = color;
    }

    /// Setup the K vs QNC starting position
    /// Black King on e8, White King on e1, White QNC on d1
    pub fn setup_k_vs_qnc() -> Self {
        let mut board = Board::new();

        // Black King on e8 (row 0, col 4)
        board.set_piece((0, 4), Some(Piece::new(PieceType::King, Color::Black)));

        // White King on e1 (row 7, col 4)
        board.set_piece((7, 4), Some(Piece::new(PieceType::King, Color::White)));

        // White QNC (Actress) on d1 (row 7, col 3)
        board.set_piece((7, 3), Some(Piece::new(PieceType::QNC, Color::White)));

        board.side_to_move = Color::White;
        board
    }

    /// Execute a move, returns the Move with captured piece info for unmake
    pub fn make_move(&mut self, from: Square, to: Square) -> Move {
        let captured = self.get_piece(to);
        let piece = self.get_piece(from);

        self.set_piece(to, piece);
        self.set_piece(from, None);
        self.side_to_move = self.side_to_move.opposite();

        Move { from, to, captured }
    }

    /// Undo a move, restoring the previous state
    pub fn unmake_move(&mut self, mv: Move) {
        let piece = self.get_piece(mv.to);

        self.set_piece(mv.from, piece);
        self.set_piece(mv.to, mv.captured);
        self.side_to_move = self.side_to_move.opposite();
    }

    /// Find the position of a King of the given color
    pub fn find_king(&self, color: Color) -> Option<Square> {
        for row in 0..8 {
            for col in 0..8 {
                if let Some(piece) = self.squares[row][col] {
                    if piece.piece_type == PieceType::King && piece.color == color {
                        return Some((row as u8, col as u8));
                    }
                }
            }
        }
        None
    }

    /// Check if a square is attacked by any piece of the given color
    pub fn is_square_attacked(&self, square: Square, by_color: Color) -> bool {
        for row in 0..8 {
            for col in 0..8 {
                if let Some(piece) = self.squares[row][col] {
                    if piece.color == by_color {
                        let from = (row as u8, col as u8);
                        let moves = match piece.piece_type {
                            PieceType::King => KingMoves::generate_moves(self, from),
                            PieceType::QNC => QncMoves::generate_moves(self, from),
                        };
                        if moves.contains(&square) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Check if the King of the given color is in check
    pub fn is_in_check(&self, color: Color) -> bool {
        if let Some(king_square) = self.find_king(color) {
            self.is_square_attacked(king_square, color.opposite())
        } else {
            false // No king found (shouldn't happen in valid game)
        }
    }

    /// Generate all legal moves for the current side to move
    pub fn generate_legal_moves(&mut self) -> Vec<Move> {
        let mut legal_moves = Vec::new();
        let color = self.side_to_move;

        // Find all pieces of current color and generate their moves
        for row in 0..8 {
            for col in 0..8 {
                if let Some(piece) = self.squares[row][col] {
                    if piece.color == color {
                        let from = (row as u8, col as u8);
                        let pseudo_moves = match piece.piece_type {
                            PieceType::King => KingMoves::generate_moves(self, from),
                            PieceType::QNC => QncMoves::generate_moves(self, from),
                        };

                        // Filter: only keep moves that don't leave King in check
                        for to in pseudo_moves {
                            let mv = self.make_move(from, to);
                            let our_color = color; // make_move toggled side_to_move
                            if !self.is_in_check(our_color) {
                                legal_moves.push(mv);
                            }
                            self.unmake_move(mv);
                        }
                    }
                }
            }
        }

        legal_moves
    }
}

// =============================================================================
// Default Implementation
// =============================================================================

impl Default for Board {
    fn default() -> Self {
        Board::new()
    }
}

// =============================================================================
// Display Implementation (for debugging)
// =============================================================================

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  a b c d e f g h")?;
        writeln!(f, "  +-+-+-+-+-+-+-+")?;

        for row in 0..8 {
            let rank = 8 - row; // Convert row to chess rank (8 to 1)
            write!(f, "{} ", rank)?;

            for col in 0..8 {
                let piece_char = match self.squares[row][col] {
                    None => '.',
                    Some(piece) => {
                        let c = match piece.piece_type {
                            PieceType::King => 'K',
                            PieceType::QNC => 'A', // A for Actress
                        };
                        // Lowercase for black pieces
                        if piece.color == Color::Black {
                            c.to_ascii_lowercase()
                        } else {
                            c
                        }
                    }
                };
                write!(f, "{} ", piece_char)?;
            }
            writeln!(f, "| {}", rank)?;
        }

        writeln!(f, "  +-+-+-+-+-+-+-+")?;
        writeln!(f, "  a b c d e f g h")?;
        writeln!(f)?;
        writeln!(f, "Side to move: {:?}", self.side_to_move)?;

        Ok(())
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_opposite() {
        assert_eq!(Color::White.opposite(), Color::Black);
        assert_eq!(Color::Black.opposite(), Color::White);
    }

    #[test]
    fn test_piece_creation() {
        let white_king = Piece::new(PieceType::King, Color::White);
        assert_eq!(white_king.piece_type, PieceType::King);
        assert_eq!(white_king.color, Color::White);

        let white_qnc = Piece::new(PieceType::QNC, Color::White);
        assert_eq!(white_qnc.piece_type, PieceType::QNC);
        assert_eq!(white_qnc.color, Color::White);
    }

    #[test]
    fn test_board_empty() {
        let board = Board::new();

        // All squares should be empty
        for row in 0..8 {
            for col in 0..8 {
                assert_eq!(board.get_piece((row, col)), None);
            }
        }

        // Default side to move is White
        assert_eq!(board.side_to_move(), Color::White);
    }

    #[test]
    fn test_board_set_get_piece() {
        let mut board = Board::new();
        let king = Piece::new(PieceType::King, Color::White);

        // Set piece
        board.set_piece((4, 4), Some(king));

        // Get piece
        let retrieved = board.get_piece((4, 4));
        assert_eq!(retrieved, Some(king));

        // Remove piece
        board.set_piece((4, 4), None);
        assert_eq!(board.get_piece((4, 4)), None);
    }

    #[test]
    fn test_k_vs_qnc_setup() {
        let board = Board::setup_k_vs_qnc();

        // Black King on e8 (row 0, col 4)
        let black_king = board.get_piece((0, 4)).expect("There should be a piece at (0, 4)");
        assert_eq!(black_king.piece_type, PieceType::King, "Piece at (0, 4) should be a King");
        assert_eq!(black_king.color, Color::Black, "King at (0, 4) should be Black");

        // White King on e1 (row 7, col 4)
        let white_king = board.get_piece((7, 4)).expect("There should be a piece at (7, 4)");
        assert_eq!(white_king.piece_type, PieceType::King, "Piece at (7, 4) should be a King");
        assert_eq!(white_king.color, Color::White, "King at (7, 4) should be White");
        // White QNC on d1 (row 7, col 3)
        let white_qnc = board.get_piece((7, 3)).expect("There should be a piece at (7, 3)");
        assert_eq!(white_qnc.piece_type, PieceType::QNC, "Piece at (7, 3) should be a QNC");
        assert_eq!(white_qnc.color, Color::White, "QNC at (7, 3) should be White");

        // Side to move is White
        assert_eq!(board.side_to_move(), Color::White);
    }

    #[test]
    fn test_board_display() {
        let board = Board::setup_k_vs_qnc();
        let display = format!("{}", board);

        // Check that the display contains expected elements
        assert!(display.contains("a b c d e f g h"));
        assert!(display.contains("k")); // Black king (lowercase)
        assert!(display.contains("K")); // White king (uppercase)
        assert!(display.contains("A")); // White QNC/Actress (uppercase)
        assert!(display.contains("Side to move: White"));
    }

    #[test]
    fn test_make_move_basic() {
        let mut board = Board::new();
        let king = Piece::new(PieceType::King, Color::White);
        board.set_piece((4, 4), Some(king)); // e4

        // Move king from e4 to e5
        let mv = board.make_move((4, 4), (3, 4));

        // Check piece moved
        assert_eq!(board.get_piece((4, 4)), None, "Original square should be empty");
        assert_eq!(board.get_piece((3, 4)), Some(king), "Piece should be at new square");

        // Check move info
        assert_eq!(mv.from, (4, 4));
        assert_eq!(mv.to, (3, 4));
        assert_eq!(mv.captured, None);

        // Check side to move toggled
        assert_eq!(board.side_to_move(), Color::Black);
    }

    #[test]
    fn test_make_move_capture() {
        let mut board = Board::new();
        let white_king = Piece::new(PieceType::King, Color::White);
        let black_king = Piece::new(PieceType::King, Color::Black);

        board.set_piece((4, 4), Some(white_king));
        board.set_piece((3, 4), Some(black_king));

        // White king captures black king
        let mv = board.make_move((4, 4), (3, 4));

        // Check capture info stored
        assert_eq!(mv.captured, Some(black_king));

        // Check board state
        assert_eq!(board.get_piece((4, 4)), None);
        assert_eq!(board.get_piece((3, 4)), Some(white_king));
    }

    #[test]
    fn test_unmake_move_basic() {
        let mut board = Board::new();
        let king = Piece::new(PieceType::King, Color::White);
        board.set_piece((4, 4), Some(king));

        // Make and unmake move
        let mv = board.make_move((4, 4), (3, 4));
        board.unmake_move(mv);

        // Check board restored
        assert_eq!(board.get_piece((4, 4)), Some(king), "Piece should be back at original square");
        assert_eq!(board.get_piece((3, 4)), None, "Target square should be empty");

        // Check side to move restored
        assert_eq!(board.side_to_move(), Color::White);
    }

    #[test]
    fn test_unmake_move_capture() {
        let mut board = Board::new();
        let white_king = Piece::new(PieceType::King, Color::White);
        let black_king = Piece::new(PieceType::King, Color::Black);

        board.set_piece((4, 4), Some(white_king));
        board.set_piece((3, 4), Some(black_king));

        // Make and unmake capture
        let mv = board.make_move((4, 4), (3, 4));
        board.unmake_move(mv);

        // Check both pieces restored
        assert_eq!(board.get_piece((4, 4)), Some(white_king), "White king should be restored");
        assert_eq!(board.get_piece((3, 4)), Some(black_king), "Captured piece should be restored");

        // Check side to move restored
        assert_eq!(board.side_to_move(), Color::White);
    }

    #[test]
    fn test_find_king() {
        let board = Board::setup_k_vs_qnc();

        // Find white king at e1 (row 7, col 4)
        assert_eq!(board.find_king(Color::White), Some((7, 4)));

        // Find black king at e8 (row 0, col 4)
        assert_eq!(board.find_king(Color::Black), Some((0, 4)));
    }

    #[test]
    fn test_king_not_in_check() {
        // K vs QNC setup: kings are far apart
        let board = Board::setup_k_vs_qnc();

        // Black king at e8 is not in check (QNC at d1 can't reach)
        assert!(!board.is_in_check(Color::Black));

        // White king at e1 is not attacked by black
        assert!(!board.is_in_check(Color::White));
    }

    #[test]
    fn test_king_in_check_by_qnc_queen_move() {
        let mut board = Board::new();

        // Black king at e8 (row 0, col 4)
        board.set_piece((0, 4), Some(Piece::new(PieceType::King, Color::Black)));

        // White QNC at e1 (row 7, col 4) - same file, Queen-like attack
        board.set_piece((7, 4), Some(Piece::new(PieceType::QNC, Color::White)));

        // Black king should be in check (QNC attacks on e-file)
        assert!(board.is_in_check(Color::Black));
    }

    #[test]
    fn test_king_in_check_by_qnc_knight_move() {
        let mut board = Board::new();

        // Black king at e4 (row 4, col 4)
        board.set_piece((4, 4), Some(Piece::new(PieceType::King, Color::Black)));

        // White QNC at f6 (row 2, col 5) - Knight-like attack (2,1)
        board.set_piece((2, 5), Some(Piece::new(PieceType::QNC, Color::White)));

        // Black king should be in check
        assert!(board.is_in_check(Color::Black));
    }

    #[test]
    fn test_king_in_check_by_qnc_camel_move() {
        let mut board = Board::new();

        // Black king at e4 (row 4, col 4)
        board.set_piece((4, 4), Some(Piece::new(PieceType::King, Color::Black)));

        // White QNC at f7 (row 1, col 5) - Camel-like attack (3,1)
        board.set_piece((1, 5), Some(Piece::new(PieceType::QNC, Color::White)));

        // Black king should be in check
        assert!(board.is_in_check(Color::Black));
    }

    #[test]
    fn test_king_in_check_by_enemy_king() {
        let mut board = Board::new();

        // Black king at e4 (row 4, col 4)
        board.set_piece((4, 4), Some(Piece::new(PieceType::King, Color::Black)));

        // White king at e5 (row 3, col 4) - adjacent
        board.set_piece((3, 4), Some(Piece::new(PieceType::King, Color::White)));

        // Both kings attack each other
        assert!(board.is_in_check(Color::Black));
        assert!(board.is_in_check(Color::White));
    }

    #[test]
    fn test_legal_moves_excludes_self_check() {
        let mut board = Board::new();

        // White king at e1 (row 7, col 4)
        board.set_piece((7, 4), Some(Piece::new(PieceType::King, Color::White)));

        // Black QNC at e8 (row 0, col 4) - controls e-file
        board.set_piece((0, 4), Some(Piece::new(PieceType::QNC, Color::Black)));

        // Black king somewhere safe
        board.set_piece((0, 0), Some(Piece::new(PieceType::King, Color::Black)));

        board.set_side_to_move(Color::White);
        let legal_moves = board.generate_legal_moves();

        // White king cannot stay on e-file (e2 would be check)
        // Can only move to d1, d2, f1, f2
        for mv in &legal_moves {
            assert_ne!(mv.to.1, 4, "King should not move to e-file (col 4)");
        }

        // Should have exactly 4 legal moves (d1, d2, f1, f2)
        assert_eq!(legal_moves.len(), 4);
    }

    #[test]
    fn test_legal_moves_king_safe() {
        let mut board = Board::new();

        // White king at e4 (row 4, col 4) - center of board
        board.set_piece((4, 4), Some(Piece::new(PieceType::King, Color::White)));

        // Black king far away
        board.set_piece((0, 0), Some(Piece::new(PieceType::King, Color::Black)));

        board.set_side_to_move(Color::White);
        let legal_moves = board.generate_legal_moves();

        // King at center should have 8 legal moves
        assert_eq!(legal_moves.len(), 8);
    }

    #[test]
    fn test_legal_moves_checkmate_position() {
        let mut board = Board::new();

        // Black king at a8 (row 0, col 0) - corner
        board.set_piece((0, 0), Some(Piece::new(PieceType::King, Color::Black)));

        // White king at a6 (row 2, col 0) - cuts off escape
        board.set_piece((2, 0), Some(Piece::new(PieceType::King, Color::White)));

        // White QNC at b6 (row 2, col 1) - gives check and covers escape squares
        board.set_piece((2, 1), Some(Piece::new(PieceType::QNC, Color::White)));

        board.set_side_to_move(Color::Black);

        // Black king is in check
        assert!(board.is_in_check(Color::Black));

        let legal_moves = board.generate_legal_moves();

        // Black king has no legal moves - checkmate!
        assert_eq!(legal_moves.len(), 0, "Should be checkmate with no legal moves");
    }
}
