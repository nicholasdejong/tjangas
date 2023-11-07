use crate::bitboard::BitBoard;

pub fn subsets(mask: BitBoard) -> Vec<BitBoard> {
    let mut res: Vec<BitBoard> = Vec::with_capacity(2usize.pow(mask.len()));
    let mut subset = BitBoard::EMPTY;
    loop {
        res.push(subset);
        subset = BitBoard(subset.0.wrapping_sub(mask.0) & mask.0);
        if subset.is_empty() {
            break;
        }
    }
    res
}

pub fn magic_index(blockers: BitBoard, hash: u64, shift_count: usize) -> usize {
    (blockers.0.wrapping_mul(hash) >> shift_count) as usize
}
