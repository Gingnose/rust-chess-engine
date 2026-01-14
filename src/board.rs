// Board representation and piece logic
// Using Mailbox (8x8 array) approach for clarity and extensibility

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
}
