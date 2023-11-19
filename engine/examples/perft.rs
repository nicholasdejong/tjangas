use engine::board::Board;

fn main() {
    let brd = Board::default();
    assert_eq!(brd.moves().iter().map(|m| m.len()).sum::<u32>(), 20);
}