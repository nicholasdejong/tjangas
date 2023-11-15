use engine::board::Board;
use std::time::SystemTime;

fn main() {
    let brd = Board::default();
    let now = SystemTime::now();
    for _ in 0..100_000_000 {
        let moves = brd.moves();
        let _ = moves;
    }
    let elapsed = now.elapsed().unwrap().as_micros() as f64 / 1_000_000.0;
    let nps = 100_000_000.0 / elapsed;
    println!("NPS: {nps:.0}");
}