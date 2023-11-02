use crate::{square::Square, bitboard::BitBoard, sliders::{common::*, dumb7fill::{bishop_moves,rook_moves}}};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Slider {
    Bishop,
    Rook,
}

impl Slider {
    pub const fn blockers(&self, sq: Square) -> BitBoard {
        match self {
            Self::Bishop => BitBoard(BISHOP_BLOCKERS[sq.0]),
            Self::Rook => BitBoard(ROOK_BLOCKERS[sq.0])
        }
    }

    /// Pseudo-legal move generation using dumb7fill
    pub const fn pseudo_moves(&self, sq: Square, blockers: BitBoard) -> BitBoard {
        match self {
            Self::Bishop => BitBoard(bishop_moves(sq.0, blockers.0)),
            Self::Rook => BitBoard(rook_moves(sq.0, blockers.0))
        }
    }

    pub const fn shift_count(&self) -> usize {
        match self {
            Self::Bishop => 9,
            Self::Rook => 12
        }
    }
}