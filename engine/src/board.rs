use types::bitboard::BitBoard;
use types::color::Color;
use types::nonsliders::common::{KNIGHT_MOVES, PAWN_ATTACKS};
use types::sliders::magic::magic_index;
use types::square::Square;
use types::sliders::common::{TABLE_SIZE, ROOK_BLOCKERS, BISHOP_BLOCKERS, ROOK_MAGICS, ROOK_SHIFT, BISHOP_MAGICS, BISHOP_SHIFT, BISHOP_ATTACKS, ROOK_ATTACKS};

use crate::moves::squares_between;

include!(concat!(env!("OUT_DIR"), "/slider_moves.rs"));

pub const SLIDER_TABLE: [u64; TABLE_SIZE] = get_table();

#[derive(Clone, Copy, Debug, Default)]
pub struct Board {
    pieces: [[BitBoard; 6]; 2], // [KQRBNP, kqrbnp]
    all: [BitBoard; 2], // white, black
    turn: Color,
    castling: (bool, bool, bool, bool), // KQkq
    enpassant: Option<Square>,
    halfmoves: u8,
    fullmoves: u16,
}

fn piece_idx(piece: char) -> usize {
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

fn square_idx<'a>(sq: &'a str) -> usize {
    let col = sq.chars().next().expect("Invalid square");
    let row = sq.chars().nth(1).expect("Invalid square");
    return 8 * row as usize - 49 + col as usize;
}

#[derive(Debug)]
pub struct FENParseError;

impl std::str::FromStr for Board {
    type Err = FENParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut board = Board::default();

        let mut parts = s.split_whitespace();
        let pieces = parts.next().ok_or(FENParseError)?;
        let color = parts.next().ok_or(FENParseError)?;
        let castling = parts.next().ok_or(FENParseError)?;
        let enpassant = parts.next().ok_or(FENParseError)?;
        let halfmoves = parts.next().ok_or(FENParseError)?;
        let fullmoves = parts.next().ok_or(FENParseError)?;

        let mut sq = 0;
        for row in pieces.rsplit("/") {
            let prev = sq;
            for char in row.chars() {
                match char {
                    'K' | 'Q' | 'R' | 'B' | 'N' | 'P' => {
                        board.pieces[0][piece_idx(char)].0 |= 1 << sq;
                        board.all[0].0 |= 1 << sq;
                        sq += 1;
                    }
                    'k' | 'q' | 'r' | 'b' | 'n' | 'p' => {
                        board.pieces[1][piece_idx(char)].0 |= 1 << sq;
                        board.all[1].0 |= 1 << sq;
                        sq += 1;
                    }
                    '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' => {
                        sq += char as usize - 48;
                    }
                    _ => return Err(FENParseError)
                }
            }
            if sq - prev != 8 {
                return Err(FENParseError);
            }
        }

        if let Ok(color) = Color::from_str(color) {
            board.turn = color;
        } else {
            return Err(FENParseError);
        }

        if board.pieces[0][0] == BitBoard(16) {
            board.castling.0 = castling.contains("K");
            board.castling.1 = castling.contains("Q");
        }
        if board.pieces[1][0] == BitBoard(0x1000000000000000) {
            board.castling.2 = castling.contains("k");
            board.castling.3 = castling.contains("q");
        }

        if enpassant != "-" {
            if ['a','b','c','d','e','f','g','h'].contains(&enpassant.chars().next().ok_or(FENParseError)?) && ['1','2','3','4','5','6','7','8'].contains(&enpassant.chars().nth(1).ok_or(FENParseError)?) {
                let sq = square_idx(enpassant);
                if sq > 47 || sq < 16 {
                    return Err(FENParseError);
                }
                if (board.turn == Color::White && 1 << (sq - 8) & board.pieces[1][5].0 > 0) || (board.turn == Color::Black && 1 << (sq + 8) & board.pieces[0][5].0 > 0) {
                    board.enpassant = Some(Square(sq));
                }
            }
        }

        if let Ok(halfmoves) = halfmoves.parse::<u8>() {
            board.halfmoves = halfmoves;
        } else {
            return Err(FENParseError);
        }
        if let Ok(fullmoves) = fullmoves.parse::<u16>() {
            board.fullmoves = fullmoves.min(500);
        } else {
            return Err(FENParseError);
        }

        Ok(board)
    }
}

impl Board {
    pub const fn occupied(&self) -> BitBoard {
        BitBoard(self.all[0].0 | self.all[1].0)
    }

    pub const fn us(&self) -> BitBoard {
        self.all[self.turn as usize]
    }

    pub const fn them(&self) -> BitBoard {
        self.all[1 - self.turn as usize]
    }

    // FIXME: Untested code
    /// Calculates checkmask and pinmask for current position and color
    fn checkmask_pinmask(&self) -> (BitBoard /* Checkmask */, BitBoard /* Orthagonal */, BitBoard /* Diagonal */) {
        let color = self.turn as usize;
        let king = self.pieces[color][0].0.trailing_zeros() as usize;
        let enemy = self.pieces[1 - color];

        let diagonal = BitBoard(BISHOP_ATTACKS[king]) & (enemy[1] | enemy[3]);
        let orthagonal = BitBoard(ROOK_ATTACKS[king]) & (enemy[1] | enemy[2]);

        let mut checkmask = BitBoard::EMPTY;
        let mut pinmask = (BitBoard::EMPTY, BitBoard::EMPTY);

        for attacker in orthagonal {
            let between = squares_between(Square(king), attacker);
            if (between & self.occupied()).len() == 0 {
                checkmask |= between | attacker.bitboard();
            }
            if (between & self.us()).len() == 1 {
                pinmask.0 |= between | attacker.bitboard();
            }
        }

        for attacker in diagonal {
            let between = squares_between(Square(king), attacker);
            if (between & self.occupied()).len() == 0 {
                checkmask |= between | attacker.bitboard();
            }
            if (between & self.us()).len() == 1 {
                pinmask.1 |= between | attacker.bitboard();
            }
        }

        if checkmask.is_empty() {
            checkmask = BitBoard::FULL;
        }
        if pinmask.0.is_empty() {
            pinmask.0 = BitBoard::FULL;
        }
        if pinmask.1.is_empty() {
            pinmask.1 = BitBoard::FULL;
        }
        (checkmask, pinmask.0, pinmask.1)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    #[test]
    fn test_board_pieces() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let brd = Board::from_str(fen).expect("Invalid fen");
        assert_eq!(brd.pieces[0], [BitBoard(16), BitBoard(8), BitBoard(129), BitBoard(36), BitBoard(66), BitBoard(65280)]);
        assert_eq!(brd.pieces[0], [BitBoard(0x1000000000000000), BitBoard(0x800000000000000), BitBoard(0x8100000000000000), BitBoard(0x2400000000000000), BitBoard(0x4200000000000000), BitBoard(0xff000000000000)]);
    }
}