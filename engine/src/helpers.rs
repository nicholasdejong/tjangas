use types::{square::Square, bitboard::BitBoard};

pub fn square_idx(sq: &str) -> usize {
    let col = sq.chars().next().expect("Invalid square");
    let row = sq.chars().nth(1).expect("Invalid square");
    8 * row as usize - 49 + col as usize
}

pub const fn piece_idx(piece: char) -> usize {
    match piece {
        'k' | 'K' => 0,
        'q' | 'Q' => 1,
        'r' | 'R' => 2,
        'b' | 'B' => 3,
        'n' | 'N' => 4,
        'p' | 'P' => 5,
        _ => panic!("Invalid piece")
    }
}

pub const fn squares_between(from: Square, to: Square) -> BitBoard {
    const fn squares_between(from: Square, to: Square) -> BitBoard {
        let df = to.file() as i8 - from.file() as i8;
        let dr = to.rank() as i8 - from.rank() as i8;
        let orthagonal = df == 0 || dr == 0;
        let diagonal = df.abs() == dr.abs();
        if !(diagonal || orthagonal) {
            return BitBoard::EMPTY;
        }
        let df = df.signum();
        let dr = dr.signum();
        let mut square = from.offset(df, dr);
        let mut between = BitBoard::EMPTY;
        while square.0 != to.0 {
            between.0 |= square.bitboard().0;
            square = square.offset(df, dr);
        }
        between
    }
    const TABLE: [[BitBoard; Square::NUM]; Square::NUM] = {
        let mut table = [[BitBoard::EMPTY; Square::NUM]; Square::NUM];
        let mut i = 0;
        while i < Square::NUM {
            let mut j = 0;
            while j < Square::NUM {
                table[i][j] = squares_between(Square(i), Square(j));
                j += 1;
            }
            i += 1;
        }
        table
    };
    TABLE[from.0][to.0]
}
