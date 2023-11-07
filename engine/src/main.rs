use types::sliders::common::TABLE_SIZE;
mod board;


include!(concat!(env!("OUT_DIR"), "/slider_moves.rs"));

pub const TABLE: [u64; TABLE_SIZE] = get_table();

fn main() {}
