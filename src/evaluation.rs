// Evaluation function for Amazon + K vs R + K endgame
// Goal: Push enemy King to corner/edge and deliver checkmate

use crate::board::{Board, Color, Square};

// Score constants
const CHECKMATE_SCORE: i32 = 100_000;
const CHECK_BONUS: i32 = 50;
const KING_PROXIMITY_WEIGHT: i32 = 10;

/// Piece-Square Table for enemy King position
/// Higher values = better for the attacker (King pushed to edge/corner)
/// Center = 0, Edge = 2-3, Corner = 4
const ENEMY_KING_PST: [[i32; 8]; 8] = [
    [4, 3, 3, 3, 3, 3, 3, 4],
    [3, 2, 2, 2, 2, 2, 2, 3],
    [3, 2, 1, 1, 1, 1, 2, 3],
    [3, 2, 1, 0, 0, 1, 2, 3],
    [3, 2, 1, 0, 0, 1, 2, 3],
    [3, 2, 1, 1, 1, 1, 2, 3],
    [3, 2, 2, 2, 2, 2, 2, 3],
    [4, 3, 3, 3, 3, 3, 3, 4],
];

/// Evaluate the position from the perspective of the given color
/// Positive score = good for `for_color`
/// Negative score = bad for `for_color`
pub fn evaluate(board: &mut Board, for_color: Color) -> i32 {
    let enemy_color = for_color.opposite();

    // 1. Terminal state detection
    // Check if enemy is checkmated (we win!)
    if board.is_checkmate(enemy_color) {
        return CHECKMATE_SCORE;
    }

    // Check if enemy is stalemated (draw - avoid this!)
    if board.is_stalemate(enemy_color) {
        return 0;
    }

    // Check if we are checkmated (we lose!)
    if board.is_checkmate(for_color) {
        return -CHECKMATE_SCORE;
    }

    // Check if we are stalemated (draw)
    if board.is_stalemate(for_color) {
        return 0;
    }

    let mut score = 0;

    // 2. Enemy King position (pushed to edge/corner is good)
    if let Some(enemy_king_sq) = board.find_king(enemy_color) {
        score += evaluate_enemy_king_position(enemy_king_sq);
    }

    // 3. Check bonus (giving check is good - keeps pressure)
    if board.is_in_check(enemy_color) {
        score += CHECK_BONUS;
    }

    // 4. King proximity (our King closer to enemy King helps with mating)
    if let (Some(our_king_sq), Some(enemy_king_sq)) =
        (board.find_king(for_color), board.find_king(enemy_color))
    {
        score += evaluate_king_proximity(our_king_sq, enemy_king_sq);
    }

    score
}

/// Evaluate enemy King position using Piece-Square Table
/// Higher score = King is pushed toward edge/corner (good for attacker)
fn evaluate_enemy_king_position(square: Square) -> i32 {
    let (row, col) = square;
    ENEMY_KING_PST[row as usize][col as usize] * 100 // Scale for significance
}

/// Evaluate King proximity
/// Closer Kings = better for the attacker (helps with mating net)
fn evaluate_king_proximity(our_king: Square, enemy_king: Square) -> i32 {
    let row_diff = (our_king.0 as i32 - enemy_king.0 as i32).abs();
    let col_diff = (our_king.1 as i32 - enemy_king.1 as i32).abs();

    // Chebyshev distance (max of row/col difference)
    let distance = row_diff.max(col_diff);

    // Closer is better: invert distance (max distance is 7, so 7 - distance)
    (7 - distance) * KING_PROXIMITY_WEIGHT
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Piece, PieceType};

    #[test]
    fn test_checkmate_score() {
        let mut board = Board::new();

        // Checkmate position: Black king at a8, White king at a6, White Amazon at b6
        board.set_piece((0, 0), Some(Piece::new(PieceType::King, Color::Black)));
        board.set_piece((2, 0), Some(Piece::new(PieceType::King, Color::White)));
        board.set_piece((2, 1), Some(Piece::new(PieceType::Amazon, Color::White)));

        // From White's perspective, this is checkmate (max score)
        let score = evaluate(&mut board, Color::White);
        assert_eq!(score, CHECKMATE_SCORE);

        // From Black's perspective, this is being checkmated (negative max score)
        let score_black = evaluate(&mut board, Color::Black);
        assert_eq!(score_black, -CHECKMATE_SCORE);
    }

    #[test]
    fn test_stalemate_score() {
        let mut board = Board::new();

        // Stalemate position from previous test
        board.set_piece((0, 7), Some(Piece::new(PieceType::King, Color::Black))); // h8
        board.set_piece((2, 7), Some(Piece::new(PieceType::King, Color::White))); // h6
        board.set_piece((2, 4), Some(Piece::new(PieceType::Amazon, Color::White))); // e6

        // Stalemate = draw = 0
        let score = evaluate(&mut board, Color::White);
        assert_eq!(score, 0);
    }

    #[test]
    fn test_king_corner_better_than_center() {
        let mut board_corner = Board::new();
        let mut board_center = Board::new();

        // Setup 1: Black king in corner (a8)
        board_corner.set_piece((0, 0), Some(Piece::new(PieceType::King, Color::Black)));
        board_corner.set_piece((7, 4), Some(Piece::new(PieceType::King, Color::White)));
        board_corner.set_piece((7, 3), Some(Piece::new(PieceType::Amazon, Color::White)));

        // Setup 2: Black king in center (d4)
        board_center.set_piece((4, 3), Some(Piece::new(PieceType::King, Color::Black)));
        board_center.set_piece((7, 4), Some(Piece::new(PieceType::King, Color::White)));
        board_center.set_piece((7, 3), Some(Piece::new(PieceType::Amazon, Color::White)));

        let score_corner = evaluate(&mut board_corner, Color::White);
        let score_center = evaluate(&mut board_center, Color::White);

        // King in corner should be higher score for White
        assert!(
            score_corner > score_center,
            "Corner ({}) should be better than center ({})",
            score_corner,
            score_center
        );
    }

    #[test]
    fn test_check_bonus() {
        let mut board_check = Board::new();
        let mut board_no_check = Board::new();

        // Setup 1: Black king in check
        board_check.set_piece((0, 4), Some(Piece::new(PieceType::King, Color::Black)));
        board_check.set_piece((7, 4), Some(Piece::new(PieceType::King, Color::White)));
        board_check.set_piece((4, 4), Some(Piece::new(PieceType::Amazon, Color::White))); // On same file

        // Setup 2: Black king not in check
        board_no_check.set_piece((0, 4), Some(Piece::new(PieceType::King, Color::Black)));
        board_no_check.set_piece((7, 4), Some(Piece::new(PieceType::King, Color::White)));
        board_no_check.set_piece((7, 3), Some(Piece::new(PieceType::Amazon, Color::White))); // Not attacking

        let score_check = evaluate(&mut board_check, Color::White);
        let score_no_check = evaluate(&mut board_no_check, Color::White);

        // Check position should have higher score
        assert!(
            score_check > score_no_check,
            "Check ({}) should give bonus over no check ({})",
            score_check,
            score_no_check
        );
    }

    #[test]
    fn test_king_proximity() {
        let mut board_close = Board::new();
        let mut board_far = Board::new();

        // Setup 1: Kings close together
        board_close.set_piece((0, 0), Some(Piece::new(PieceType::King, Color::Black)));
        board_close.set_piece((2, 2), Some(Piece::new(PieceType::King, Color::White))); // Close
        board_close.set_piece((7, 7), Some(Piece::new(PieceType::Amazon, Color::White)));

        // Setup 2: Kings far apart
        board_far.set_piece((0, 0), Some(Piece::new(PieceType::King, Color::Black)));
        board_far.set_piece((7, 7), Some(Piece::new(PieceType::King, Color::White))); // Far
        board_far.set_piece((7, 6), Some(Piece::new(PieceType::Amazon, Color::White)));

        let score_close = evaluate(&mut board_close, Color::White);
        let score_far = evaluate(&mut board_far, Color::White);

        // Closer Kings should give higher score
        assert!(
            score_close > score_far,
            "Close kings ({}) should be better than far kings ({})",
            score_close,
            score_far
        );
    }
}
