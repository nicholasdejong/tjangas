use std::str::FromStr;
use std::env;
use engine::board::Board;

fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() >= 8, "Please provide valid FEN and depth");
    let fen = &args[1..7].join(" ");
    let depth = args[7].parse::<usize>().expect("Invalid depth");
    let brd = Board::from_str(fen).expect("Invalid FEN");

    fn perft(brd: &Board, depth: usize) -> usize {
        let mut brd = *brd;
        let mut nodes = 0;
        if depth == 0 {
            return 1;
        }
        for piece_moves in brd.moves() {
            for mv in piece_moves.convert(brd.turn) {
                brd.apply_move(&mv);
                nodes += perft(&mut brd, depth - 1); 
                brd.undo_move(&mv);            
            }
        }
        nodes
    }

    let nodes = perft(&brd, depth);
    println!("\nNodes searched: {nodes}");
}
