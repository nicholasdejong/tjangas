use crate::{piece::{Piece, PromotionPiece}, square::Square, bitboard::BitBoard, color::Color};

pub enum MoveFlags {
    Promotion(PromotionPiece)
}

#[derive(Debug)]
pub struct PieceMoves {
    pub piece: Piece,
    pub from: Square,
    pub moves: BitBoard
}

impl PieceMoves {
    pub const fn len(&self) -> u32 {
        if self.piece as usize == 5 {
            let promotion = 0xff000000000000ff;
            (self.moves.0 & !promotion).count_ones() + (self.moves.0 & promotion).count_ones() * 4
        } else {
            self.moves.len()
        }
    }

    /// Specifies whether `PieceMoves` contains any moves
    pub fn is_empty(&self) -> bool {
        self.moves.is_empty()
    }

    /// Converts `PieceMoves` to `Vec<Move>`
    pub fn convert(&self, color: Color) -> Vec<Move> {
        let seventh_rank = match color {
            Color::White => BitBoard(0xff000000000000),
            Color::Black => BitBoard(0xff00)
        };
        let mut moves = vec![];
        if self.piece == Piece::Pawn {
            for sq in self.moves & seventh_rank {
                moves.push(Move {
                    piece: self.piece,
                    from: self.from,
                    to: sq,
                    flags: Some(MoveFlags::Promotion(PromotionPiece::Queen))
                });
                moves.push(Move {
                    piece: self.piece,
                    from: self.from,
                    to: sq,
                    flags: Some(MoveFlags::Promotion(PromotionPiece::Rook))
                });
                moves.push(Move {
                    piece: self.piece,
                    from: self.from,
                    to: sq,
                    flags: Some(MoveFlags::Promotion(PromotionPiece::Bishop))
                });
                moves.push(Move {
                    piece: self.piece,
                    from: self.from,
                    to: sq,
                    flags: Some(MoveFlags::Promotion(PromotionPiece::Knight))
                });
            }
        } else {
            for sq in self.moves {
                moves.push(Move {
                    piece: self.piece,
                    from: self.from,
                    to: sq,
                    flags: None
                });
            }
        }
        moves
    }
}

pub struct Move {
    pub piece: Piece,
    pub from: Square,
    pub to: Square,
    pub flags: Option<MoveFlags>
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.from, self.to, 
            if let Some(flags) = &self.flags {
                let MoveFlags::Promotion(promo) = flags;
                format!("{promo}")
            } else {String::new()}
        )
    }
}
