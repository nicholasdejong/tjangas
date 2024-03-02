use crate::bitboard::BitBoard;
use std::error::Error;
use thiserror::Error;

#[derive(Clone, Copy, Debug)]
pub struct Square(pub usize);

impl Square {
    pub const NUM: usize = 64;

    pub const fn file(&self) -> usize {
        self.0 & 7
    }

    pub const fn rank(&self) -> usize {
        self.0 / 8
    }

    pub const fn offset(&self, df: i8, dr: i8) -> Square {
        let file = self.file() as i8 + df;
        let rank = self.rank() as i8 + dr;
        if file < 0 || rank < 0 || file > 7 || rank > 7 {
            *self
        } else {
            Square(rank as usize * 8 + file as usize)
        }
    }

    pub const fn bitboard(&self) -> BitBoard {
        BitBoard(1 << self.0)
    }
}

impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        assert!(self.0 <= 63, "Invalid Square");
        write!(f, "{}{}", ((self.0 as u8 & 7) + 97) as char, (self.0 as u8 / 8 + 49) as char)
    }
}

#[derive(Debug, Error)]
pub enum SquareParseError {
    #[error("Expected 2 characters, found {0}.")]
    CharLenError(usize),
    #[error("The square file is out of bounds.")]
    FileError,
    #[error("The square rank is out of bounds.")]
    RankError,
}

impl std::str::FromStr for Square {
    type Err = Box<dyn Error>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err(Box::new(SquareParseError::CharLenError(s.len())));
        }
        let mut chars = s.chars();
        let file = chars.next().ok_or(SquareParseError::FileError)?.to_digit(10).ok_or(SquareParseError::FileError)?;
        let rank = chars.next().ok_or(SquareParseError::RankError)?.to_digit(10).ok_or(SquareParseError::RankError)?;

        if file < 97 || file > 104 {
            return Err(Box::new(SquareParseError::FileError));
        }
        if rank == 0 || rank > 8 {
            return Err(Box::new(SquareParseError::RankError));
        }
        let file = file - 97;
        let rank = rank - 1;
        Ok(Square(rank as usize * 8 + file as usize))
    }
}

#[cfg(test)]
mod tests {
    use crate::square::Square;
    #[test]
    fn test_square() {
        assert_eq!(format!("{}", Square(42)), String::from("c6"));
    }
}
