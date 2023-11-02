use rand::prelude::*;
use std::{env, fs, path::Path};
use types::{square::Square, slider::Slider, bitboard::BitBoard};

#[derive(Clone, Copy, Debug, Default)]
pub struct Magic {
    hash: u64,
    index_bits: usize, // 64 - shift
}

#[derive(Clone)]
struct BitBoardList(Vec<BitBoard>);

impl std::fmt::Debug for BitBoardList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        for num in &self.0[0..self.0.len() - 1] {
            result.push_str(&num.display());
            result.push_str(", ");
        }

        result.push_str(&self.0[self.0.len() - 1].display());
        write!(f, "&[{result}]")
    }
}

impl Default for BitBoardList {
    fn default() -> Self {
        BitBoardList(vec![])
    }
}

// generate u64 with low active bit count
fn low_u64(rng: &mut ThreadRng) -> u64 {
    rng.gen::<u64>() & rng.gen::<u64>() & rng.gen::<u64>()
}

fn subsets(mask: BitBoard) -> Vec<BitBoard> {
    let mut res: Vec<BitBoard> = Vec::with_capacity(2usize.pow(mask.len()));
    let mut subset = BitBoard::EMPTY;
    loop {
        res.push(subset);
        subset = BitBoard(subset.0.wrapping_sub(mask.0) & mask.0);
        if subset.is_empty() {
            break;
        }
    }
    res
}

pub fn magic_index(blockers: BitBoard, magic: Magic) -> usize {
    (blockers.0.wrapping_mul(magic.hash) >> magic.index_bits) as usize
}

#[derive(Debug)]
struct HashCollision;

fn try_fill_table(magic: Magic, sq: Square, slider: Slider) -> Result<Vec<BitBoard>, HashCollision> {
    let blockers = slider.blockers(sq);
    let mut table: Vec<BitBoard> = vec![BitBoard::EMPTY; 2usize.pow(blockers.len())];
    for subset in subsets(blockers) {
        let moves = slider.pseudo_moves(sq, subset);
        let idx = magic_index(subset, magic);
        let entry = &mut table[idx];
        if entry.is_empty() {
            *entry = moves;
        } else if entry.0 != moves.0 {
            return Err(HashCollision);
        }
    }
    Ok(table)
}

fn find_magic(sq: Square, slider: Slider) -> Magic {
    let mut rng = thread_rng();
    let blockers = slider.blockers(sq);
    let mut magic = Magic {index_bits: 64 - blockers.len() as usize, ..Default::default()};
    loop {
        let hash = low_u64(&mut rng);
        if ((blockers.0 * hash) & 0xff00000000000000).count_ones() < 6 {
            continue;
        }
        magic.hash = hash;
        let result =  try_fill_table(magic, sq, slider);
        if result.is_ok() {
            println!("Found magic for {slider:?} at {}", sq.0);
            return magic;
        }
    }
}


fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("magics.rs");

    // let bishop_magics = {
    //     let mut magics: [Magic; Square::NUM] = [Magic::default(); Square::NUM];
    //     let mut i = 0;
    //     while i < Square::NUM {
    //         magics[i] = find_magic(Square(i), Slider::Bishop);
    //         i += 1;
    //     }
    //     magics
    // };
    let bishop_magics = [Magic { hash: 2315556112300245248, index_bits: 58 }, Magic { hash: 4504709138235424, index_bits: 59 }, Magic { hash: 9227959200443809792, index_bits: 59 }, Magic { hash: 4756375194556170304, index_bits: 59 }, Magic { hash: 1153220846647574594, index_bits: 59 }, Magic { hash: 2289252197466112, index_bits: 59 }, Magic { hash: 1225124239016853506, index_bits: 59 }, Magic { hash: 4613097796072376832, index_bits: 58 }, Magic { hash: 2322204396175941, index_bits: 59 }, Magic { hash: 576478628091551872, index_bits: 59 }, Magic { hash: 1585284678206636096, index_bits: 59 }, Magic { hash: 9872211457880361216, index_bits: 59 }, Magic { hash: 72411714364121377, index_bits: 59 }, Magic { hash: 26560220905537, index_bits: 59 }, Magic { hash: 720581472867753984, index_bits: 59 }, Magic { hash: 723179725665083520, index_bits: 59 }, Magic { hash: 6931338772088834, index_bits: 59 }, Magic { hash: 12666511611790852, index_bits: 59 }, Magic { hash: 4512503161692224, index_bits: 57 }, Magic { hash: 9512730546364485664, index_bits: 57 }, Magic { hash: 272749813198487713, index_bits: 57 }, Magic { hash: 1227301286536290314, index_bits: 57 }, Magic { hash: 4612830071080878914, index_bits: 59 }, Magic { hash: 9223516073448517640, index_bits: 59 }, Magic { hash: 581360451600778240, index_bits: 59 }, Magic { hash: 2310355416829681792, index_bits: 59 }, Magic { hash: 75041803338496, index_bits: 57 }, Magic { hash: 3459891515388594304, index_bits: 55 }, Magic { hash: 9152886760022021, index_bits: 55 }, Magic { hash: 1319555790608040064, index_bits: 57 }, Magic { hash: 432913358937722888, index_bits: 59 }, Magic { hash: 290545964435489, index_bits: 59 }, Magic { hash: 171453479626352645, index_bits: 59 }, Magic { hash: 5139187143541024, index_bits: 59 }, Magic { hash: 563156113031552, index_bits: 57 }, Magic { hash: 4612531577262506048, index_bits: 55 }, Magic { hash: 76636510749065728, index_bits: 55 }, Magic { hash: 4634234803426758662, index_bits: 57 }, Magic { hash: 2902570526776297729, index_bits: 59 }, Magic { hash: 36601101444719112, index_bits: 59 }, Magic { hash: 180457883109376, index_bits: 59 }, Magic { hash: 11567926960831343144, index_bits: 59 }, Magic { hash: 1162210454555070465, index_bits: 57 }, Magic { hash: 4503685862262784, index_bits: 57 }, Magic { hash: 13835093317164990592, index_bits: 57 }, Magic { hash: 1459309250143601160, index_bits: 57 }, Magic { hash: 27140485713757312, index_bits: 59 }, Magic { hash: 6792862308239872, index_bits: 59 }, Magic { hash: 4612816333644759112, index_bits: 59 }, Magic { hash: 18313745114079489, index_bits: 59 }, Magic { hash: 450678977943585304, index_bits: 59 }, Magic { hash: 577116199779958784, index_bits: 59 }, Magic { hash: 5765258459960573953, index_bits: 59 }, Magic { hash: 288898888919285793, index_bits: 59 }, Magic { hash: 1162494059013541888, index_bits: 59 }, Magic { hash: 20354161409327745, index_bits: 59 }, Magic { hash: 72339620179886084, index_bits: 58 }, Magic { hash: 360853127773291012, index_bits: 59 }, Magic { hash: 216172786417176704, index_bits: 59 }, Magic { hash: 2339764133560834, index_bits: 59 }, Magic { hash: 144185557088600584, index_bits: 59 }, Magic { hash: 6773484483221128256, index_bits: 59 }, Magic { hash: 648529410781610048, index_bits: 59 }, Magic { hash: 18016614997827840, index_bits: 58 }];

    // let rook_magics = {
    //     let mut magics: [Magic; Square::NUM] = [Magic::default(); Square::NUM];
    //     let mut i = 0;
    //     while i < Square::NUM {
    //         magics[i] = find_magic(Square(i), Slider::Rook);
    //         i += 1;
    //     }
    //     magics
    // };
    let rook_magics = [Magic { hash: 4719772480408403968, index_bits: 52 }, Magic { hash: 18014535952633856, index_bits: 53 }, Magic { hash: 13871104446635450368, index_bits: 53 }, Magic { hash: 1261025487925348352, index_bits: 53 }, Magic { hash: 72062026445291522, index_bits: 53 }, Magic { hash: 324263573367685137, index_bits: 53 }, Magic { hash: 36030996058996864, index_bits: 53 }, Magic { hash: 72072438520807426, index_bits: 52 }, Magic { hash: 9367627989263835520, index_bits: 53 }, Magic { hash: 70369282097216, index_bits: 54 }, Magic { hash: 150307725614645536, index_bits: 54 }, Magic { hash: 9241667948997775360, index_bits: 54 }, Magic { hash: 281492290801920, index_bits: 54 }, Magic { hash: 36170084296688640, index_bits: 54 }, Magic { hash: 1450722038556722184, index_bits: 54 }, Magic { hash: 1299569995541381442, index_bits: 53 }, Magic { hash: 150083341385776, index_bits: 53 }, Magic { hash: 2323858095052189696, index_bits: 54 }, Magic { hash: 2449976890063062272, index_bits: 54 }, Magic { hash: 13511348914751488, index_bits: 54 }, Magic { hash: 1369235574099018752, index_bits: 54 }, Magic { hash: 577869226841342976, index_bits: 54 }, Magic { hash: 11544872226000897, index_bits: 54 }, Magic { hash: 85761974370369, index_bits: 53 }, Magic { hash: 2348662941949378568, index_bits: 53 }, Magic { hash: 2286985260060673, index_bits: 54 }, Magic { hash: 4620702023439876160, index_bits: 54 }, Magic { hash: 4503741361819904, index_bits: 54 }, Magic { hash: 72339090490525697, index_bits: 54 }, Magic { hash: 18577357053104136, index_bits: 54 }, Magic { hash: 613333978547749376, index_bits: 54 }, Magic { hash: 564332933710116, index_bits: 53 }, Magic { hash: 18181799163199520, index_bits: 53 }, Magic { hash: 40533633609498688, index_bits: 54 }, Magic { hash: 12396175603038953472, index_bits: 54 }, Magic { hash: 2310351007965513728, index_bits: 54 }, Magic { hash: 288239172328624385, index_bits: 54 }, Magic { hash: 19140307014648960, index_bits: 54 }, Magic { hash: 18594945011025986, index_bits: 54 }, Magic { hash: 1225260865779925125, index_bits: 53 }, Magic { hash: 70370891694112, index_bits: 53 }, Magic { hash: 9223654062661369890, index_bits: 54 }, Magic { hash: 962714935132553232, index_bits: 54 }, Magic { hash: 2309229788530868240, index_bits: 54 }, Magic { hash: 161004237508577280, index_bits: 54 }, Magic { hash: 149181806913258496, index_bits: 54 }, Magic { hash: 27023801216925712, index_bits: 54 }, Magic { hash: 580964852311392260, index_bits: 53 }, Magic { hash: 282025273721344, index_bits: 53 }, Magic { hash: 1337569364225819200, index_bits: 54 }, Magic { hash: 72198400285606528, index_bits: 54 }, Magic { hash: 144129482264936704, index_bits: 54 }, Magic { hash: 9369180472971657344, index_bits: 54 }, Magic { hash: 4644345705922688, index_bits: 54 }, Magic { hash: 90212734330863744, index_bits: 54 }, Magic { hash: 281682217926912, index_bits: 53 }, Magic { hash: 1729946325721563190, index_bits: 52 }, Magic { hash: 18015086023622785, index_bits: 53 }, Magic { hash: 2270492048230465, index_bits: 53 }, Magic { hash: 614178536760053862, index_bits: 53 }, Magic { hash: 9655999144812349441, index_bits: 53 }, Magic { hash: 2384093087121015874, index_bits: 53 }, Magic { hash: 1152922776185733252, index_bits: 53 }, Magic { hash: 1801511320286400546, index_bits: 52 }];

    let bishop_moves = {
        // let mut moves: [Vec<BitBoard>; Square::NUM] = vec![Vec::new(); Square::NUM].try_into().expect("static");
        let mut moves: [BitBoardList; Square::NUM] = vec![BitBoardList::default(); Square::NUM].try_into().unwrap();
        let mut i = 0;
        while i < Square::NUM {
            moves[i] = BitBoardList(try_fill_table(bishop_magics[i], Square(i), Slider::Bishop).unwrap());
            i += 1;
        }
        moves
    };

    let rook_moves = {
        // let mut moves: [Vec<BitBoard>; Square::NUM] = vec![Vec::new(); Square::NUM].try_into().expect("static");
        let mut moves: [BitBoardList; Square::NUM] = vec![BitBoardList::default(); Square::NUM].try_into().expect("static");
        let mut i = 0;
        while i < Square::NUM {
            moves[i] = BitBoardList(try_fill_table(rook_magics[i], Square(i), Slider::Rook).unwrap());
            i += 1;
        }
        moves
    };

    // let bishop_moves = BitBoardList(bishop_moves);
    // let rook_moves = BitBoardList(rook_moves);

    fs::write(
        &dest_path,
        format!("
        pub const fn bishop_magics() -> [Magic; Square::NUM] {{ {bishop_magics:?} }}
        pub const fn rook_magics() -> [Magic; Square::NUM] {{ {rook_magics:?} }}
        
        pub const fn bishop_moves() -> [Vec<BitBoard>; Square::NUM] {{ {bishop_moves:?} }}
        pub const fn rook_moves() -> [Vec<BitBoard>; Square::NUM] {{ {rook_moves:?} }}
        ")
    ).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}


// fn main() {}