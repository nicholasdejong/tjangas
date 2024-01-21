use thiserror::Error;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Color {
    #[default]
    White,
    Black,
}

impl std::ops::Not for Color {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[derive(Error, Debug)]
#[error("Invalid color provided")]
pub struct ColorParseError;

impl std::str::FromStr for Color {
    type Err = ColorParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "w" | "W" | "White" | "white" | "WHITE" => Ok(Color::White),
            "b" | "B" | "Black" | "black" | "BLACK" => Ok(Color::Black),
            _ => Err(ColorParseError)
        }
    }
}