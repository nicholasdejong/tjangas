use crate::{piece::{Piece, Promotion}, square::Square};

pub struct Move {
    pub piece: Piece,
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Promotion>
}