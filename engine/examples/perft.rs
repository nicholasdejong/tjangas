use std::str::FromStr;
use std::env;
use engine::board::Board;
use types::moves::Move;
use types::piece::Piece;
use types::square::Square;

fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() >= 8, "Please provide valid FEN and depth");
    let fen = &args[1..7].join(" ");
    let depth = args[7].parse::<usize>().expect("Invalid depth");
    let mut brd = Board::from_str(fen).expect("Invalid FEN");

    // brd.apply_move(&Move {
    //     piece: Piece::Knight,
    //     from: Square(14),
    //     to: Square(30),
    //     flags: None
    // });

    fn perft(brd: &Board, depth: usize) -> usize {
        let mut brd = *brd;
        let mut nodes: usize = 0;
        for piece_moves in brd.moves() {
            // dbg!(&piece_moves.moves);
            for mv in piece_moves.convert(brd.turn) {
                // println!("{:?}: {}->{}", mv.piece, mv.from, mv.to);
                brd.apply_move(&mv);
                let children_nodes = perft_children(&mut brd, depth - 1);
                println!("{mv}({}->{}): {}",mv.from.0, mv.to.0, children_nodes);
                nodes += children_nodes;
                brd.undo_move(&mv);
            }
        }
        fn perft_children(brd: &mut Board, depth: usize) -> usize {
            if depth == 1 {
                return brd.moves().iter().map(|m| m.len() as usize).sum();
            } else 
            if depth == 0 {
                return 1;
            }
            let mut sum: usize = 0;
            for piece_moves in brd.moves() {
                for mv in piece_moves.convert(brd.turn) {
                    brd.apply_move(&mv);
                    let nodes = perft_children(brd, depth - 1);
                    sum += nodes;
                    brd.undo_move(&mv);
                }
            }
            sum
        }
        // perft(&mut brd, depth)
        nodes
    }

    let nodes = perft(&brd, depth);
    println!("\nNodes searched: {nodes}");


    // let count = brd.moves().iter().map(|m| m.len()).sum::<u32>();
    // println!("Nodes: {count}");
}
