use std::error::Error;
use thiserror::Error;

use types::bitboard::BitBoard;
use types::color::Color;
use types::nonsliders::common::{KNIGHT_MOVES, KING_MOVES, PAWN_ATTACKS};
use types::piece::{Piece, PromotionPiece};
use types::moves::{PieceMoves, Move, MoveFlags::Promotion};
use types::sliders::magic::magic_index;
use types::square::Square;
use types::sliders::common::{ROOK_BLOCKERS, BISHOP_BLOCKERS, ROOK_MAGICS, BISHOP_MAGICS, BISHOP_ATTACKS, ROOK_ATTACKS, ROOK_OFFSETS, BISHOP_OFFSETS, BISHOP_SIZE, ROOK_SIZE, BISHOP_SHIFTS, ROOK_SHIFTS};

use crate::helpers::{square_idx, piece_idx, squares_between};

include!(concat!(env!("OUT_DIR"), "/slider_moves.rs"));

pub const BISHOP_TABLE: [u64; BISHOP_SIZE] = get_bishop_table();
pub const ROOK_TABLE: [u64; ROOK_SIZE] = get_rook_table();

pub const PIECES: [Piece; 6] = [Piece::King, Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight, Piece::Pawn];

#[derive(Clone, Copy, Debug)]
pub struct Board {
    pub pieces: [[BitBoard; 6]; 2], // [KQRBNP, kqrbnp]
    pub all: [BitBoard; 2], // white, black
    pub turn: Color,
    pub castling: [(bool, bool); 2], // [(K, Q), (k, q)]
    prev_castling: [(bool, bool); 2], // store previous castling state (undo move)
    pub enpassant: BitBoard,
    prev_enpassant: BitBoard, // store previous enpessant (undo move)
    pub halfmoves: u8,
    pub fullmoves: u16,
    pub squares: [Option<Piece>; 64], // Current position piece lookup
    prev_square: Option<Piece>
}

struct Masks {
    checkmask: BitBoard,
    diagonal: BitBoard,
    orthagonal: BitBoard,
    en_pessant: BitBoard // Contains mask of "pinned" enemy pawns preventing illegal en pessant
}
#[derive(Debug, Error)]
pub enum FENParseError {
    #[error("{0} is not a valid en-passant square.")]
    EnPassantError(String),
    #[error("FEN has incorrect spacing.")]
    SpacingError,
    #[error("The FEN piece rows are invalid")]
    RowError,
    #[error("FEN contains invalid symbol: {0}")]
    SymbolError(String),
}

impl std::str::FromStr for Board {
    type Err = Box<dyn Error>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut board = Board::new();

        let mut parts = s.split_whitespace();
        let pieces = parts.next().ok_or(FENParseError::SpacingError)?;
        let color = parts.next().ok_or(FENParseError::SpacingError)?;
        let castling = parts.next().ok_or(FENParseError::SpacingError)?;
        let enpassant = parts.next().ok_or(FENParseError::SpacingError)?;
        let halfmoves = parts.next().ok_or(FENParseError::SpacingError)?;
        let fullmoves = parts.next().ok_or(FENParseError::SpacingError)?;

        let mut sq = 0;
        for row in pieces.rsplit("/") {
            let prev = sq;
            for char in row.chars() {
                match char {
                    'K' | 'Q' | 'R' | 'B' | 'N' | 'P' => {
                        board.pieces[0][piece_idx(char)].0 |= 1 << sq;
                        board.all[0].0 |= 1 << sq;
                        board.squares[sq] = Some(PIECES[piece_idx(char)]);
                        sq += 1;
                    }
                    'k' | 'q' | 'r' | 'b' | 'n' | 'p' => {
                        board.pieces[1][piece_idx(char)].0 |= 1 << sq;
                        board.all[1].0 |= 1 << sq;
                        board.squares[sq] = Some(PIECES[piece_idx(char)]);
                        sq += 1;
                    }
                    '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' => {
                        sq += char as usize - 48;
                    }
                    invalid => return Err(Box::new(FENParseError::SymbolError(String::from(invalid))))
                }
            }
            if sq - prev != 8 {
                return Err(Box::new(FENParseError::RowError));
            }
        }
        if sq != 64 {
            return Err(Box::new(FENParseError::RowError));
        }

        board.turn = Color::from_str(color)?;

        if board.pieces[0][0] == BitBoard(16) {
            board.castling[0].0 = castling.contains("K");
            board.castling[0].1 = castling.contains("Q");
        }
        if board.pieces[1][0] == BitBoard(0x1000000000000000) {
            board.castling[1].0 = castling.contains("k");
            board.castling[1].1 = castling.contains("q");
        }

        if enpassant != "-" {
            if ['a','b','c','d','e','f','g','h'].contains(&enpassant.chars().next().ok_or(FENParseError::EnPassantError(String::from(enpassant)))?) && ['1','2','3','4','5','6','7','8'].contains(&enpassant.chars().nth(1).ok_or(FENParseError::EnPassantError(String::from(enpassant)))?) {
                let sq = square_idx(enpassant);
                if sq > 47 || sq < 16 {
                    return Err(Box::new(FENParseError::EnPassantError(String::from(enpassant))));
                }
                if (board.turn == Color::White && 1 << (sq - 8) & board.pieces[1][5].0 > 0) || (board.turn == Color::Black && 1 << (sq + 8) & board.pieces[0][5].0 > 0) {
                    board.enpassant = Square(sq).bitboard();
                }
            } else {
                return Err(Box::new(FENParseError::EnPassantError(String::from(enpassant))));
            }
        }

        board.halfmoves = halfmoves.parse()?;
        board.fullmoves = fullmoves.parse()?;

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
            prev_castling: [(false, false); 2],
            enpassant: BitBoard::EMPTY,
            prev_enpassant: BitBoard::EMPTY,
            halfmoves: 0,
            fullmoves: 0,
            pieces: [[BitBoard::EMPTY; 6]; 2],
            turn: Color::White,
            squares: [None; 64],
            prev_square: None
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
            let blockers = BitBoard(BISHOP_BLOCKERS[piece.0]) & (self.occupied() ^ self.pieces[self.turn as usize][0]);
            let idx = magic_index(blockers, BISHOP_MAGICS[piece.0], BISHOP_SHIFTS[piece.0]) + BISHOP_OFFSETS[piece.0];
            mask |= BitBoard(BISHOP_TABLE[idx]);
        }
        // rooks / queen orthagonals
        for piece in enemy[1] | enemy[2] {
            let blockers = BitBoard(ROOK_BLOCKERS[piece.0]) & (self.occupied() ^ self.pieces[self.turn as usize][0]);
            // dbg!(piece, ROOK_BLOCKERS[piece.0], blockers);
            let idx = magic_index(blockers, ROOK_MAGICS[piece.0], ROOK_SHIFTS[piece.0]) + ROOK_OFFSETS[piece.0];
            mask |= BitBoard(ROOK_TABLE[idx]);
        }
        // knights
        for knight in enemy[4] {
            mask |= BitBoard(KNIGHT_MOVES[knight.0]);
        }
        mask
    }

    // TODO: Compare nodes searched for known positions to find errors
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
            moves: bb & !self.us() & !danger
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
            let idx = magic_index(blockers, ROOK_MAGICS[from.0], ROOK_SHIFTS[from.0]) + ROOK_OFFSETS[from.0];
            let piece = match (from.bitboard() & pieces[1]).is_empty() {
                true => Piece::Rook,
                false => Piece::Queen
            };
            moves.push(PieceMoves {
                piece,
                from,
                moves: BitBoard(ROOK_TABLE[idx]) & checkmask & !self.us()
            });
        }

        // Pinless bishop + queen moves
        let bishops = (pieces[1] | pieces[3]) & !(orthagonal | diagonal);
        for from in bishops {
            let blockers = BitBoard(BISHOP_BLOCKERS[from.0]) & self.occupied();
            let idx = magic_index(blockers, BISHOP_MAGICS[from.0], BISHOP_SHIFTS[from.0]) + BISHOP_OFFSETS[from.0];
            let piece = match (from.bitboard() & pieces[1]).is_empty() {
                true => Piece::Bishop,
                false => Piece::Queen
            };
            moves.push(PieceMoves {
                piece,
                from,
                moves: BitBoard(BISHOP_TABLE[idx]) & checkmask & !self.us()
            });
        }

        // Diagonally pinned queens and bishops
        let diagonal = (pieces[1] | pieces[3]) & diagonal;
        for from in diagonal {
            let blockers = BitBoard(BISHOP_BLOCKERS[from.0]) & self.occupied();
            let idx = magic_index(blockers, BISHOP_MAGICS[from.0], BISHOP_SHIFTS[from.0]) + BISHOP_OFFSETS[from.0];
            let piece = match (from.bitboard() & pieces[1]).is_empty() {
                true => Piece::Bishop,
                false => Piece::Queen
            };
            moves.push(PieceMoves {
                piece,
                from,
                moves: BitBoard(BISHOP_TABLE[idx]) & checkmask & !self.us() & diagonal
            });
        }

        // Orthagonally pinned queens and rooks
        let orthagonal = (pieces[1] | pieces[2]) & orthagonal;
        for from in orthagonal {
            let blockers = BitBoard(ROOK_BLOCKERS[from.0]) & self.occupied();
            let idx = magic_index(blockers, ROOK_MAGICS[from.0], ROOK_SHIFTS[from.0]) + ROOK_OFFSETS[from.0];
            let piece = match (from.bitboard() & pieces[1]).is_empty() {
                true => Piece::Rook,
                false => Piece::Queen
            };
            moves.push(PieceMoves {
                piece,
                from,
                moves: BitBoard(ROOK_TABLE[idx]) & checkmask & !self.us() & orthagonal
            });
        }

        let second_rank = match self.turn {
            Color::White => BitBoard(0xff00),
            Color::Black => BitBoard(0xff000000000000)
        };

        // Orthagonally pinned pawns
        // A pinned pawn cannot promote
        let pawns = pieces[5] & orthagonal;
        for from in pawns {
            let mut bb = from.bitboard().shift_color(8, self.turn) & !self.occupied() & orthagonal;
            if !(second_rank & from.bitboard()).is_empty() && !bb.is_empty() {
                bb |= bb.shift_color(8, self.turn);
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
            let mut bb = BitBoard(PAWN_ATTACKS[color][from.0]) & (self.all[1 - color] | (self.enpassant.shift_color(8, self.turn) & !en_pessant));
            // single push
            let single = from.bitboard().shift_color(8, self.turn) & !self.occupied();
            bb |= single;
            // double push
            if !single.is_empty() {
                bb |= single.shift_color(8, self.turn) & !self.occupied();
            }

            moves.push(PieceMoves {
                piece: Piece::Pawn,
                from,
                moves: bb & checkmask
            });
        }

        moves
    }

    /// Applies given move to current position
    pub fn apply_move(&mut self, mv: &Move) {
        let piece = mv.piece as usize;
        let color = self.turn as usize;

        self.pieces[color][piece] ^= mv.from.bitboard() | mv.to.bitboard();
        self.all[color] ^= mv.from.bitboard() | mv.to.bitboard();
        self.all[1 - color] &= !mv.to.bitboard();
        if let Some(p) = self.squares[mv.to.0] {
            // we captured a piece, but we don't know whose piece it is!
            self.pieces[0][p as usize] &= !mv.to.bitboard();
            self.pieces[1][p as usize] &= !mv.to.bitboard();
        }
        self.squares[mv.from.0] = None;
        self.prev_square = self.squares[mv.to.0];
        self.squares[mv.to.0] = Some(mv.piece);

        match mv.piece {
            Piece::King => {
                self.prev_castling = self.castling;
                self.castling[color] = (false, false);
            }
            Piece::Pawn => {
                // Promotion
                if let Some(flags) = &mv.flags {
                    let Promotion(promotion) = flags;
                    self.pieces[color][piece] ^= mv.to.bitboard();
                    self.squares[mv.to.0] = match promotion {
                        &PromotionPiece::Queen => Some(Piece::Queen),
                        &PromotionPiece::Rook => Some(Piece::Rook),
                        &PromotionPiece::Bishop => Some(Piece::Bishop),
                        &PromotionPiece::Knight => Some(Piece::Knight)
                    };
                    self.pieces[color][*promotion as usize + 1] ^= mv.to.bitboard();
                }
                // En pessant
                self.prev_enpassant = self.enpassant;
                self.enpassant |= (mv.from.bitboard().shift_color(16, self.turn) & mv.to.bitboard()).shift_color(8, !self.turn);
            }
            Piece::Rook => {
                // Kingside rook
                self.prev_castling = self.castling;
                if (self.pieces[color][0].shl(3) & self.pieces[color][2]).is_empty() {
                    self.castling[color].0 = false;
                }
                // Queenside rook
                if (self.pieces[color][0].shr(4) & self.pieces[color][2]).is_empty() {
                    self.castling[color].1 = false;
                }
            }
            _ => {}
        }

        self.turn = !self.turn;
    }

    /// Undo move to position
    pub fn undo_move(&mut self, mv: &Move) {
        let piece = mv.piece as usize;
        let color = self.turn as usize;

        self.squares[mv.to.0] = self.prev_square;
        self.squares[mv.from.0] = Some(mv.piece);

        self.castling = self.prev_castling;
        self.enpassant = self.prev_enpassant;

        if let Some(flags) = &mv.flags {
            let Promotion(promotion) = flags;
            self.pieces[color][piece] ^= mv.from.bitboard();
            self.pieces[color][*promotion as usize] ^= mv.to.bitboard();
        } else {
            self.pieces[color][piece] ^= mv.from.bitboard() | mv.to.bitboard();
        }

        self.turn = !self.turn;
    }
}