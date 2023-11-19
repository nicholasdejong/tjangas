use types::bitboard::BitBoard;
use types::color::Color;
use types::nonsliders::common::{KNIGHT_MOVES, KING_MOVES, PAWN_ATTACKS};
use types::piece::Piece;
use types::moves::PieceMoves;
use types::slider::Slider;
// use types::sliders::dumb7fill::{bishop_moves, rook_moves};
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
    enpassant: BitBoard,
    halfmoves: u8,
    fullmoves: u16,
}

struct Masks {
    checkmask: BitBoard,
    diagonal: BitBoard,
    orthagonal: BitBoard,
    en_pessant: BitBoard // Contains mask of "pinned" enemy pawns preventing illegal en pessant
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
                    board.enpassant = Square(sq).bitboard();
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
            enpassant: BitBoard::EMPTY,
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
    fn masks(&self) -> Masks {
        let color = self.turn as usize;
        let king = self.pieces[color][0].0.trailing_zeros() as usize;
        let enemy = self.pieces[1 - color];

        let diagonal = BitBoard(BISHOP_ATTACKS[king]) & (enemy[1] | enemy[3]);
        let orthagonal = BitBoard(ROOK_ATTACKS[king]) & (enemy[1] | enemy[2]);

        let mut checkmask = BitBoard::EMPTY;
        let mut pinmask = (BitBoard::EMPTY, BitBoard::EMPTY);
        let mut en_pessant = BitBoard::EMPTY;

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
            if (between & enemy[5]).len() == 1 {
                en_pessant |= between & enemy[5];
            }
        }

        if checkmask.is_empty() {
            checkmask = BitBoard::FULL;
        }

        Masks {
            checkmask,
            orthagonal: pinmask.0,
            diagonal: pinmask.1,
            en_pessant
        }
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
        for piece in enemy[1] | enemy[3] {
            let blockers = BitBoard(BISHOP_BLOCKERS[piece.0]) & self.occupied();
            let idx = magic_index(blockers, BISHOP_MAGICS[piece.0], BISHOP_SHIFT) + BISHOP_OFFSETS[piece.0];
            mask |= BitBoard(SLIDER_TABLE[idx]);
        }
        // rooks / queen orthagonals
        for piece in enemy[1] | enemy[2] {
            let blockers = BitBoard(ROOK_BLOCKERS[piece.0]) & self.occupied();
            let idx = magic_index(blockers, ROOK_MAGICS[piece.0], ROOK_SHIFT) + ROOK_OFFSETS[piece.0];
            mask |= BitBoard(SLIDER_TABLE[idx]);
        }
        // knights
        for knight in enemy[4] {
            mask |= BitBoard(KNIGHT_MOVES[knight.0]);
        }
        mask
    }

    // FIXME: Untested, especially pawn moves
    /// Returns all moves for current position and color
    pub fn moves(&self) -> Vec<PieceMoves> {
        let Masks {checkmask, orthagonal, diagonal, en_pessant} = self.masks();
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
        moves.push(PieceMoves {
            piece: Piece::King,
            from: king,
            moves: bb & !self.us() & danger
        });

        if (checkmask & self.them()).len() > 1 && checkmask != BitBoard::FULL {
            // Double check, only generate king moves
            return moves;
        }

        let knights = pieces[4] & !(orthagonal | diagonal);
        for from in knights {
            moves.push(PieceMoves {
                piece: Piece::Knight,
                from,
                moves: BitBoard(KNIGHT_MOVES[from.0]) & checkmask & !self.us()
            });
        }

        // Pinless rook + queen moves
        let rooks = (pieces[1] | pieces[2]) & !(orthagonal | diagonal);
        for from in rooks {
            let blockers = BitBoard(ROOK_BLOCKERS[from.0]) & self.occupied();
            let idx = magic_index(blockers, ROOK_MAGICS[from.0], ROOK_SHIFT) + ROOK_OFFSETS[from.0];
            moves.push(PieceMoves {
                piece: [Piece::Queen, Piece::Rook][(pieces[2] & from.bitboard()).signum()],
                from,
                moves: BitBoard(SLIDER_TABLE[idx]) & checkmask & !self.us()
            });
        }

        // Pinless bishop + queen moves
        let bishops = (pieces[1] | pieces[3]) & !(orthagonal | diagonal);
        for from in bishops {
            let blockers = BitBoard(BISHOP_BLOCKERS[from.0]) & self.occupied();
            let idx = magic_index(blockers, BISHOP_MAGICS[from.0], BISHOP_SHIFT) + BISHOP_OFFSETS[from.0];
            dbg!(idx);
            dbg!(SLIDER_TABLE[idx], Slider::Bishop.pseudo_moves(from, blockers));

            // FIXME: Fix hash collisions
            // moves.push(PieceMoves {
            //     piece: [Piece::Queen, Piece::Bishop][(pieces[3] & from.bitboard()).signum()],
            //     from,
            //     moves: BitBoard(SLIDER_TABLE[idx]) & checkmask & !self.us()
            // });
        }

        // Diagonally pinned queens and bishops
        let diagonal = (pieces[1] | pieces[3]) & diagonal;
        for from in diagonal {
            let blockers = BitBoard(BISHOP_BLOCKERS[from.0]) & self.occupied();
            let idx = magic_index(blockers, BISHOP_MAGICS[from.0], BISHOP_SHIFT) + BISHOP_OFFSETS[from.0];
            moves.push(PieceMoves {
                piece: [Piece::Queen, Piece::Bishop][(pieces[3] & from.bitboard()).signum()],
                from,
                moves: BitBoard(SLIDER_TABLE[idx]) & checkmask & !self.us() & diagonal
            });
        }

        // Orthagonally pinned queens and rooks
        let orthagonal = (pieces[1] | pieces[2]) & orthagonal;
        for from in orthagonal {
            let blockers = BitBoard(ROOK_BLOCKERS[from.0]) & self.occupied();
            let idx = magic_index(blockers, ROOK_MAGICS[from.0], ROOK_SHIFT) + ROOK_OFFSETS[from.0];
            moves.push(PieceMoves {
                piece: [Piece::Queen, Piece::Bishop][(pieces[2] & from.bitboard()).signum()],
                from,
                moves: BitBoard(SLIDER_TABLE[idx]) & checkmask & !self.us() & orthagonal
            });
        }

        let push_white = |x: u64| x << 8;
        let push_black = |x: u64| x >> 8;
        let push_forward = [push_white, push_black][color];
        let second_rank = BitBoard([0xff00, 0xff000000000000][color]);

        // Orthagonally pinned pawns
        let pawns = pieces[5] & orthagonal;
        for from in pawns {
            let mut bb = BitBoard(push_forward(from.bitboard().0)) & !self.occupied() & orthagonal;
            if !(second_rank & from.bitboard()).is_empty() && !bb.is_empty() {
                bb |= BitBoard(push_forward(bb.0));
            }
            moves.push(PieceMoves {
                piece: Piece::Pawn,
                from,
                moves: bb & checkmask
            });
        }

        // Diagonally pinned pawns
        let pawns = pieces[5] & diagonal;
        for from in pawns {
            moves.push(PieceMoves {
                piece: Piece::Pawn,
                from,
                moves: BitBoard(PAWN_ATTACKS[color][from.0]) & checkmask & diagonal & self.all[1 - color]
            });
        }

        // Pinless pawn pushes and captures
        let pawns = pieces[5] & !(orthagonal | diagonal);
        for from in pawns {
            // captures
            let mut bb = BitBoard(PAWN_ATTACKS[color][from.0]) & (self.all[1 - color] | (BitBoard(push_forward(self.enpassant.0)) & !en_pessant));
            // single push
            let single = BitBoard(push_forward(from.bitboard().0)) & !self.occupied();
            bb |= single;
            // double push
            if !single.is_empty() {
                bb |= BitBoard(push_forward(single.0)) & !self.occupied();
            }

            moves.push(PieceMoves {
                piece: Piece::Pawn,
                from,
                moves: bb & checkmask
            });
        }

        moves
    }
}