use std::{env, fs, path::Path};
use types::{
    bitboard::BitBoard,
    slider::Slider,
    sliders::{common::*, magic::*},
    square::Square,
};

fn bishop_table() -> [BitBoard; BISHOP_SIZE] {
    let mut table = [BitBoard::EMPTY; BISHOP_SIZE];
    for sq in 0..Square::NUM {
        let blockers = BitBoard(BISHOP_BLOCKERS[sq]);
        let magic = BISHOP_MAGICS[sq];
        for subset in subsets(blockers) {
            let idx = magic_index(subset, magic, BISHOP_SHIFTS[sq]) + BISHOP_OFFSETS[sq];
            table[idx] = Slider::Bishop.pseudo_moves(Square(sq), subset);
        }
    }
    table
}

fn rook_table() -> [BitBoard; ROOK_SIZE] {
    let mut table = [BitBoard::EMPTY; ROOK_SIZE];
    for sq in 0..Square::NUM {
        let blockers = BitBoard(ROOK_BLOCKERS[sq]);
        let magic = ROOK_MAGICS[sq];
        for subset in subsets(blockers) {
            let idx = magic_index(subset, magic, ROOK_SHIFTS[sq]) + ROOK_OFFSETS[sq];
            table[idx] = Slider::Rook.pseudo_moves(Square(sq), subset);
        }
    }
    table
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("slider_moves.rs");

    fs::write(
        dest_path,
        format!(
            "pub const fn get_bishop_table() -> [u64; {BISHOP_SIZE}] {{ {:?} }}

            pub const fn get_rook_table() -> [u64; {ROOK_SIZE}] {{ {:?} }}
        ",
            bishop_table(), rook_table()
        ),
    )
    .unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}
