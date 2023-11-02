use std::time::SystemTime;
mod gamestate;
// mod magics;
// mod pseudo_legals;

use rand::prelude::*;
// use std::{env, fs, path::Path};
use types::{square::Square, slider::Slider, bitboard::BitBoard, sliders::common::{ROOK_BLOCKERS, BISHOP_BLOCKERS, BISHOP_SHIFTS, ROOK_MAGICS, BISHOP_MAGICS}};

// use crate::pseudo_legals::{BISHOP_MAGICS, ROOK_MAGICS, BISHOP_MOVES, ROOK_MOVES};


#[derive(Clone, Copy, Default)]
pub struct Magic {
    hash: u64,
    index_bits: usize, // 64 - shift
}

impl std::fmt::Debug for Magic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:x}", self.hash)
    }
}

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

#[derive(Debug)]
struct HashCollision;

fn try_fill_table(magic: Magic, sq: Square, slider: Slider) -> Result<Vec<BitBoard>, HashCollision> {
    let blockers = slider.blockers(sq);
    // println!("{}", 64 - magic.index_bits);
    let mut table: Vec<BitBoard> = vec![BitBoard::EMPTY; 2usize.pow(64 - magic.index_bits as u32)];
    // let mut table: Vec<BitBoard> = vec![BitBoard::EMPTY; 2usize.pow(blockers.len())];
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

fn find_magic(sq: Square, slider: Slider, shift_count: usize) -> Magic {
    let mut rng = thread_rng();
    let blockers = slider.blockers(sq);
    println!("{} {slider:?}",sq.0);
    let mut magic = Magic {index_bits: shift_count, ..Default::default()};
    loop {
        let hash = low_u64(&mut rng);
        if (blockers.0.wrapping_mul(hash) & 0xff00000000000000).count_ones() < 6 {
            continue;
        }
        magic.hash = hash;
        let result =  try_fill_table(magic, sq, slider);
        if let Ok(table) = result {
            // println!("Found magic for {slider:?} at {}", sq.0);
            // println!("Magic Efficiency: {}", list_efficiency(&table, BitBoard::EMPTY));
            return magic;
        }
    }
}

fn efficient_magic(sq: Square, slider: Slider) {
    let mut rng = thread_rng();
    // let mut magic = Magic {index_bits: 64 - /*slider.blockers(sq).len() as usize*/5, hash: 0xffedf9fd7cfcffff};
    let mut magic = Magic {index_bits: 64 - 4, ..Default::default()};
    let mut best: f32 = 1.;

    loop {
        if let Ok(table) = try_fill_table(magic, sq, slider) {
            let eff = list_efficiency(&table, BitBoard::EMPTY);
            if eff < best {
                best = eff;
                println!("New efficiency: {eff}, magic: 0x{:x}, shift: {}", magic.hash, magic.index_bits);
            }
        }
        let hash = low_u64(&mut rng);
        magic.hash = hash;
    }
}

fn list_efficiency<T:std::cmp::PartialEq>(list: &[T], empty: T) -> f32 {
    if list.is_empty() || list.len() == 1 && list[0] == empty {
        return 0.0
    }
    let mut idx = list.len() - 1;
    while idx > 0 {
        if list[idx] != empty {
            break;
        }
        idx -= 1;
    }
    return (idx as f32 + 1.) / list.len() as f32;
}


fn main() {
    // let out_dir = env::var_os("OUT_DIR").unwrap();
    // let dest_path = Path::new(&out_dir).join("magics.rs");
    // let my_vec = vec![1,2,3,4,0,0,0];
    let start = SystemTime::now();
    // let bishop_magics = &BISHOP_MAGICS;
    // let rook_magics = &ROOK_MAGICS;

    // let bishop_moves = &BISHOP_MOVES;
    // let rook_moves = &ROOK_MOVES;

    // println!("{:x}", rook_magics[1].hash);
    // let bishop_magics = {
    //     let mut magics: [Magic; Square::NUM] = [Magic::default(); Square::NUM];
    //     for i in vec![2,3,4,5,10,11,12,13,18,19,20,21,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,42,43,44,45,50,51,52,53,58,59,60,61] {
    //         magics[i] = find_magic(Square(i), Slider::Bishop, BISHOP_SHIFTS[i]);
    //         println!("0x{:x}", magics[i].hash);
    //     }
    //     magics
    // };

    // let rook_magics = {
    //     let mut magics: [Magic; Square::NUM] = [Magic::default(); Square::NUM];
    //     let mut i = 0;
    //     while i < Square::NUM {
    //         magics[i] = find_magic(Square(i), Slider::Rook);
    //         i += 1;
    //     }
    //     magics
    // };
    let rook_magics = ROOK_MAGICS.map(|x| Magic { hash: x, index_bits: 64 - 12 });

    let bishop_magics: Vec<Magic> = BISHOP_MAGICS.iter().enumerate().map(|(i, x)| Magic {hash: *x, index_bits: BISHOP_SHIFTS[i]}).collect();

    let bishop_moves = {
        let mut moves: [Vec<BitBoard>; Square::NUM] = vec![Vec::new(); Square::NUM].try_into().expect("static");
        let mut i = 0;
        while i < Square::NUM {
            moves[i] = try_fill_table(bishop_magics[i], Square(i), Slider::Bishop).unwrap();
            i += 1;
        }
        moves
    };
    let rook_moves = {
        let mut moves: [Vec<BitBoard>; Square::NUM] = vec![Vec::new(); Square::NUM].try_into().expect("static");
        let mut i = 0;
        while i < Square::NUM {
            moves[i] = try_fill_table(rook_magics[i], Square(i), Slider::Rook).unwrap();
            i += 1;
        }
        moves
    };
    let elapsed = start.elapsed();
    println!("Magic gen took {}s", elapsed.expect("time").as_secs());
    // println!("{bishop_magics:?}");
    // println!("{rook_magics:?}");
    let bishop = Slider::Bishop;
    let mut sum = 0;
    for sq in 0..Square::NUM {
        let blockers = BitBoard(BISHOP_BLOCKERS[sq] & 0x80808ff08080808);
        let all_moves = &*bishop_moves[sq];
        // size += std::mem::size_of_val(&all_moves[0..all_moves.len() / 4]);
        
        let size = std::mem::size_of_val(all_moves);
        sum += size;
        let moves = bishop_moves[sq][magic_index(blockers, bishop_magics[sq])];
        assert!(!moves.is_empty());
        assert_eq!(moves, bishop.pseudo_moves(Square(sq), blockers))
    }
    println!("Size: {sum}");
    // println!("Magic: 0x{:x}", rook_moves[35][magic_index(BitBoard(0x8002200000800), rook_magics[35])].0);
    // println!("Dumb7fill: 0x{:x}", rook.pseudo_moves(Square(35), BitBoard(0x8002200000800)).0);
    // println!("Blockers: 0x{:x}", ROOK_BLOCKERS[35]);
    // let nodes = 1_000_000_000;
    // let start = SystemTime::now();
    // let sq = thread_rng().gen_range(0..Square::NUM);
    // for _ in 0..nodes {
    //     let _ = rook_moves[sq][magic_index(BitBoard::EMPTY, rook_magics[sq])];
    // }
    // let elapsed = start.elapsed();
    // println!("Time: {}", elapsed.expect("time").as_nanos());
    // println!("{:?}", rook_magics[0]);
    // efficient_magic(Square(2), Slider::Bishop);
    // println!("{:?}", rook_moves[35][magic_index(BitBoard::EMPTY, rook_magics[35])]);
    // fs::write(
    //     &dest_path,
    //     format!("
    //     pub fn bishop_magics() -> [Magic; Square::NUM] {{ {bishop_magics:?} }}
    //     pub fn rook_magics() -> [Magic; Square::NUM] {{ {rook_magics:?} }}
        
    //     pub fn bishop_moves() -> [Vec<BitBoard>; Square::NUM] {{ {bishop_moves:?} }}
    //     pub fn rook_moves() -> [Vec<BitBoard>; Square::NUM] {{ {rook_moves:?} }}
    //     ")
    // ).unwrap();

    // println!("cargo:rerun-if-changed=build.rs");
}