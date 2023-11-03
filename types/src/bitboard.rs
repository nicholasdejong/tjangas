
#[derive(Clone, Copy, PartialEq)]
pub struct BitBoard(pub u64);

impl BitBoard {
    pub const EMPTY: BitBoard = BitBoard(0);

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

impl std::ops::BitAnd for BitBoard {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 & rhs.0)
    }
}

impl std::fmt::Debug for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}