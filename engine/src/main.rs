use types::sliders::common::TABLE_SIZE;

include!(concat!(env!("OUT_DIR"), "/slider_moves.rs"));

pub const TABLE: [u64; TABLE_SIZE] = get_table();

fn main() {}