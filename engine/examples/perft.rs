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

    brd.apply_move(&Move {
        piece: Piece::Bishop,
        from: Square(54),
        to: Square(45),
        flags: None
    });

    fn perft(brd: &mut Board, depth: usize) -> usize {
        if depth == 1 {
            return brd.moves().iter().map(|m| m.len() as usize).sum();
        }
        let mut sum: usize = 0;
        for piece_moves in brd.moves() {
            for mv in piece_moves.convert(brd.turn) {
                brd.apply_move(&mv);
                let nodes = perft(brd, depth - 1);
                println!("{mv}: {nodes}");
                sum += nodes;
                brd.undo_move(&mv);
            }
        }
        sum
    }

    let nodes = perft(&mut brd, depth);
    println!("\nNodes searched: {nodes}");


    // let count = brd.moves().iter().map(|m| m.len()).sum::<u32>();
    // println!("Nodes: {count}");
}