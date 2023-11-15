use crate::{bitboard::BitBoard, square::Square};

const NOT_H: u64 = 0x7f7f7f7f7f7f7f7f;
const NOT_A: u64 = 0xfefefefefefefefe;
const NOT_EDGE: u64 = 0x7e7e7e7e7e7e00;

pub const BISHOP_BLOCKERS: [BitBoard; Square::NUM] = populate_bishop_blockers();
pub const ROOK_BLOCKERS: [BitBoard; Square::NUM] = populate_rook_blockers();

const fn sout_attacks(mut pieces: u64, empty: u64, end: usize) -> u64 {
    let mut i = 0;
    while i < end {
        pieces |= (pieces >> 8) & empty;
        i += 1;
    }
    pieces >> 8
}
const fn nort_attacks(mut pieces: u64, empty: u64, end: usize) -> u64 {
    let mut i = 0;
    while i < end {
        pieces |= (pieces << 8) & empty;
        i += 1;
    }
    pieces << 8
}
const fn east_attacks(mut pieces: u64, mut empty: u64, end: usize) -> u64 {
    empty &= NOT_A;
    let mut i = 0;
    while i < end {
        pieces |= (pieces << 1) & empty;
        i += 1;
    }
    (pieces << 1) & NOT_A
}
const fn west_attacks(mut pieces: u64, mut empty: u64, end: usize) -> u64 {
    empty &= NOT_H;
    let mut i = 0;
    while i < end {
        pieces |= (pieces >> 1) & empty;
        i += 1;
    }
    (pieces >> 1) & NOT_H
}
const fn noea_attacks(mut pieces: u64, mut empty: u64) -> u64 {
    empty &= NOT_A;
    let mut i = 0;
    while i < 6 {
        pieces |= (pieces << 9) & empty;
        i += 1;
    }
    (pieces << 9) & NOT_A
}
const fn nowe_attacks(mut pieces: u64, mut empty: u64) -> u64 {
    empty &= NOT_H;
    let mut i = 0;
    while i < 6 {
        pieces |= (pieces << 7) & empty;
        i += 1;
    }
    (pieces << 7) & NOT_H
}
const fn soea_attacks(mut pieces: u64, mut empty: u64) -> u64 {
    empty &= NOT_A;
    let mut i = 0;
    while i < 6 {
        pieces |= (pieces >> 7) & empty;
        i += 1;
    }
    (pieces >> 7) & NOT_A
}
const fn sowe_attacks(mut pieces: u64, mut empty: u64) -> u64 {
    empty &= NOT_H;
    let mut i = 0;
    while i < 6 {
        pieces |= (pieces >> 9) & empty;
        i += 1;
    }
    (pieces >> 9) & NOT_H
}

pub const fn bishop_moves(bb: u64, blockers: u64) -> u64 {
    let empty = !blockers;
    nowe_attacks(bb, empty)
        | noea_attacks(bb, empty)
        | soea_attacks(bb, empty)
        | sowe_attacks(bb, empty)
}

pub const fn rook_moves(bb: u64, blockers: u64) -> u64 {
    let empty = !blockers;
    nort_attacks(bb, empty, 6)
        | sout_attacks(bb, empty, 6)
        | west_attacks(bb, empty, 6)
        | east_attacks(bb, empty, 6)
}

const fn populate_bishop_blockers() -> [BitBoard; Square::NUM] {
    let mut i = 0;
    let mut blockers = [BitBoard::EMPTY; Square::NUM];
    while i < Square::NUM {
        blockers[i] = BitBoard(bishop_moves(1 << i, 0) & NOT_EDGE);
        i += 1;
    }
    blockers
}

const fn populate_rook_blockers() -> [BitBoard; Square::NUM] {
    let mut i = 0;
    let mut blockers = [BitBoard::EMPTY; Square::NUM];
    while i < Square::NUM {
        blockers[i] = BitBoard(
            (0xffffffffffffff & nort_attacks(1 << i, !0, 5))
                | (0xffffffffffffff00 & sout_attacks(1 << i, !0, 5))
                | (0xfefefefefefefefe & west_attacks(1 << i, !0, 5))
                | (0x7f7f7f7f7f7f7f7f & east_attacks(1 << i, !0, 5)),
        );
        i += 1;
    }
    blockers
}
