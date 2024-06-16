use std::array::IntoIter;

use serde::{de::Visitor, Deserialize, Serialize};

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

pub type RanksIterator = IntoIter<Rank, 8>;
impl Rank {
    pub fn all_ranks_ascending() -> RanksIterator {
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
    pub const fn one_through_four_bit_filter() -> u64 {
        Rank::one_bit_filter()
            | Rank::two_bit_filter()
            | Rank::three_bit_filter()
            | Rank::four_bit_filter()
    }

    #[inline]
    pub const fn five_through_eight_bit_filter() -> u64 {
        Rank::five_bit_filter()
            | Rank::six_bit_filter()
            | Rank::seven_bit_filter()
            | Rank::eight_bit_filter()
    }

    #[inline]
    pub const fn one_or_two_bit_filter() -> u64 {
        Rank::one_bit_filter() | Rank::two_bit_filter()
    }

    #[inline]
    pub const fn three_or_four_bit_filter() -> u64 {
        Rank::three_bit_filter() | Rank::four_bit_filter()
    }

    #[inline]
    pub const fn five_or_six_bit_filter() -> u64 {
        Rank::five_bit_filter() | Rank::six_bit_filter()
    }

    #[inline]
    pub const fn seven_or_eight_bit_filter() -> u64 {
        Rank::seven_bit_filter() | Rank::eight_bit_filter()
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

    fn try_from_i128(i128: i128) -> Option<Self> {
        if i128 < 1 || i128 > 8 {
            return None;
        }
        return Some(Self::try_from(i128 as i8 - 1).unwrap());
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
        S: serde::Serializer,
    {
        serializer.serialize_i8(self.as_int())
    }
}

impl<'de> Deserialize<'de> for Rank {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let visitor = RankDeserializeVisitor;
        deserializer.deserialize_i8(&visitor)
    }
}

struct RankDeserializeVisitor;
impl RankDeserializeVisitor {
    const fn err_message() -> &'static str {
        "Expected an integer between 0 and 8."
    }
}

impl<'de> Visitor<'de> for &RankDeserializeVisitor {
    type Value = Rank;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an integer between 1 and 8.")
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i128(v as i128)
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i128(v as i128)
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i128(v as i128)
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i128(v as i128)
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v > i128::MAX as u128 {
            Err(E::custom(RankDeserializeVisitor::err_message()))
        } else {
            self.visit_i128(v as i128)
        }
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i128(v as i128)
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i128(v as i128)
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i128(v as i128)
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i128(v as i128)
    }

    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v > 0 && v <= 8 {
            return Ok(Rank::try_from_i128(v).unwrap());
        } else {
            return Err(E::custom(RankDeserializeVisitor::err_message()));
        }
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Rank::try_from(v).map_err(|_| E::custom(RankDeserializeVisitor::err_message()))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.len() != 0 {
            return Err(E::custom(RankDeserializeVisitor::err_message()));
        }
        match v.chars().next() {
            None => return Err(E::custom(RankDeserializeVisitor::err_message())),
            Some(ch) => {
                return Ok(Rank::try_from(ch)
                    .map_err(|_| E::custom(RankDeserializeVisitor::err_message()))?);
            }
        }
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(&v)
    }
}
