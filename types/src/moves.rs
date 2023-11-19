use crate::{piece::{Piece, Promotion}, square::Square, bitboard::BitBoard};

pub enum MoveFlags {
    Promotion(Promotion)
}

#[derive(Debug)]
pub struct PieceMoves {
    pub piece: Piece,
    pub from: Square,
    pub moves: BitBoard
}

impl PieceMoves {
    pub fn len(&self) -> u32 {
        self.moves.len()
    }
}