use rand::prelude::*;
use crate::{square::Square, slider::Slider, bitboard::BitBoard};
// use lazy_static::lazy_static;

#[derive(Clone, Copy, Debug, Default)]
pub struct Magic {
    pub hash: u64,
    pub index_bits: usize, // 64 - shift
}

// lazy_static! {
//     // static ref BISHOP_RESULTS: ([Magic; Square::NUM], [Vec<BitBoard>; Square::NUM]) = find_bishop_magics();
//     // static ref ROOK_RESULTS: ([Magic; Square::NUM], [Vec<BitBoard>; Square::NUM]) = find_rook_magics();

//     // pub static ref BISHOP_MAGICS: [Magic; Square::NUM] = BISHOP_RESULTS.0;
//     // pub static ref ROOK_MAGICS: [Magic; Square::NUM] = ROOK_RESULTS.0;

//     // pub static ref BISHOP_MOVES: [Vec<BitBoard>; Square::NUM] = BISHOP_RESULTS.1.clone();
//     // pub static ref ROOK_MOVES: [Vec<BitBoard>; Square::NUM] = ROOK_RESULTS.1.clone();
//     // #[derive(Debug)]
//     pub static ref BISHOP_MOVES: [BitBoard; Square::NUM] = bishop_moves();
//     // #[derive(Debug)]
//     pub static ref ROOK_MOVES: [BitBoard; Square::NUM] = rook_moves();
// }

// generate u64 with low active bit count
fn low_u64(rng: &mut ThreadRng) -> u64 {
    rng.gen::<u64>() & rng.gen::<u64>() & rng.gen::<u64>()
}

fn subsets(mask: BitBoard) -> Vec<BitBoard> {
    let mut res: Vec<BitBoard> = Vec::with_capacity(2usize.pow(mask.len()));
    let mut subset = BitBoard::EMPTY;
    loop {
        res.push(subset);
        subset = BitBoard(subset.0.wrapping_sub(mask.0) & mask.0);
        if subset.is_empty() {
            break;
        }
    }
    res
}

pub fn magic_index(blockers: BitBoard, magic: Magic) -> usize {
    (blockers.0.wrapping_mul(magic.hash) >> magic.index_bits) as usize
}

struct HashCollision;

fn test_magic(magic: Magic, sq: Square, slider: Slider) -> Result<Vec<BitBoard>, HashCollision> {
    let blockers = slider.blockers(sq);
    let mut table: Vec<BitBoard> = vec![BitBoard::EMPTY; 2usize.pow(blockers.len())];
    for subset in subsets(blockers) {
        let moves = slider.pseudo_moves(sq, subset);
        let idx = magic_index(subset, magic);
        let entry = &mut table[idx];
        if entry.is_empty() {
            *entry = moves;
        } else if entry.0 != moves.0 {
            return Err(HashCollision);
        }
    }
    Ok(table)
}

fn find_magic(sq: Square, slider: Slider) -> (Magic, Vec<BitBoard>) {
    let mut rng = thread_rng();
    loop {
        let hash = low_u64(&mut rng);
        let magic = Magic {
            hash,
            index_bits: 64 - slider.blockers(sq).len() as usize,
        };
        if let Ok(table) = test_magic(magic, sq, slider) {
            return (magic, table);
        }
    }
}

fn find_bishop_magics() -> ([Magic; Square::NUM], [Vec<BitBoard>; Square::NUM]) {
    let mut table: [Vec<BitBoard>; Square::NUM] = vec![Vec::new(); Square::NUM].try_into().expect("static");
    let mut magics: [Magic; Square::NUM] = vec![Magic::default(); Square::NUM].try_into().expect("static");
    for i in 0..Square::NUM {
        let magic = find_magic(Square(i), Slider::Bishop);
        magics[i] = magic.0;
        table[i] = magic.1;
    }
    (magics, table)
}

fn find_rook_magics() -> ([Magic; Square::NUM], [Vec<BitBoard>; Square::NUM]) {
    let mut table: [Vec<BitBoard>; Square::NUM] = vec![Vec::new(); Square::NUM].try_into().expect("static");
    let mut magics: [Magic; Square::NUM] = vec![Magic::default(); Square::NUM].try_into().expect("static");
    for i in 0..Square::NUM {
        let magic = find_magic(Square(i), Slider::Rook);
        magics[i] = magic.0;
        table[i] = magic.1;
    }
    (magics, table)
}

fn bishop_moves() -> [BitBoard; Square::NUM] {
    let mut moves: [BitBoard; Square::NUM] = [BitBoard::EMPTY; Square::NUM];
    let slider = Slider::Bishop;
    for i in 0..Square::NUM {
        moves[i] = slider.pseudo_moves(Square(i), BitBoard::EMPTY);
    }
    moves
}

fn rook_moves() -> [BitBoard; Square::NUM] {
    let mut moves: [BitBoard; Square::NUM] = [BitBoard::EMPTY; Square::NUM];
    let slider = Slider::Rook;
    for i in 0..Square::NUM {
        moves[i] = slider.pseudo_moves(Square(i), BitBoard::EMPTY);
    }
    moves
}

// /// Pseudo-legal move generation using magics
// pub fn bishop_moves(sq: Square, blockers: BitBoard) -> BitBoard {
//     let magic = BISHOP_MAGICS[sq.0];
//     let idx = magic_index(blockers, magic);
//     BISHOP_MOVES[sq.0][idx]
// }

// /// Pseudo-legal move generation using magics
// pub fn rook_moves(sq: Square, blockers: BitBoard) -> BitBoard {
//     let magic = ROOK_MAGICS[sq.0];
//     let idx = magic_index(blockers, magic);
//     ROOK_MOVES[sq.0][idx]
// }