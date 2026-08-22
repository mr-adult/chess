use std::{
    array::from_fn,
    fmt::Debug,
    ops::{BitAnd, BitOr, BitXor},
    simd::Simd,
};

use chess_common::{File, Rank};

pub(crate) struct BitBoard(
    pub(crate) u64,
    // #[allow(unused)]
    // #[cfg(debug_assertions)]
    // chess_common::Location,
);

impl BitBoard {
    pub(crate) const LEFT_DIR_RIGHT_SHIFT_OFFSET: u32 = 1;
    pub(crate) const UP_LEFT_DIR_LEFT_SHIFT_OFFSET: u32 = 7;
    pub(crate) const UP_DIR_LEFT_SHIFT_OFFSET: u32 = 8;
    pub(crate) const UP_RIGHT_DIR_LEFT_SHIFT_OFFSET: u32 = 9;

    pub(crate) const RIGHT_DIR_LEFT_SHIFT_OFFSET: u32 = 1;
    pub(crate) const DOWN_RIGHT_DIR_RIGHT_SHIFT_OFFSET: u32 = 7;
    pub(crate) const DOWN_DIR_RIGHT_SHIFT_OFFSET: u32 = 8;
    pub(crate) const DOWN_LEFT_DIR_RIGHT_SHIFT_OFFSET: u32 = 9;

    pub(crate) fn new(value: u64) -> Self {
        BitBoard(
            value,
            // #[cfg(debug_assertions)]
            // chess_common::Location::try_from(value).unwrap_or(chess_common::Location::new(File::a, Rank::One)),
        )
    }

    pub(crate) fn left(&self) -> Self {
        Self::new(Self::left_u64(self.0))
    }
    pub(crate) fn left_u64(value: u64) -> u64 {
        value.unbounded_shr(Self::LEFT_DIR_RIGHT_SHIFT_OFFSET) & !File::h_bit_filter()
    }
    pub(crate) fn simd_left<const N: usize>(value: Simd<u64, N>) -> Simd<u64, N> {
        value >> Simd::<u64, N>::splat(Self::LEFT_DIR_RIGHT_SHIFT_OFFSET as u64)
            & Self::bitwise_not_h_bit_filter_simd()
    }
    pub(crate) const fn bitwise_not_h_bit_filter_simd<const N: usize>() -> Simd<u64, N> {
        Simd::<u64, N>::splat(!File::h_bit_filter())
    }

    pub(crate) fn right(&self) -> Self {
        Self::new(Self::right_u64(self.0))
    }
    pub(crate) fn right_u64(value: u64) -> u64 {
        value.unbounded_shl(Self::RIGHT_DIR_LEFT_SHIFT_OFFSET) & !File::a_bit_filter()
    }
    pub(crate) fn simd_right<const N: usize>(value: Simd<u64, N>) -> Simd<u64, N> {
        value << Simd::<u64, N>::splat(Self::RIGHT_DIR_LEFT_SHIFT_OFFSET as u64)
            & Self::bitwise_not_a_bit_filter_simd()
    }
    pub(crate) const fn bitwise_not_a_bit_filter_simd<const N: usize>() -> Simd<u64, N> {
        Simd::<u64, N>::splat(!File::a_bit_filter())
    }

    pub(crate) fn up(&self) -> Self {
        Self::new(Self::up_u64(self.0))
    }
    pub(crate) fn up_u64(value: u64) -> u64 {
        value.unbounded_shl(Self::UP_DIR_LEFT_SHIFT_OFFSET)
    }
    pub(crate) fn simd_up<const N: usize>(value: Simd<u64, N>) -> Simd<u64, N> {
        value << Simd::<u64, N>::splat(Self::UP_DIR_LEFT_SHIFT_OFFSET as u64)
            & Simd::<u64, N>::splat(!Rank::one_bit_filter())
    }

    pub(crate) fn down(&self) -> Self {
        Self::new(Self::down_u64(self.0))
    }
    pub(crate) fn down_u64(value: u64) -> u64 {
        value.unbounded_shr(Self::DOWN_DIR_RIGHT_SHIFT_OFFSET)
    }
    pub(crate) fn simd_down<const N: usize>(value: Simd<u64, N>) -> Simd<u64, N> {
        value >> Simd::<u64, N>::splat(Self::DOWN_DIR_RIGHT_SHIFT_OFFSET as u64)
            & Simd::<u64, N>::splat(!Rank::eight_bit_filter())
    }

    pub(crate) fn up_left(&self) -> Self {
        Self::new(Self::up_left_u64(self.0))
    }
    pub(crate) fn up_left_u64(value: u64) -> u64 {
        value.unbounded_shl(Self::UP_LEFT_DIR_LEFT_SHIFT_OFFSET) & !File::h_bit_filter()
    }
    pub(crate) fn simd_up_left<const N: usize>(value: Simd<u64, N>) -> Simd<u64, N> {
        value << Simd::<u64, N>::splat(Self::UP_LEFT_DIR_LEFT_SHIFT_OFFSET as u64)
            & Self::bitwise_not_h_bit_filter_simd()
            & Simd::<u64, N>::splat(!Rank::one_bit_filter())
    }

    pub(crate) fn up_right(&self) -> Self {
        Self::new(Self::up_right_u64(self.0))
    }
    pub(crate) fn up_right_u64(value: u64) -> u64 {
        value.unbounded_shl(Self::UP_RIGHT_DIR_LEFT_SHIFT_OFFSET) & !File::a_bit_filter()
    }
    pub(crate) fn simd_up_right<const N: usize>(value: Simd<u64, N>) -> Simd<u64, N> {
        value << Simd::<u64, N>::splat(Self::UP_RIGHT_DIR_LEFT_SHIFT_OFFSET as u64)
            & Self::bitwise_not_a_bit_filter_simd()
            & Simd::<u64, N>::splat(!Rank::one_bit_filter())
    }

    pub(crate) fn down_left(&self) -> Self {
        Self::new(Self::down_left_u64(self.0))
    }
    pub(crate) fn down_left_u64(value: u64) -> u64 {
        value.unbounded_shr(Self::DOWN_LEFT_DIR_RIGHT_SHIFT_OFFSET) & !File::h_bit_filter()
    }
    pub(crate) fn simd_down_left<const N: usize>(value: Simd<u64, N>) -> Simd<u64, N> {
        value >> Simd::<u64, N>::splat(Self::DOWN_LEFT_DIR_RIGHT_SHIFT_OFFSET as u64)
            & Self::bitwise_not_h_bit_filter_simd()
            & Simd::<u64, N>::splat(!Rank::eight_bit_filter())
    }

    pub(crate) fn down_right(&self) -> Self {
        Self::new(Self::down_right_u64(self.0))
    }
    pub(crate) fn down_right_u64(value: u64) -> u64 {
        value.unbounded_shr(Self::DOWN_RIGHT_DIR_RIGHT_SHIFT_OFFSET) & !File::a_bit_filter()
    }
    pub(crate) fn simd_down_right<const N: usize>(value: Simd<u64, N>) -> Simd<u64, N> {
        value >> Simd::<u64, N>::splat(Self::DOWN_RIGHT_DIR_RIGHT_SHIFT_OFFSET as u64)
            & Self::bitwise_not_a_bit_filter_simd()
            & Simd::<u64, N>::splat(!Rank::eight_bit_filter())
    }

    pub(crate) fn intersects_with(&self, other: &BitBoard) -> bool {
        self.intersects_with_u64(other.0)
    }

    pub(crate) const fn intersects_with_u64(&self, other: u64) -> bool {
        (self.0 & other) != 0
    }

    pub(crate) fn bit_count(&self) -> i32 {
        let mut total = 0;

        let mut value = self.0;
        while value != 0 {
            if value & 1 != 0 {
                total += 1;
            }
            value = value >> 1;
        }

        total
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
