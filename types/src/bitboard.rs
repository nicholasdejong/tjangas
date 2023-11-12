use crate::square::Square;

#[derive(Clone, Copy, PartialEq, Default)]
pub struct BitBoard(pub u64);

impl BitBoard {
    pub const EMPTY: BitBoard = BitBoard(0);
    pub const FULL: BitBoard = BitBoard(!0);

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

impl_bit_assign!(BitAndAssign::bitand_assign, BitOrAssign::bitor_assign);

impl std::fmt::Debug for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
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
            let sq =  Some(Square(self.bitboard.leading_zeros() as usize));
            self.bitboard &= self.bitboard - 1;
            return sq;
        }
        None
    }
}