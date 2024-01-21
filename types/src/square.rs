use crate::bitboard::BitBoard;

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

#[cfg(test)]
mod tests {
    use crate::square::Square;
    #[test]
    fn test_square() {
        assert_eq!(format!("{}", Square(42)), String::from("c6"));
    }
}