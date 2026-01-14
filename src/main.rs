use rust_chess_engine::board::Board;

fn main() {
    println!("=== Rust Chess Engine ===");
    println!("Board Representation: Mailbox (8x8 array)");
    println!();

    // Create K vs QNC starting position
    let board = Board::setup_k_vs_qnc();

    println!("K vs QNC Starting Position:");
    println!("{}", board);

    println!("Legend:");
    println!("  K/k = King (White/Black)");
    println!("  A/a = QNC/Actress (White/Black)");
    println!("  .   = Empty square");
}
