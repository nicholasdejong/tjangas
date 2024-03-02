use crate::{square::Square, color::Color};

#[derive(Clone, Copy, PartialEq, Default)]
pub struct BitBoard(pub u64);

impl BitBoard {
    pub const EMPTY: BitBoard = BitBoard(0);
    pub const FULL: BitBoard = BitBoard(!0);
    pub const NOT_A: BitBoard = BitBoard(0xfefefefefefefefe);
    pub const NOT_H: BitBoard = BitBoard(0x7f7f7f7f7f7f7f7f);

    pub const fn len(&self) -> u32 {
        self.0.count_ones()
    }

    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn display(&self) -> String {
        if self.is_empty() {
            String::from("0")
        } else {
            format!("0x{:x}", self.0)
        }
    }

    pub const fn shl(&self, other: usize) -> BitBoard {
        BitBoard(self.0 << other)
    }

    pub const fn shr(&self, other: usize) -> BitBoard {
        BitBoard(self.0 >> other)
    }

    /// Returns zero if the bitboard is empty or full and one otherwise
    pub const fn signum(&self) -> usize {
        1 - (self.0.wrapping_sub(1) >> 63) as usize
    }

    /// Shifts left if White and right otherwise
    pub const fn shift_color(&self, bits: usize, color: Color) -> BitBoard {
        match color {
            Color::White => self.shl(bits),
            Color::Black => self.shr(bits)
        }
    }
}

impl std::ops::Not for BitBoard {
    type Output = Self;
    fn not(self) -> Self::Output {
        BitBoard(!self.0)
    }
}

macro_rules! impl_bit_ops {
    ($($t:ident::$f:ident),*) => {
        $(impl std::ops::$t for BitBoard {
            type Output = Self;
            fn $f(self, rhs: Self) -> Self::Output {
                BitBoard(std::ops::$t::$f(self.0, rhs.0))
            }
        })*
    };
}

impl_bit_ops!(BitAnd::bitand, BitOr::bitor, BitXor::bitxor);

macro_rules! impl_bit_assign {
    ($($t:ident::$f:ident),*) => {
        $(impl std::ops::$t for BitBoard {
            fn $f(&mut self, rhs: Self) {
                std::ops::$t::$f(&mut self.0, rhs.0);
            }
        })*
    };
}

impl_bit_assign!(BitAndAssign::bitand_assign, BitOrAssign::bitor_assign, BitXorAssign::bitxor_assign);

impl std::fmt::Debug for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl std::fmt::Display for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mask = BitBoard(0xff << 56); // Eighth rank
        let mut str = String::new();
        for i in 0..8 {
            let rank = (self.0 & mask.shr(8 * i).0) >> 8 * (7 - i);
            let rank_str = format!("{rank:0>8b}\n").chars().rev().collect::<String>().replace("0", ".").replace("1", "x");
            str += rank_str.as_str();
        }
        write!(f, "{str}")
    }
}

pub struct BitBoardIterator {
    bitboard: u64
}

impl std::iter::IntoIterator for BitBoard {
    type Item = Square;
    type IntoIter = BitBoardIterator;
    fn into_iter(self) -> Self::IntoIter {
        BitBoardIterator { bitboard: self.0 }
    }
}

impl Iterator for BitBoardIterator {
    type Item = Square;
    fn next(&mut self) -> Option<Self::Item> {
        if self.bitboard > 0 {
            let sq =  Some(Square(self.bitboard.trailing_zeros() as usize));
            self.bitboard &= self.bitboard - 1;
            return sq;
        }
        None
    }
}
