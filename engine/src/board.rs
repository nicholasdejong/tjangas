use types::bitboard::BitBoard;
use types::color::Color;
use types::nonsliders::common::{KNIGHT_MOVES, KING_MOVES};
use types::piece::Piece;
use types::r#move::Move;
use types::sliders::dumb7fill::{bishop_moves, rook_moves};
use types::sliders::magic::magic_index;
use types::square::Square;
use types::sliders::common::{TABLE_SIZE, ROOK_BLOCKERS, BISHOP_BLOCKERS, ROOK_MAGICS, ROOK_SHIFT, BISHOP_MAGICS, BISHOP_SHIFT, BISHOP_ATTACKS, ROOK_ATTACKS, ROOK_OFFSETS, BISHOP_OFFSETS};

use crate::moves::squares_between;

include!(concat!(env!("OUT_DIR"), "/slider_moves.rs"));

pub const SLIDER_TABLE: [u64; TABLE_SIZE] = get_table();

#[derive(Clone, Copy, Debug)]
pub struct Board {
    pieces: [[BitBoard; 6]; 2], // [KQRBNP, kqrbnp]
    all: [BitBoard; 2], // white, black
    turn: Color,
    castling: [(bool, bool); 2], // [(K, Q), (k, q)]
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
        let mut board = Board::new();

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
        if sq != 64 {
            return Err(FENParseError);
        }

        board.turn = Color::from_str(color).map_err(|_| FENParseError)?;

        if board.pieces[0][0] == BitBoard(16) {
            board.castling[0].0 = castling.contains("K");
            board.castling[0].1 = castling.contains("Q");
        }
        if board.pieces[1][0] == BitBoard(0x1000000000000000) {
            board.castling[1].0 = castling.contains("k");
            board.castling[1].1 = castling.contains("q");
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
            } else {
                return Err(FENParseError);
            }
        }

        board.halfmoves = halfmoves.parse().map_err(|_| FENParseError)?;
        board.fullmoves = fullmoves.parse().map_err(|_| FENParseError)?;

        Ok(board)
    }
}

impl Default for Board {
    fn default() -> Self {
        use std::str::FromStr;
        Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").expect("Error parsing FEN")
    }
}

impl Board {
    pub const fn new() -> Self {
        Board {
            all: [BitBoard::EMPTY; 2],
            castling: [(false, false); 2],
            enpassant: None,
            halfmoves: 0,
            fullmoves: 0,
            pieces: [[BitBoard::EMPTY; 6]; 2],
            turn: Color::White
        }
    }

    pub const fn occupied(&self) -> BitBoard {
        BitBoard(self.all[0].0 | self.all[1].0)
    }

    pub const fn us(&self) -> BitBoard {
        self.all[self.turn as usize]
    }

    pub const fn them(&self) -> BitBoard {
        self.all[1 - self.turn as usize]
    }


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
            if (between & self.us()).len() == 1 && (between & self.them()).len() == 0 {
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

    /// Returns mask containing all squares attacked by enemy pieces
    fn danger(&self) -> BitBoard {
        let enemy_color = 1 - self.turn as usize;
        let enemy = self.pieces[enemy_color];

        // pawns
        let mut mask = BitBoard(((enemy[5] & BitBoard::NOT_A).0 >> 9 | (enemy[5] & BitBoard::NOT_H).0 >> 7) << (16 * enemy_color));
        // king
        mask |= BitBoard(KING_MOVES[enemy[0].0.trailing_zeros() as usize]);
        // bishops / queen diagonals
        mask |= BitBoard(bishop_moves(enemy[1].0 | enemy[3].0, self.occupied().0));
        // rooks / queen orthagonals
        mask |= BitBoard(rook_moves(enemy[1].0 | enemy[2].0, self.occupied().0));
        // knights
        for knight in enemy[4] {
            mask |= BitBoard(KNIGHT_MOVES[knight.0]);
        }
        mask
    }

    // FIXME: Untested, especially pawn moves
    /// Returns all moves for current position and color
    pub fn moves(&self) -> Vec<Move> {
        let (checkmask, orthagonal, diagonal) = self.checkmask_pinmask();
        let mut moves = vec![];
        let color = self.turn as usize;
        let pieces = self.pieces[color];
        let danger = self.danger();

        // King moves
        let king = Square(self.pieces[color][0].0.trailing_zeros() as usize);
        let mut bb = BitBoard(KING_MOVES[king.0]);
        if (danger & king.bitboard()).is_empty() {
            // Not in check => castling
            if self.castling[color].0 && (danger & BitBoard(king.bitboard().0 << 1)).is_empty() {
                bb |= BitBoard(king.bitboard().0 << 2);
            }
            if self.castling[color].1 && (danger & BitBoard(king.bitboard().0 >> 1)).is_empty() {
                bb |= BitBoard(king.bitboard().0 >> 2);
            }
        }
        for sq in bb & !self.us() & danger {
            moves.push(Move {
                piece: Piece::King,
                from: king,
                to: sq,
                promotion: None
            });
        }

        if (checkmask & self.them()).len() > 1 {
            // Double check, only generate king moves
            return moves;
        }

        let knights = pieces[4] & !(orthagonal | diagonal);
        for from in knights {
            let bb = BitBoard(KNIGHT_MOVES[from.0]) & checkmask & !self.us();
            for sq in bb {
                moves.push(Move {
                    piece: Piece::Knight,
                    from,
                    to: sq,
                    promotion: None
                });
            }
        }

        let rooks = pieces[2] & !diagonal;
        for from in rooks {
            let blockers = BitBoard(ROOK_BLOCKERS[from.0]) & self.occupied();
            let idx = magic_index(blockers, ROOK_MAGICS[from.0], ROOK_SHIFT) + ROOK_OFFSETS[from.0];
            let bb = BitBoard(SLIDER_TABLE[idx]) & checkmask & !self.us();
            for sq in bb {
                moves.push(Move {
                    piece: Piece::Rook,
                    from,
                    to: sq,
                    promotion: None
                });
            }
        }

        let bishops = pieces[3] & !orthagonal;
        for from in bishops {
            let blockers = BitBoard(BISHOP_BLOCKERS[from.0]) & self.occupied();
            let idx = magic_index(blockers, BISHOP_MAGICS[from.0], BISHOP_SHIFT) + BISHOP_OFFSETS[from.0];
            let bb = BitBoard(SLIDER_TABLE[idx]) & checkmask & !self.us();
            for sq in bb {
                moves.push(Move {
                    piece: Piece::Bishop,
                    from,
                    to: sq,
                    promotion: None
                });
            }
        }

        // Diagonal queen moves
        let queens = pieces[1] & !orthagonal;
        for from in queens {
            let blockers = BitBoard(BISHOP_BLOCKERS[from.0]) & self.occupied();
            let idx = magic_index(blockers, BISHOP_MAGICS[from.0], BISHOP_SHIFT) + BISHOP_OFFSETS[from.0];
            let bb = BitBoard(SLIDER_TABLE[idx]) & checkmask & !self.us();
            for sq in bb {
                moves.push(Move {
                    piece: Piece::Queen,
                    from,
                    to: sq,
                    promotion: None
                });
            }
        }
        // Orthagonal queen moves
        let queens = pieces[1] & !diagonal;
        for from in queens {
            let blockers = BitBoard(ROOK_BLOCKERS[from.0]) & self.occupied();
            let idx = magic_index(blockers, ROOK_MAGICS[from.0], ROOK_SHIFT) + ROOK_OFFSETS[from.0];
            let bb = BitBoard(SLIDER_TABLE[idx]) & checkmask & !self.us();
            for sq in bb {
                moves.push(Move {
                    piece: Piece::Queen,
                    from,
                    to: sq,
                    promotion: None
                });
            }
        }

        // Pinned pawn pushes
        let pawns = pieces[5] & orthagonal;
        let mut bb = BitBoard(pawns.0 << 8) & !self.occupied() & checkmask & orthagonal;
        // Pinned pawn captures
        let pawns = pieces[5] & diagonal;
        let captures = BitBoard(((pawns & BitBoard::NOT_A).0 << 7 | (pawns & BitBoard::NOT_H).0 << 9) >> (16 * color));
        bb |= captures & self.all[1 - color] & checkmask & diagonal;
        // Unpinned pawn pushes and captures
        let pawns = pieces[5] & !(orthagonal | diagonal);
        bb |= BitBoard(pawns.0 << 8) & !self.occupied() & checkmask;
        let captures = BitBoard(((pawns & BitBoard::NOT_A).0 << 7 | (pawns & BitBoard::NOT_H).0 << 9) >> (16 * color));
        bb |= captures & self.all[1 - color] & checkmask;

        // TODO: en passant

        moves
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    #[test]
    fn test_board_pieces() {
        let brd = Board::default();
        assert_eq!(brd.pieces[0], [BitBoard(16), BitBoard(8), BitBoard(129), BitBoard(36), BitBoard(66), BitBoard(65280)]);
        assert_eq!(brd.pieces[1], [BitBoard(0x1000000000000000), BitBoard(0x800000000000000), BitBoard(0x8100000000000000), BitBoard(0x2400000000000000), BitBoard(0x4200000000000000), BitBoard(0xff000000000000)]);

        let brd = Board::from_str("3nbrqk/3ppppp/8/8/8/8/PPPPP3/KQRBN3 w - - 0 1").unwrap();
        assert_eq!(brd.pieces[0], [BitBoard(1), BitBoard(2), BitBoard(4), BitBoard(8), BitBoard(16), BitBoard(0x1f00)]);
        assert_eq!(brd.pieces[1], [BitBoard(0x8000000000000000), BitBoard(0x4000000000000000), BitBoard(0x2000000000000000), BitBoard(0x1000000000000000), BitBoard(0x800000000000000), BitBoard(0xf8000000000000)]);
    }

    #[test]
    fn test_checkmask_pinmask() {
        let brd = Board::default();
        let masks = brd.checkmask_pinmask();
        assert_eq!(masks, (BitBoard::FULL, BitBoard::FULL, BitBoard::FULL));

        let brd = Board::from_str("8/2k3b1/8/8/8/2K5/8/8 w - - 0 1").unwrap();
        let masks = brd.checkmask_pinmask();
        assert_eq!(masks, (BitBoard(0x40201008000000), BitBoard::FULL, BitBoard::FULL));

        let brd = Board::from_str("8/8/5b2/8/3N4/2K5/8/8 w - - 0 1").unwrap();
        let masks = brd.checkmask_pinmask();
        assert_eq!(masks, (BitBoard::FULL, BitBoard::FULL, BitBoard(0x201008000000)));
    }
}