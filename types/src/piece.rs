#[derive(Clone, Copy, Debug)]
pub enum Piece {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn
}

pub enum Promotion {
    Queen,
    Rook,
    Bishop,
    Knight
}