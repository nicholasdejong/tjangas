use types::bitboard::BitBoard;
use types::color::Color;
use types::square::Square;

#[derive(Clone, Copy, Debug, Default)]
struct Pieces {
    white: [BitBoard; 6], // kqrbnp
    black: [BitBoard; 6]
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Board {
    pieces: Pieces,
    turn: Color,
    castling: (bool, bool, bool, bool), // KQkq
    halfmoves: u8,
    fullmoves: u16,
    enpassant: Option<Square>
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
            for char in row.chars() {
                match char {
                    'K' | 'Q' | 'R' | 'B' | 'N' | 'P' => {
                        board.pieces.white[piece_idx(char)].0 |= 1 << sq;
                        sq += 1;
                    }
                    'k' | 'q' | 'r' | 'b' | 'n' | 'p' => {
                        board.pieces.black[piece_idx(char)].0 |= 1 << sq;
                        sq += 1;
                    }
                    '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' => {
                        sq += char as usize - 48;
                    }
                    _ => return Err(FENParseError)
                }
            }
        }

        if let Ok(color) = Color::from_str(color) {
            board.turn = color;
        } else {
            return Err(FENParseError);
        }

        if board.pieces.white[0] == BitBoard(16) {
            board.castling.0 = castling.contains("K");
            board.castling.1 = castling.contains("Q");
        }
        if board.pieces.black[0] == BitBoard(0x1000000000000000) {
            board.castling.2 = castling.contains("k");
            board.castling.3 = castling.contains("q");
        }

        if enpassant != "-" {
            if ['a','b','c','d','e','f','g','h'].contains(&enpassant.chars().next().ok_or(FENParseError)?) && ['1','2','3','4','5','6','7','8'].contains(&enpassant.chars().nth(1).ok_or(FENParseError)?) {
                let sq = square_idx(enpassant);
                if sq > 47 || sq < 16 {
                    return Err(FENParseError);
                }
                if (board.turn == Color::White && 1 << (sq - 8) & board.pieces.black[5].0 > 0) || (board.turn == Color::Black && 1 << (sq + 8) & board.pieces.white[5].0 > 0) {
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
    
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    #[test]
    fn test_board_pieces() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let brd = Board::from_str(fen).expect("Invalid fen");
        assert_eq!(brd.pieces.white, [BitBoard(16), BitBoard(8), BitBoard(129), BitBoard(36), BitBoard(66), BitBoard(65280)]);
        assert_eq!(brd.pieces.black, [BitBoard(0x1000000000000000), BitBoard(0x800000000000000), BitBoard(0x8100000000000000), BitBoard(0x2400000000000000), BitBoard(0x4200000000000000), BitBoard(0xff000000000000)]);
    }
}