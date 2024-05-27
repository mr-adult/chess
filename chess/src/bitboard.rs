use std::{array::from_fn, fmt::Debug};

use chess_common::{File, Rank};

#[derive(Clone, Default)]
pub(crate) struct BitBoard(pub(crate) u64);

impl BitBoard {

}

impl Debug for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result: [String; 8] = from_fn(|_| String::with_capacity(0));
        for rank in Rank::all_ranks_ascending() {
            match rank {
                Rank::One => {
                    let bits = (self.0 & Rank::one_bit_filter()) as u8;
                    result[rank.as_index()] = format!("{:08b}", bits);
                }
                Rank::Two => {
                    let bits = (self.0 & Rank::two_bit_filter()) >> 8 as u8;
                    result[rank.as_index()] = format!("{:08b}", bits);
                }
                Rank::Three=> {
                    let bits = (self.0 & Rank::three_bit_filter()) >> 16 as u8;
                    result[rank.as_index()] = format!("{:08b}", bits);
                }
                Rank::Four => {
                    let bits = (self.0 & Rank::four_bit_filter()) >> 24 as u8;
                    result[rank.as_index()] = format!("{:08b}", bits);
                }
                Rank::Five=> {
                    let bits = (self.0 & Rank::five_bit_filter()) >> 32 as u8;
                    result[rank.as_index()] = format!("{:08b}", bits);
                }
                Rank::Six=> {
                    let bits = (self.0 & Rank::six_bit_filter()) >> 40 as u8;
                    result[rank.as_index()] = format!("{:08b}", bits);
                }
                Rank::Seven=> {
                    let bits = (self.0 & Rank::seven_bit_filter()) >> 48 as u8;
                    result[rank.as_index()] = format!("{:08b}", bits);
                }
                Rank::Eight=> {
                    let bits = (self.0 & Rank::eight_bit_filter()) >> 56 as u8;
                    result[rank.as_index()] = format!("{:08b}", bits);
                }
            }
        }

        let mut result_string = '\n'.to_string();
        result_string.push_str(&result.join("\n"));
        result_string.push('\n');
        write!(f, "{}", result_string)
    }
}
