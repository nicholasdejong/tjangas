#[derive(Clone, Copy, Debug)]
pub enum Color {
    White,
    Black
}

impl std::ops::Not for Color {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White
        }
    }
}