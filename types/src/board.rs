use crate::bitboard::BitBoard;

#[derive(Clone, Copy, Debug)]
pub struct Board {
    pub white: [BitBoard; 6], // kqrbnp
    pub black: [BitBoard; 6],
}