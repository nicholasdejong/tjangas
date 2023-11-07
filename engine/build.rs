use std::{env, fs, path::Path};
use types::{
    bitboard::BitBoard,
    slider::Slider,
    sliders::{common::*, magic::*},
    square::Square,
};

fn populate_table() -> [BitBoard; TABLE_SIZE] {
    let mut table = [BitBoard::EMPTY; TABLE_SIZE];
    // bishop
    for sq in 0..Square::NUM {
        let blockers = Slider::Bishop.blockers(Square(sq));
        for subset in subsets(blockers) {
            let idx = magic_index(subset, BISHOP_MAGICS[sq], 55) + BISHOP_OFFSETS[sq];
            table[idx] = Slider::Bishop.pseudo_moves(Square(sq), subset);
        }
    }
    // rook
    for sq in 0..Square::NUM {
        let blockers = Slider::Rook.blockers(Square(sq));
        for subset in subsets(blockers) {
            let idx = magic_index(subset, ROOK_MAGICS[sq], 52) + ROOK_OFFSETS[sq];
            table[idx] = Slider::Rook.pseudo_moves(Square(sq), subset);
        }
    }
    table
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("slider_moves.rs");

    fs::write(
        &dest_path,
        format!(
            "pub const fn get_table() -> [u64; {TABLE_SIZE}] {{ {:?} }}
        ",
            populate_table()
        ),
    )
    .unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}
