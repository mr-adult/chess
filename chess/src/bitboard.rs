use std::{
    array::from_fn,
    fmt::Debug,
    ops::{BitAnd, BitOr, BitXor},
};

use chess_common::{File, Rank};

#[cfg(debug_assertions)]
use chess_common::Location;

pub(crate) struct BitBoard(
    pub(crate) u64,
    #[allow(unused)]
    #[cfg(debug_assertions)]
    Location,
);

impl BitBoard {
    pub(crate) fn new(value: u64) -> Self {
        BitBoard(
            value,
            #[cfg(debug_assertions)]
            Location::try_from(value).unwrap_or(Location::new(File::a, Rank::One)),
        )
    }

    pub(crate) fn left(&self) -> Self {
        Self::new(self.0.wrapping_shr(1) & !File::h_bit_filter())
    }

    pub(crate) fn right(&self) -> Self {
        Self::new(self.0.wrapping_shl(1) & !File::a_bit_filter())
    }

    pub(crate) fn up(&self) -> Self {
        Self::new(self.0.wrapping_shl(8))
    }

    pub(crate) fn down(&self) -> Self {
        Self::new(self.0.wrapping_shr(8))
    }

    pub(crate) fn up_left(&self) -> Self {
        Self::new(self.0.wrapping_shl(7) & !File::h_bit_filter())
    }

    pub(crate) fn up_right(&self) -> Self {
        Self::new(self.0.wrapping_shl(9) & !File::a_bit_filter())
    }

    pub(crate) fn down_left(&self) -> Self {
        Self::new(self.0.wrapping_shr(9) & !File::h_bit_filter())
    }

    pub(crate) fn down_right(&self) -> Self {
        Self::new(self.0.wrapping_shr(7) & !File::a_bit_filter())
    }

    pub(crate) fn intersects_with(&self, other: &BitBoard) -> bool {
        self.intersects_with_u64(other.0)
    }

    pub(crate) fn intersects_with_u64(&self, other: u64) -> bool {
        (self.0 & other) != 0
    }
}

impl BitAnd for BitBoard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::new(self.0 & rhs.0)
    }
}

impl BitOr for BitBoard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::new(self.0 | rhs.0)
    }
}

impl BitXor for BitBoard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::new(self.0 ^ rhs.0)
    }
}

impl Default for BitBoard {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Debug for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const LEN: usize = 9;
        const PADDING: usize = 2;
        let mut result: [String; LEN] = from_fn(|i| {
            if i == LEN - 1 {
                "  abcdefgh".to_string()
            } else {
                String::with_capacity(0)
            }
        });

        let format_byte = |byte: u8| {
            let mut result = String::with_capacity(8);
            for file in File::all_files_ascending() {
                if file.bit_filter() & byte as u64 != 0 {
                    result.push('1');
                } else {
                    result.push('0');
                }
            }
            result
        };

        for rank in Rank::all_ranks_ascending().rev() {
            match rank {
                Rank::One => {
                    let bits = (self.0 & Rank::one_bit_filter()) as u8;
                    result[LEN - rank.as_index() - PADDING] = format!("1 {}", format_byte(bits));
                }
                Rank::Two => {
                    let bits = ((self.0 & Rank::two_bit_filter()) >> 8) as u8;
                    result[LEN - rank.as_index() - PADDING] = format!("2 {}", format_byte(bits));
                }
                Rank::Three => {
                    let bits = ((self.0 & Rank::three_bit_filter()) >> 16) as u8;
                    result[LEN - rank.as_index() - PADDING] = format!("3 {}", format_byte(bits));
                }
                Rank::Four => {
                    let bits = ((self.0 & Rank::four_bit_filter()) >> 24) as u8;
                    result[LEN - rank.as_index() - PADDING] = format!("4 {}", format_byte(bits));
                }
                Rank::Five => {
                    let bits = ((self.0 & Rank::five_bit_filter()) >> 32) as u8;
                    result[LEN - rank.as_index() - PADDING] = format!("5 {}", format_byte(bits));
                }
                Rank::Six => {
                    let bits = ((self.0 & Rank::six_bit_filter()) >> 40) as u8;
                    result[LEN - rank.as_index() - PADDING] = format!("6 {}", format_byte(bits));
                }
                Rank::Seven => {
                    let bits = ((self.0 & Rank::seven_bit_filter()) >> 48) as u8;
                    result[LEN - rank.as_index() - PADDING] = format!("7 {}", format_byte(bits));
                }
                Rank::Eight => {
                    let bits = ((self.0 & Rank::eight_bit_filter()) >> 56) as u8;
                    result[LEN - rank.as_index() - PADDING] = format!("8 {}", format_byte(bits));
                }
            }
        }

        let mut result_string = '\n'.to_string();
        result_string.push_str(&result.join("\n"));
        result_string.push('\n');
        write!(f, "{}", result_string)
    }
}

impl Clone for BitBoard {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}
