use types::color::Color;
use types::board::Board;
use types::square::Square;

pub struct GameState {
    board: Board,
    turn: Color,
    halfmoves: u8,
    fullmoves: u16,
    enpassant: Option<Square>
}

pub struct FENParseError;

impl std::str::FromStr for GameState {
    type Err = FENParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!();
    }
}

impl GameState {
    
}