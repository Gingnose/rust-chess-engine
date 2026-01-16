// Board representation and piece logic
// Using Mailbox (8x8 array) approach for clarity and extensibility

use crate::pieces::amazon::AmazonMoves;
use crate::pieces::king::KingMoves;
use crate::pieces::rook::RookMoves;

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
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PieceType {
    King,
    /// Amazon = Queen + Knight
    /// Moves like Queen (sliding) and Knight (2,1 jump)
    Amazon,
    /// Rook - moves horizontally and vertically
    Rook,
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
    /// History of position hashes for repetition detection
    position_history: Vec<u64>,
}

impl Board {
    /// Create an empty board
    pub fn new() -> Self {
        Board {
            squares: [[None; 8]; 8],
            side_to_move: Color::White,
            position_history: Vec::new(),
        }
    }

    /// Compute a hash of the current position for repetition detection
    /// Uses a simple hash combining piece positions and side to move
    pub fn position_hash(&self) -> u64 {
        let mut hash: u64 = 0;

        // Hash each piece on the board
        for row in 0..8u8 {
            for col in 0..8u8 {
                if let Some(piece) = self.get_piece((row, col)) {
                    // Create a unique value for each piece type, color, and position
                    let piece_value: u64 = match piece.piece_type {
                        PieceType::King => 1,
                        PieceType::Amazon => 2,
                        PieceType::Rook => 3,
                    };
                    let color_value: u64 = match piece.color {
                        Color::White => 0,
                        Color::Black => 64,
                    };
                    let square_value = (row as u64) * 8 + (col as u64);

                    // Combine into hash using prime multiplier
                    hash ^= (piece_value + color_value) * 31 + square_value * 127;
                    hash = hash.wrapping_mul(0x517cc1b727220a95);
                }
            }
        }

        // Include side to move in hash
        if self.side_to_move == Color::Black {
            hash ^= 0xF0F0F0F0F0F0F0F0;
        }

        hash
    }

    /// Check if the current position has occurred before (repetition)
    pub fn is_repetition(&self) -> bool {
        let current_hash = self.position_hash();
        self.position_history.iter().filter(|&&h| h == current_hash).count() >= 1
    }

    /// Count how many times the current position has occurred
    pub fn repetition_count(&self) -> usize {
        let current_hash = self.position_hash();
        self.position_history.iter().filter(|&&h| h == current_hash).count()
    }

    /// Clear position history (e.g., when starting a new game)
    pub fn clear_history(&mut self) {
        self.position_history.clear();
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

    /// Setup the Amazon + K vs R + K starting position
    /// White: Amazon on d1, King on e1
    /// Black: Rook on a8, King on e8
    pub fn setup_amazon_vs_rook() -> Self {
        let mut board = Board::new();

        // Black Rook on a8 (row 0, col 0)
        board.set_piece((0, 0), Some(Piece::new(PieceType::Rook, Color::Black)));

        // Black King on e8 (row 0, col 4)
        board.set_piece((0, 4), Some(Piece::new(PieceType::King, Color::Black)));

        // White Amazon on d1 (row 7, col 3)
        board.set_piece((7, 3), Some(Piece::new(PieceType::Amazon, Color::White)));

        // White King on e1 (row 7, col 4)
        board.set_piece((7, 4), Some(Piece::new(PieceType::King, Color::White)));

        board.side_to_move = Color::White;
        board
    }

    /// Create a board from FEN notation
    /// FEN format: "r3k3/8/8/8/8/8/8/3AK3 w - - 0 1"
    /// Supported pieces: K/k (King), A/a (Amazon), R/r (Rook), Q/q (Queen as Amazon)
    pub fn from_fen(fen: &str) -> Option<Self> {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let mut board = Board::new();

        // Parse piece placement (first part)
        let ranks: Vec<&str> = parts[0].split('/').collect();
        if ranks.len() != 8 {
            return None;
        }

        for (row, rank_str) in ranks.iter().enumerate() {
            let mut col = 0usize;
            for c in rank_str.chars() {
                if col >= 8 {
                    break;
                }
                match c {
                    '1'..='8' => {
                        // Empty squares
                        col += c.to_digit(10).unwrap() as usize;
                    }
                    'K' => {
                        board.set_piece((row as u8, col as u8), Some(Piece::new(PieceType::King, Color::White)));
                        col += 1;
                    }
                    'k' => {
                        board.set_piece((row as u8, col as u8), Some(Piece::new(PieceType::King, Color::Black)));
                        col += 1;
                    }
                    'A' | 'Q' => {
                        // Amazon (or Queen treated as Amazon for compatibility)
                        board.set_piece((row as u8, col as u8), Some(Piece::new(PieceType::Amazon, Color::White)));
                        col += 1;
                    }
                    'a' | 'q' => {
                        board.set_piece((row as u8, col as u8), Some(Piece::new(PieceType::Amazon, Color::Black)));
                        col += 1;
                    }
                    'R' => {
                        board.set_piece((row as u8, col as u8), Some(Piece::new(PieceType::Rook, Color::White)));
                        col += 1;
                    }
                    'r' => {
                        board.set_piece((row as u8, col as u8), Some(Piece::new(PieceType::Rook, Color::Black)));
                        col += 1;
                    }
                    _ => {
                        // Unknown piece, skip
                        col += 1;
                    }
                }
            }
        }

        // Parse side to move (second part)
        if parts.len() > 1 {
            board.side_to_move = match parts[1] {
                "w" | "W" => Color::White,
                "b" | "B" => Color::Black,
                _ => Color::White,
            };
        }

        // Ignore castling, en passant, halfmove clock, and fullmove number for now

        Some(board)
    }

    /// Convert board to FEN notation
    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        // Piece placement
        for row in 0..8 {
            let mut empty_count = 0;
            for col in 0..8 {
                match self.squares[row][col] {
                    None => {
                        empty_count += 1;
                    }
                    Some(piece) => {
                        if empty_count > 0 {
                            fen.push_str(&empty_count.to_string());
                            empty_count = 0;
                        }
                        let c = match piece.piece_type {
                            PieceType::King => 'K',
                            PieceType::Amazon => 'A',
                            PieceType::Rook => 'R',
                        };
                        if piece.color == Color::Black {
                            fen.push(c.to_ascii_lowercase());
                        } else {
                            fen.push(c);
                        }
                    }
                }
            }
            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
            }
            if row < 7 {
                fen.push('/');
            }
        }

        // Side to move
        fen.push(' ');
        fen.push(if self.side_to_move == Color::White { 'w' } else { 'b' });

        // Simplified: no castling, no en passant
        fen.push_str(" - - 0 1");

        fen
    }

    /// Execute a move, returns the Move with captured piece info for unmake
    pub fn make_move(&mut self, from: Square, to: Square) -> Move {
        // Save current position hash to history before making move
        let hash = self.position_hash();
        self.position_history.push(hash);

        let captured = self.get_piece(to);
        let piece = self.get_piece(from);

        self.set_piece(to, piece);
        self.set_piece(from, None);
        self.side_to_move = self.side_to_move.opposite();

        Move { from, to, captured }
    }

    /// Undo a move, restoring the previous state
    pub fn unmake_move(&mut self, mv: Move) {
        // Remove the position hash that was added when this move was made
        self.position_history.pop();

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
                            PieceType::Amazon => AmazonMoves::generate_moves(self, from),
                            PieceType::Rook => RookMoves::generate_moves(self, from),
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
                            PieceType::Amazon => AmazonMoves::generate_moves(self, from),
                            PieceType::Rook => RookMoves::generate_moves(self, from),
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

    /// Check if the given color is in checkmate
    pub fn is_checkmate(&mut self, color: Color) -> bool {
        if !self.is_in_check(color) {
            return false; // Not in check, can't be checkmate
        }

        // Save and set correct side to move
        let original_side = self.side_to_move;
        self.side_to_move = color;

        let has_no_moves = self.generate_legal_moves().is_empty();

        self.side_to_move = original_side;
        has_no_moves
    }

    /// Check if the given color is in stalemate
    pub fn is_stalemate(&mut self, color: Color) -> bool {
        if self.is_in_check(color) {
            return false; // In check, can't be stalemate
        }

        let original_side = self.side_to_move;
        self.side_to_move = color;

        let has_no_moves = self.generate_legal_moves().is_empty();

        self.side_to_move = original_side;
        has_no_moves
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
                            PieceType::Amazon => 'A', // A for Amazon
                            PieceType::Rook => 'R',
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

        let white_amazon = Piece::new(PieceType::Amazon, Color::White);
        assert_eq!(white_amazon.piece_type, PieceType::Amazon);
        assert_eq!(white_amazon.color, Color::White);
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
    fn test_amazon_vs_rook_setup() {
        let board = Board::setup_amazon_vs_rook();

        // Black Rook on a8 (row 0, col 0)
        let black_rook = board.get_piece((0, 0)).expect("There should be a piece at (0, 0)");
        assert_eq!(black_rook.piece_type, PieceType::Rook, "Piece at (0, 0) should be a Rook");
        assert_eq!(black_rook.color, Color::Black, "Rook at (0, 0) should be Black");

        // Black King on e8 (row 0, col 4)
        let black_king = board.get_piece((0, 4)).expect("There should be a piece at (0, 4)");
        assert_eq!(black_king.piece_type, PieceType::King, "Piece at (0, 4) should be a King");
        assert_eq!(black_king.color, Color::Black, "King at (0, 4) should be Black");

        // White Amazon on d1 (row 7, col 3)
        let white_amazon = board.get_piece((7, 3)).expect("There should be a piece at (7, 3)");
        assert_eq!(white_amazon.piece_type, PieceType::Amazon, "Piece at (7, 3) should be Amazon");
        assert_eq!(white_amazon.color, Color::White, "Amazon at (7, 3) should be White");

        // White King on e1 (row 7, col 4)
        let white_king = board.get_piece((7, 4)).expect("There should be a piece at (7, 4)");
        assert_eq!(white_king.piece_type, PieceType::King, "Piece at (7, 4) should be a King");
        assert_eq!(white_king.color, Color::White, "King at (7, 4) should be White");

        // Side to move is White
        assert_eq!(board.side_to_move(), Color::White);
    }

    #[test]
    fn test_board_display() {
        let board = Board::setup_amazon_vs_rook();
        let display = format!("{}", board);

        // Check that the display contains expected elements
        assert!(display.contains("a b c d e f g h"));
        assert!(display.contains("k")); // Black king (lowercase)
        assert!(display.contains("K")); // White king (uppercase)
        assert!(display.contains("A")); // White Amazon (uppercase)
        assert!(display.contains("r")); // Black rook (lowercase)
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
        let board = Board::setup_amazon_vs_rook();

        // Find white king at e1 (row 7, col 4)
        assert_eq!(board.find_king(Color::White), Some((7, 4)));

        // Find black king at e8 (row 0, col 4)
        assert_eq!(board.find_king(Color::Black), Some((0, 4)));
    }

    #[test]
    fn test_king_not_in_check() {
        // Amazon vs Rook setup: kings are far apart
        let board = Board::setup_amazon_vs_rook();

        // Black king at e8 is not in check (Amazon at d1 can't reach)
        assert!(!board.is_in_check(Color::Black));

        // White king at e1 is not attacked by black
        assert!(!board.is_in_check(Color::White));
    }

    #[test]
    fn test_king_in_check_by_amazon_queen_move() {
        let mut board = Board::new();

        // Black king at e8 (row 0, col 4)
        board.set_piece((0, 4), Some(Piece::new(PieceType::King, Color::Black)));

        // White Amazon at e1 (row 7, col 4) - same file, Queen-like attack
        board.set_piece((7, 4), Some(Piece::new(PieceType::Amazon, Color::White)));

        // Black king should be in check (Amazon attacks on e-file)
        assert!(board.is_in_check(Color::Black));
    }

    #[test]
    fn test_king_in_check_by_amazon_knight_move() {
        let mut board = Board::new();

        // Black king at e4 (row 4, col 4)
        board.set_piece((4, 4), Some(Piece::new(PieceType::King, Color::Black)));

        // White Amazon at f6 (row 2, col 5) - Knight-like attack (2,1)
        board.set_piece((2, 5), Some(Piece::new(PieceType::Amazon, Color::White)));

        // Black king should be in check
        assert!(board.is_in_check(Color::Black));
    }

    #[test]
    fn test_king_in_check_by_rook() {
        let mut board = Board::new();

        // Black king at e4 (row 4, col 4)
        board.set_piece((4, 4), Some(Piece::new(PieceType::King, Color::Black)));

        // White Rook at e1 (row 7, col 4) - same file attack
        board.set_piece((7, 4), Some(Piece::new(PieceType::Rook, Color::White)));

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

        // Black Amazon at e8 (row 0, col 4) - controls e-file
        board.set_piece((0, 4), Some(Piece::new(PieceType::Amazon, Color::Black)));

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

        // White Amazon at b6 (row 2, col 1) - gives check and covers escape squares
        board.set_piece((2, 1), Some(Piece::new(PieceType::Amazon, Color::White)));

        board.set_side_to_move(Color::Black);

        // Black king is in check
        assert!(board.is_in_check(Color::Black));

        let legal_moves = board.generate_legal_moves();

        // Black king has no legal moves - checkmate!
        assert_eq!(legal_moves.len(), 0, "Should be checkmate with no legal moves");
    }

    #[test]
    fn test_is_checkmate() {
        let mut board = Board::new();

        // Black king at a8 (row 0, col 0) - corner
        board.set_piece((0, 0), Some(Piece::new(PieceType::King, Color::Black)));

        // White king at a6 (row 2, col 0) - cuts off escape
        board.set_piece((2, 0), Some(Piece::new(PieceType::King, Color::White)));

        // White Amazon at b6 (row 2, col 1) - gives check and covers escape squares
        board.set_piece((2, 1), Some(Piece::new(PieceType::Amazon, Color::White)));

        // Black is in checkmate
        assert!(board.is_checkmate(Color::Black));

        // White is not in checkmate
        assert!(!board.is_checkmate(Color::White));
    }

    #[test]
    fn test_is_not_checkmate_when_not_in_check() {
        let mut board = Board::new();

        // White king at e1
        board.set_piece((7, 4), Some(Piece::new(PieceType::King, Color::White)));

        // Black king far away
        board.set_piece((0, 0), Some(Piece::new(PieceType::King, Color::Black)));

        // Neither side is in checkmate
        assert!(!board.is_checkmate(Color::White));
        assert!(!board.is_checkmate(Color::Black));
    }

    #[test]
    fn test_is_stalemate() {
        let mut board = Board::new();

        // Stalemate position with Amazon + K vs K:
        // Black king at a8 (row 0, col 0)
        // White king at b6 (row 2, col 1) - controls a7, b7
        // White Amazon at d7 (row 1, col 3) - controls b8 via knight move
        //   d7 to a8: (-1, -3) = NOT Amazon move (Amazon has no camel)
        //   d7 to b8: (-1, -2) = knight move, blocks b8
        //
        // Black king at a8 escape squares:
        //   a7 -> blocked by White king
        //   b7 -> blocked by White king
        //   b8 -> blocked by Amazon (knight move)
        //
        // Black king at a8 has no legal moves = stalemate!

        board.set_piece((0, 0), Some(Piece::new(PieceType::King, Color::Black))); // a8
        board.set_piece((2, 1), Some(Piece::new(PieceType::King, Color::White))); // b6
        board.set_piece((1, 3), Some(Piece::new(PieceType::Amazon, Color::White))); // d7

        // Verify: black king at a8 is NOT in check
        assert!(!board.is_in_check(Color::Black), "Black should NOT be in check for stalemate");

        // Black is in stalemate (not in check, but no legal moves)
        assert!(board.is_stalemate(Color::Black));

        // Black is NOT in checkmate
        assert!(!board.is_checkmate(Color::Black));
    }

    #[test]
    fn test_not_stalemate_with_legal_moves() {
        let mut board = Board::new();

        // White king at e4 (center, has moves)
        board.set_piece((4, 4), Some(Piece::new(PieceType::King, Color::White)));

        // Black king far away
        board.set_piece((0, 0), Some(Piece::new(PieceType::King, Color::Black)));

        // Neither side is in stalemate
        assert!(!board.is_stalemate(Color::White));
        assert!(!board.is_stalemate(Color::Black));
    }

    #[test]
    fn test_from_fen_starting_position() {
        let fen = "r3k3/8/8/8/8/8/8/3AK3 w - - 0 1";
        let board = Board::from_fen(fen).expect("FEN should parse");

        // Black Rook on a8 (row 0, col 0)
        let black_rook = board.get_piece((0, 0)).expect("Should have piece at a8");
        assert_eq!(black_rook.piece_type, PieceType::Rook);
        assert_eq!(black_rook.color, Color::Black);

        // Black King on e8 (row 0, col 4)
        let black_king = board.get_piece((0, 4)).expect("Should have piece at e8");
        assert_eq!(black_king.piece_type, PieceType::King);
        assert_eq!(black_king.color, Color::Black);

        // White Amazon on d1 (row 7, col 3)
        let white_amazon = board.get_piece((7, 3)).expect("Should have piece at d1");
        assert_eq!(white_amazon.piece_type, PieceType::Amazon);
        assert_eq!(white_amazon.color, Color::White);

        // White King on e1 (row 7, col 4)
        let white_king = board.get_piece((7, 4)).expect("Should have piece at e1");
        assert_eq!(white_king.piece_type, PieceType::King);
        assert_eq!(white_king.color, Color::White);

        // White to move
        assert_eq!(board.side_to_move(), Color::White);
    }

    #[test]
    fn test_from_fen_black_to_move() {
        let fen = "r3k3/8/8/8/8/8/8/3AK3 b - - 0 1";
        let board = Board::from_fen(fen).expect("FEN should parse");
        assert_eq!(board.side_to_move(), Color::Black);
    }

    #[test]
    fn test_to_fen() {
        let board = Board::setup_amazon_vs_rook();
        let fen = board.to_fen();
        assert_eq!(fen, "r3k3/8/8/8/8/8/8/3AK3 w - - 0 1");
    }

    #[test]
    fn test_fen_roundtrip() {
        let original = Board::setup_amazon_vs_rook();
        let fen = original.to_fen();
        let restored = Board::from_fen(&fen).expect("Should parse own FEN");

        // Verify pieces match
        for row in 0..8 {
            for col in 0..8 {
                assert_eq!(
                    original.get_piece((row, col)),
                    restored.get_piece((row, col)),
                    "Piece mismatch at ({}, {})",
                    row,
                    col
                );
            }
        }
        assert_eq!(original.side_to_move(), restored.side_to_move());
    }
}
