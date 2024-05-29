use std::array::IntoIter;

use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Rank {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
}

impl Rank {
    pub fn all_ranks_ascending() -> IntoIter<Self, 8> {
        [
            Self::One,
            Self::Two,
            Self::Three,
            Self::Four,
            Self::Five,
            Self::Six,
            Self::Seven,
            Self::Eight,
        ]
        .into_iter()
    }

    pub fn as_char(self) -> char {
        match self {
            Self::One => '1',
            Self::Two => '2',
            Self::Three => '3',
            Self::Four => '4',
            Self::Five => '5',
            Self::Six => '6',
            Self::Seven => '7',
            Self::Eight => '8',
        }
    }

    pub fn as_index(self) -> usize {
        match self {
            Self::One => 0,
            Self::Two => 1,
            Self::Three => 2,
            Self::Four => 3,
            Self::Five => 4,
            Self::Six => 5,
            Self::Seven => 6,
            Self::Eight => 7,
        }
    }

    pub fn as_int(self) -> i8 {
        match self {
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
            Self::Five => 5,
            Self::Six => 6,
            Self::Seven => 7,
            Self::Eight => 8,
        }
    }

    pub const fn bit_filter(&self) -> u64 {
        match self {
            Self::One => Self::one_bit_filter(),
            Self::Two => Self::two_bit_filter(),
            Self::Three => Self::three_bit_filter(),
            Self::Four => Self::four_bit_filter(),
            Self::Five => Self::five_bit_filter(),
            Self::Six => Self::six_bit_filter(),
            Self::Seven => Self::seven_bit_filter(),
            Self::Eight => Self::eight_bit_filter(),
        }
    }

    #[inline]    
    pub const fn one_bit_filter() -> u64 {
        0x00_00_00_00_00_00_00_FF
    }

    #[inline]    
    pub const fn two_bit_filter() -> u64 {
        0x00_00_00_00_00_00_FF_00
    }

    #[inline]    
    pub const fn three_bit_filter() -> u64 {
        0x00_00_00_00_00_FF_00_00
    }

    #[inline]    
    pub const fn four_bit_filter() -> u64 {
        0x00_00_00_00_FF_00_00_00
    }

    #[inline]    
    pub const fn five_bit_filter() -> u64 {
        0x00_00_00_FF_00_00_00_00
    }

    #[inline]    
    pub const fn six_bit_filter() -> u64 {
        0x00_00_FF_00_00_00_00_00
    }

    #[inline]    
    pub const fn seven_bit_filter() -> u64 {
        0x00_FF_00_00_00_00_00_00
    }

    #[inline]    
    pub const fn eight_bit_filter() -> u64 {
        0xFF_00_00_00_00_00_00_00
    }
}

impl TryFrom<i8> for Rank {
    type Error = ();
    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Rank::One),
            1 => Ok(Rank::Two),
            2 => Ok(Rank::Three),
            3 => Ok(Rank::Four),
            4 => Ok(Rank::Five),
            5 => Ok(Rank::Six),
            6 => Ok(Rank::Seven),
            7 => Ok(Rank::Eight),
            _ => Err(()),
        }
    }
}

impl TryFrom<u8> for Rank {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Rank::One),
            1 => Ok(Rank::Two),
            2 => Ok(Rank::Three),
            3 => Ok(Rank::Four),
            4 => Ok(Rank::Five),
            5 => Ok(Rank::Six),
            6 => Ok(Rank::Seven),
            7 => Ok(Rank::Eight),
            _ => Err(()),
        }
    }
}

impl TryFrom<char> for Rank {
    type Error = ();
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '1' => Ok(Rank::One),
            '2' => Ok(Rank::Two),
            '3' => Ok(Rank::Three),
            '4' => Ok(Rank::Four),
            '5' => Ok(Rank::Five),
            '6' => Ok(Rank::Six),
            '7' => Ok(Rank::Seven),
            '8' => Ok(Rank::Eight),
            _ => Err(()),
        }
    }
}

impl Serialize for Rank {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        serializer.serialize_i8(self.as_int())
    }
}
