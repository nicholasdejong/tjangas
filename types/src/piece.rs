#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Piece {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn
}

#[derive(Clone, Copy)]
pub enum PromotionPiece {
    Queen,
    Rook,
    Bishop,
    Knight
}

impl std::fmt::Display for PromotionPiece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let char = match self {
            Self::Queen => 'Q',
            Self::Rook => 'R',
            Self::Bishop => 'B',
            Self::Knight => 'N'
        };
        write!(f, "{char}")
    }
}