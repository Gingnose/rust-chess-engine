// Search algorithm for finding the best move
// Uses Negamax with Alpha-Beta pruning

use crate::board::{Board, Color, Move, PieceType, Square};

// Score constants
const CHECKMATE_SCORE: i32 = 100_000;
const INFINITY: i32 = i32::MAX;

// Material values
const AMAZON_VALUE: i32 = 1500;  // Very powerful piece (Q + N)
const ROOK_VALUE: i32 = 500;

// Positional weights
const CHECK_BONUS: i32 = 30;
const KING_PROXIMITY_WEIGHT: i32 = 5;
const AMAZON_CENTER_BONUS: i32 = 20;
const PIECE_SAFETY_PENALTY: i32 = 50;

// New evaluation weights
const TROPISM_WEIGHT: i32 = 15;       // Amazon approaching enemy king
const MOBILITY_WEIGHT: i32 = 3;        // Per legal move bonus
const KING_CUTOFF_BONUS: i32 = 40;     // King cutting off escape routes
const ROOK_TRAPPED_BONUS: i32 = 30;    // Bonus for trapping enemy rook
const MATING_NET_WEIGHT: i32 = 25;     // Mating net evaluation

/// Piece-Square Table for enemy King position
/// Higher values = better for the attacker (King pushed to edge/corner)
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

/// Piece-Square Table for Amazon (center is better)
const AMAZON_PST: [[i32; 8]; 8] = [
    [0, 1, 1, 2, 2, 1, 1, 0],
    [1, 2, 3, 3, 3, 3, 2, 1],
    [1, 3, 4, 5, 5, 4, 3, 1],
    [2, 3, 5, 6, 6, 5, 3, 2],
    [2, 3, 5, 6, 6, 5, 3, 2],
    [1, 3, 4, 5, 5, 4, 3, 1],
    [1, 2, 3, 3, 3, 3, 2, 1],
    [0, 1, 1, 2, 2, 1, 1, 0],
];

// =============================================================================
// Evaluation Function
// =============================================================================

/// Evaluate the position from the perspective of the side to move
/// Positive score = good for side to move
pub fn evaluate(board: &mut Board) -> i32 {
    let for_color = board.side_to_move();
    let enemy_color = for_color.opposite();

    // 1. Terminal state detection
    if board.is_checkmate(enemy_color) {
        return CHECKMATE_SCORE;
    }
    if board.is_stalemate(enemy_color) {
        return 0; // Draw - avoid this!
    }
    if board.is_checkmate(for_color) {
        return -CHECKMATE_SCORE;
    }
    if board.is_stalemate(for_color) {
        return 0;
    }

    let mut score = 0;

    // 2. Material evaluation (MOST IMPORTANT!)
    score += evaluate_material(board, for_color);

    // 3. Piece safety - penalize pieces under attack
    score += evaluate_piece_safety(board, for_color);

    // 4. Amazon position (center is better)
    score += evaluate_amazon_position(board, for_color);

    // 5. Enemy King position (pushed to edge/corner is good)
    if let Some(enemy_king_sq) = board.find_king(enemy_color) {
        score += evaluate_enemy_king_position(enemy_king_sq);
    }

    // 6. Check bonus (smaller now since material is more important)
    if board.is_in_check(enemy_color) {
        score += CHECK_BONUS;
    }

    // 7. King proximity (for endgame)
    if let (Some(our_king_sq), Some(enemy_king_sq)) =
        (board.find_king(for_color), board.find_king(enemy_color))
    {
        score += evaluate_king_proximity(our_king_sq, enemy_king_sq);
    }

    // 8. Amazon Tropism - Amazon closer to enemy king
    score += evaluate_amazon_tropism(board, for_color);

    // 9. Mobility - more legal moves is better
    score += evaluate_mobility(board, for_color);

    // 10. King Cut-off - cutting enemy king's escape routes
    score += evaluate_king_cutoff(board, for_color);

    // 11. Rook Activity - penalize active enemy rook
    score += evaluate_rook_activity(board, for_color);

    // 12. Mating Distance - how close to checkmate position
    score += evaluate_mating_distance(board, for_color);

    score
}

/// Evaluate material balance
fn evaluate_material(board: &Board, for_color: Color) -> i32 {
    let mut our_material = 0;
    let mut enemy_material = 0;

    for row in 0..8 {
        for col in 0..8 {
            if let Some(piece) = board.get_piece((row, col)) {
                let value = match piece.piece_type {
                    PieceType::Amazon => AMAZON_VALUE,
                    PieceType::Rook => ROOK_VALUE,
                    PieceType::King => 0, // King has no material value
                };
                if piece.color == for_color {
                    our_material += value;
                } else {
                    enemy_material += value;
                }
            }
        }
    }

    our_material - enemy_material
}

/// Evaluate piece safety - penalize pieces that are attacked
fn evaluate_piece_safety(board: &Board, for_color: Color) -> i32 {
    let mut penalty = 0;
    let enemy_color = for_color.opposite();

    for row in 0..8 {
        for col in 0..8 {
            if let Some(piece) = board.get_piece((row, col)) {
                if piece.color == for_color && piece.piece_type != PieceType::King {
                    let square = (row, col);
                    // If our piece is attacked, apply penalty
                    if board.is_square_attacked(square, enemy_color) {
                        // Penalty based on piece value
                        let piece_value = match piece.piece_type {
                            PieceType::Amazon => AMAZON_VALUE / 10,
                            PieceType::Rook => ROOK_VALUE / 10,
                            PieceType::King => 0,
                        };
                        penalty -= piece_value + PIECE_SAFETY_PENALTY;
                    }
                }
            }
        }
    }

    penalty
}

/// Evaluate Amazon position using PST
fn evaluate_amazon_position(board: &Board, for_color: Color) -> i32 {
    let mut score = 0;

    for row in 0..8 {
        for col in 0..8 {
            if let Some(piece) = board.get_piece((row, col)) {
                if piece.piece_type == PieceType::Amazon {
                    let pst_value = AMAZON_PST[row as usize][col as usize] * AMAZON_CENTER_BONUS;
                    if piece.color == for_color {
                        score += pst_value;
                    } else {
                        score -= pst_value;
                    }
                }
            }
        }
    }

    score
}

fn evaluate_enemy_king_position(square: Square) -> i32 {
    let (row, col) = square;
    ENEMY_KING_PST[row as usize][col as usize] * 50  // Reduced weight
}

fn evaluate_king_proximity(our_king: Square, enemy_king: Square) -> i32 {
    let row_diff = (our_king.0 as i32 - enemy_king.0 as i32).abs();
    let col_diff = (our_king.1 as i32 - enemy_king.1 as i32).abs();
    let distance = row_diff.max(col_diff);
    (7 - distance) * KING_PROXIMITY_WEIGHT
}

/// Find Amazon position for a given color
fn find_amazon(board: &Board, color: Color) -> Option<Square> {
    for row in 0..8u8 {
        for col in 0..8u8 {
            if let Some(piece) = board.get_piece((row, col)) {
                if piece.piece_type == PieceType::Amazon && piece.color == color {
                    return Some((row, col));
                }
            }
        }
    }
    None
}

/// Find Rook position for a given color
fn find_rook(board: &Board, color: Color) -> Option<Square> {
    for row in 0..8u8 {
        for col in 0..8u8 {
            if let Some(piece) = board.get_piece((row, col)) {
                if piece.piece_type == PieceType::Rook && piece.color == color {
                    return Some((row, col));
                }
            }
        }
    }
    None
}

/// Evaluate Amazon Tropism - Amazon closer to enemy king is better
fn evaluate_amazon_tropism(board: &Board, for_color: Color) -> i32 {
    let enemy_color = for_color.opposite();

    let amazon_sq = find_amazon(board, for_color);
    let enemy_king_sq = board.find_king(enemy_color);

    if let (Some(amazon), Some(king)) = (amazon_sq, enemy_king_sq) {
        // Chebyshev distance (max of row/col difference)
        let row_diff = (amazon.0 as i32 - king.0 as i32).abs();
        let col_diff = (amazon.1 as i32 - king.1 as i32).abs();
        let distance = row_diff.max(col_diff);

        // Closer = higher score (max distance is 7, so 7 - distance gives 0-7)
        return (7 - distance) * TROPISM_WEIGHT;
    }

    0
}

/// Evaluate Mobility - more legal moves is better
fn evaluate_mobility(board: &mut Board, for_color: Color) -> i32 {
    let current_side = board.side_to_move();

    // If it's our turn, count our moves
    if current_side == for_color {
        let our_moves = board.generate_legal_moves().len() as i32;
        return our_moves * MOBILITY_WEIGHT;
    }

    // Otherwise, we need to temporarily switch sides to count
    // But this is expensive, so we'll just use 0 for now
    0
}

/// Evaluate King Cut-off - our king cutting off enemy king's escape routes
fn evaluate_king_cutoff(board: &Board, for_color: Color) -> i32 {
    let enemy_color = for_color.opposite();

    let our_king_sq = board.find_king(for_color);
    let enemy_king_sq = board.find_king(enemy_color);

    if let (Some(our_king), Some(enemy_king)) = (our_king_sq, enemy_king_sq) {
        let mut bonus = 0;

        // Check if our king cuts off the enemy king on the same file
        if our_king.1 == enemy_king.1 {
            // Same file - check if we're between enemy king and center/other side
            let our_dist_to_edge = our_king.0.min(7 - our_king.0);
            let enemy_dist_to_edge = enemy_king.0.min(7 - enemy_king.0);
            if our_dist_to_edge > enemy_dist_to_edge {
                bonus += KING_CUTOFF_BONUS;
            }
        }

        // Check if our king cuts off the enemy king on the same rank
        if our_king.0 == enemy_king.0 {
            // Same rank - check if we're between enemy king and center/other side
            let our_dist_to_edge = our_king.1.min(7 - our_king.1);
            let enemy_dist_to_edge = enemy_king.1.min(7 - enemy_king.1);
            if our_dist_to_edge > enemy_dist_to_edge {
                bonus += KING_CUTOFF_BONUS;
            }
        }

        // Bonus if kings are close (opposition can be useful)
        let row_diff = (our_king.0 as i32 - enemy_king.0 as i32).abs();
        let col_diff = (our_king.1 as i32 - enemy_king.1 as i32).abs();
        if row_diff <= 2 && col_diff <= 2 {
            bonus += KING_CUTOFF_BONUS / 2;
        }

        return bonus;
    }

    0
}

/// Evaluate Rook Activity - penalize enemy rook that has many moves
fn evaluate_rook_activity(board: &Board, for_color: Color) -> i32 {
    let enemy_color = for_color.opposite();

    let enemy_rook_sq = find_rook(board, enemy_color);

    if let Some(rook) = enemy_rook_sq {
        // Count how many squares the rook can move to (simplified)
        let mut rook_mobility = 0;

        // Check horizontal moves
        for col in 0..8u8 {
            if col != rook.1 {
                let sq = (rook.0, col);
                if board.get_piece(sq).is_none() {
                    rook_mobility += 1;
                } else {
                    break; // Blocked
                }
            }
        }

        // Check vertical moves
        for row in 0..8u8 {
            if row != rook.0 {
                let sq = (row, rook.1);
                if board.get_piece(sq).is_none() {
                    rook_mobility += 1;
                } else {
                    break; // Blocked
                }
            }
        }

        // Less mobility for enemy rook = better for us
        // Max rook mobility is 14 (7 + 7)
        return (14 - rook_mobility) * (ROOK_TRAPPED_BONUS / 7);
    }

    0
}

/// Evaluate Mating Distance - how close are we to a mating position
fn evaluate_mating_distance(board: &Board, for_color: Color) -> i32 {
    let enemy_color = for_color.opposite();

    let our_king_sq = board.find_king(for_color);
    let enemy_king_sq = board.find_king(enemy_color);
    let amazon_sq = find_amazon(board, for_color);

    if let (Some(our_king), Some(enemy_king), Some(amazon)) =
        (our_king_sq, enemy_king_sq, amazon_sq)
    {
        // Distance of enemy king to nearest corner
        let corner_dist = [
            enemy_king.0.max(enemy_king.1),                    // Distance to (0,0)
            enemy_king.0.max(7 - enemy_king.1),                // Distance to (0,7)
            (7 - enemy_king.0).max(enemy_king.1),              // Distance to (7,0)
            (7 - enemy_king.0).max(7 - enemy_king.1),          // Distance to (7,7)
        ]
        .into_iter()
        .min()
        .unwrap_or(7) as i32;

        // Distance from our pieces to enemy king
        let amazon_dist = (amazon.0 as i32 - enemy_king.0 as i32)
            .abs()
            .max((amazon.1 as i32 - enemy_king.1 as i32).abs());
        let our_king_dist = (our_king.0 as i32 - enemy_king.0 as i32)
            .abs()
            .max((our_king.1 as i32 - enemy_king.1 as i32).abs());

        // Score: enemy king close to corner + our pieces close to enemy king
        let corner_score = (7 - corner_dist) * 2;
        let approach_score = 14 - amazon_dist - our_king_dist;

        return (corner_score + approach_score) * MATING_NET_WEIGHT / 10;
    }

    0
}

// =============================================================================
// Move Ordering (for better Alpha-Beta pruning)
// =============================================================================

/// Get the material value of a piece type
fn piece_value(piece_type: PieceType) -> i32 {
    match piece_type {
        PieceType::Amazon => AMAZON_VALUE,
        PieceType::Rook => ROOK_VALUE,
        PieceType::King => 10000, // King is invaluable
    }
}

/// Score a move for ordering purposes
/// Higher score = should be searched first
fn score_move(board: &Board, mv: &Move) -> i32 {
    let mut score = 0;

    // 1. Captures are very important - use MVV-LVA
    //    (Most Valuable Victim - Least Valuable Attacker)
    if let Some(captured) = mv.captured {
        // Value of captured piece minus a fraction of attacker value
        let victim_value = piece_value(captured.piece_type);
        
        // Get attacker piece type
        if let Some(attacker) = board.get_piece(mv.from) {
            let attacker_value = piece_value(attacker.piece_type);
            // MVV-LVA: prioritize capturing valuable pieces with less valuable pieces
            score += 10000 + victim_value - attacker_value / 100;
        } else {
            score += 10000 + victim_value;
        }
    }

    score
}

/// Order moves for better Alpha-Beta pruning efficiency
/// Captures are searched first (MVV-LVA ordering)
fn order_moves(board: &Board, moves: Vec<Move>) -> Vec<Move> {
    let mut scored_moves: Vec<(Move, i32)> = moves
        .into_iter()
        .map(|mv| {
            let score = score_move(board, &mv);
            (mv, score)
        })
        .collect();

    // Sort in descending order (highest score first)
    scored_moves.sort_by(|a, b| b.1.cmp(&a.1));

    scored_moves.into_iter().map(|(mv, _)| mv).collect()
}

// =============================================================================
// Quiescence Search (to avoid horizon effect)
// =============================================================================

/// Quiescence search - continue searching captures at depth 0
/// This prevents the "horizon effect" where the engine stops searching
/// right before a major tactical change (like a piece being captured)
fn quiescence(board: &mut Board, mut alpha: i32, beta: i32) -> i32 {
    // "Stand pat" - evaluate the current position
    let stand_pat = evaluate(board);

    // If standing pat is good enough, we can prune
    if stand_pat >= beta {
        return beta;
    }

    // Update alpha if stand pat is better
    if stand_pat > alpha {
        alpha = stand_pat;
    }

    // Generate only capture moves
    let all_moves = board.generate_legal_moves();
    let captures: Vec<Move> = all_moves
        .into_iter()
        .filter(|mv| mv.captured.is_some())
        .collect();

    // If no captures, return the stand pat score
    if captures.is_empty() {
        return stand_pat;
    }

    // Order captures (MVV-LVA)
    let ordered_captures = order_moves(board, captures);

    for mv in ordered_captures {
        board.make_move(mv.from, mv.to);
        let score = -quiescence(board, -beta, -alpha);
        board.unmake_move(mv);

        if score >= beta {
            return beta; // Beta cutoff
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

// =============================================================================
// Search Algorithm: Negamax with Alpha-Beta Pruning
// =============================================================================

/// Negamax search with Alpha-Beta pruning
/// Returns the score of the position from the side to move's perspective
pub fn negamax(board: &mut Board, depth: i32, mut alpha: i32, beta: i32) -> i32 {
    // Base case: reached maximum depth - use quiescence search
    if depth == 0 {
        return quiescence(board, alpha, beta);
    }

    let moves = board.generate_legal_moves();

    // No legal moves = checkmate or stalemate
    if moves.is_empty() {
        if board.is_in_check(board.side_to_move()) {
            // Checkmate - return negative score (we lose)
            // Add depth to prefer faster checkmates
            return -CHECKMATE_SCORE + (100 - depth);
        } else {
            // Stalemate - draw
            return 0;
        }
    }

    // Order moves for better pruning (captures first)
    let ordered_moves = order_moves(board, moves);

    let mut best_score = -INFINITY;

    for mv in ordered_moves {
        board.make_move(mv.from, mv.to);
        let score = -negamax(board, depth - 1, -beta, -alpha);
        board.unmake_move(mv);

        best_score = best_score.max(score);
        alpha = alpha.max(score);

        if alpha >= beta {
            break; // Beta cutoff (pruning)
        }
    }

    best_score
}

/// Find the best move for the current position
/// Returns the best move and its score
pub fn find_best_move(board: &mut Board, depth: i32) -> Option<(Move, i32)> {
    let moves = board.generate_legal_moves();

    if moves.is_empty() {
        return None;
    }

    // Order moves for better pruning
    let ordered_moves = order_moves(board, moves);

    let mut best_move = None;
    let mut best_score = -INFINITY;
    let mut alpha = -INFINITY;
    let beta = INFINITY;

    for mv in ordered_moves {
        board.make_move(mv.from, mv.to);
        let score = -negamax(board, depth - 1, -beta, -alpha);
        board.unmake_move(mv);

        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }
        alpha = alpha.max(score);
    }

    best_move.map(|mv| (mv, best_score))
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Color, Piece, PieceType};

    #[test]
    fn test_find_checkmate_in_one() {
        let mut board = Board::new();

        // Position where White can checkmate in 1
        // Black king at a8 (row 0, col 0) - corner
        // White king at c7 (row 1, col 2) - cuts off b8, b7
        // White Amazon at d4 (row 4, col 3) - can move to b6 for checkmate
        board.set_piece((0, 0), Some(Piece::new(PieceType::King, Color::Black)));
        board.set_piece((1, 2), Some(Piece::new(PieceType::King, Color::White)));
        board.set_piece((4, 3), Some(Piece::new(PieceType::Amazon, Color::White)));
        board.set_side_to_move(Color::White);

        // Verify this is NOT already checkmate
        assert!(
            !board.is_checkmate(Color::Black),
            "Should not be checkmate yet"
        );

        let result = find_best_move(&mut board, 4);
        assert!(result.is_some(), "Should find a move");

        let (best_move, score) = result.unwrap();

        // Score should be very high (checkmate found)
        assert!(
            score > CHECKMATE_SCORE - 1000,
            "Should find checkmate, score: {}, move: {:?}",
            score,
            best_move
        );
    }

    #[test]
    fn test_avoid_stalemate() {
        let mut board = Board::new();

        // Position where White must avoid stalemate
        // If White plays wrong, it's stalemate
        // Black king at h8 (row 0, col 7)
        // White king at h6 (row 2, col 7)
        // White Amazon at f5 (row 3, col 5) - should NOT block all escape squares
        board.set_piece((0, 7), Some(Piece::new(PieceType::King, Color::Black)));
        board.set_piece((2, 7), Some(Piece::new(PieceType::King, Color::White)));
        board.set_piece((3, 5), Some(Piece::new(PieceType::Amazon, Color::White)));
        board.set_side_to_move(Color::White);

        let result = find_best_move(&mut board, 3);
        assert!(result.is_some(), "Should find a move");

        let (best_move, score) = result.unwrap();

        // Apply the move
        board.make_move(best_move.from, best_move.to);

        // Should NOT be stalemate
        assert!(
            !board.is_stalemate(Color::Black),
            "Should avoid stalemate, move: {:?}, score: {}",
            best_move,
            score
        );
    }

    #[test]
    fn test_search_returns_move() {
        let mut board = Board::setup_amazon_vs_rook();

        let result = find_best_move(&mut board, 3);
        assert!(result.is_some(), "Should find a move in starting position");

        let (mv, _score) = result.unwrap();

        // Move should be valid (within board)
        assert!(mv.from.0 < 8 && mv.from.1 < 8);
        assert!(mv.to.0 < 8 && mv.to.1 < 8);
    }

    #[test]
    fn test_no_moves_returns_none() {
        let mut board = Board::new();

        // Checkmate position - no legal moves for Black
        board.set_piece((0, 0), Some(Piece::new(PieceType::King, Color::Black)));
        board.set_piece((2, 0), Some(Piece::new(PieceType::King, Color::White)));
        board.set_piece((2, 1), Some(Piece::new(PieceType::Amazon, Color::White)));
        board.set_side_to_move(Color::Black);

        let result = find_best_move(&mut board, 3);
        assert!(result.is_none(), "Should return None when no legal moves");
    }

    #[test]
    fn test_evaluation_prefers_corner() {
        let mut board_corner = Board::new();
        let mut board_center = Board::new();

        // Black king in corner
        board_corner.set_piece((0, 0), Some(Piece::new(PieceType::King, Color::Black)));
        board_corner.set_piece((7, 4), Some(Piece::new(PieceType::King, Color::White)));
        board_corner.set_piece((7, 3), Some(Piece::new(PieceType::Amazon, Color::White)));
        board_corner.set_side_to_move(Color::White);

        // Black king in center
        board_center.set_piece((4, 4), Some(Piece::new(PieceType::King, Color::Black)));
        board_center.set_piece((7, 4), Some(Piece::new(PieceType::King, Color::White)));
        board_center.set_piece((7, 3), Some(Piece::new(PieceType::Amazon, Color::White)));
        board_center.set_side_to_move(Color::White);

        let score_corner = evaluate(&mut board_corner);
        let score_center = evaluate(&mut board_center);

        assert!(
            score_corner > score_center,
            "Corner ({}) should be better than center ({})",
            score_corner,
            score_center
        );
    }
}
