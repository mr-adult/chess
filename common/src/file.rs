use std::array::IntoIter;

use serde_derive::{Deserialize, Serialize};

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum File {
    a,
    b,
    c,
    d,
    e,
    f,
    g,
    h,
}

pub type FilesIterator = IntoIter<File, 8>;
impl File {
    pub fn all_files_ascending() -> FilesIterator {
        [
            Self::a,
            Self::b,
            Self::c,
            Self::d,
            Self::e,
            Self::f,
            Self::g,
            Self::h,
        ]
        .into_iter()
    }

    pub fn as_char(self) -> char {
        match self {
            Self::a => 'a',
            Self::b => 'b',
            Self::c => 'c',
            Self::d => 'd',
            Self::e => 'e',
            Self::f => 'f',
            Self::g => 'g',
            Self::h => 'h',
        }
    }

    pub fn as_index(self) -> usize {
        match self {
            Self::a => 0,
            Self::b => 1,
            Self::c => 2,
            Self::d => 3,
            Self::e => 4,
            Self::f => 5,
            Self::g => 6,
            Self::h => 7,
        }
    }

    pub fn as_int(self) -> i8 {
        match self {
            Self::a => 0,
            Self::b => 1,
            Self::c => 2,
            Self::d => 3,
            Self::e => 4,
            Self::f => 5,
            Self::g => 6,
            Self::h => 7,
        }
    }

    pub const fn bit_filter(&self) -> u64 {
        match self {
            Self::a => Self::a_bit_filter(),
            Self::b => Self::b_bit_filter(),
            Self::c => Self::c_bit_filter(),
            Self::d => Self::d_bit_filter(),
            Self::e => Self::e_bit_filter(),
            Self::f => Self::f_bit_filter(),
            Self::g => Self::g_bit_filter(),
            Self::h => Self::h_bit_filter(),
        }
    }

    #[inline]
    pub const fn a_through_d_bit_filter() -> u64 {
        Self::a_or_b_bit_filter() | Self::c_or_d_bit_filter()
    }

    #[inline]
    pub const fn e_through_h_bit_filter() -> u64 {
        Self::e_or_f_bit_filter() | Self::g_or_h_bit_filter()
    }

    #[inline]
    pub const fn a_or_b_bit_filter() -> u64 {
        Self::a_bit_filter() | Self::b_bit_filter()
    }

    #[inline]
    pub const fn c_or_d_bit_filter() -> u64 {
        Self::c_bit_filter() | Self::d_bit_filter()
    }

    #[inline]
    pub const fn e_or_f_bit_filter() -> u64 {
        Self::e_bit_filter() | Self::f_bit_filter()
    }

    #[inline]
    pub const fn g_or_h_bit_filter() -> u64 {
        Self::g_bit_filter() | Self::h_bit_filter()
    }

    #[inline]
    pub const fn a_bit_filter() -> u64 {
        0x01_01_01_01_01_01_01_01
    }

    #[inline]
    pub const fn b_bit_filter() -> u64 {
        0x02_02_02_02_02_02_02_02
    }

    #[inline]
    pub const fn c_bit_filter() -> u64 {
        0x04_04_04_04_04_04_04_04
    }

    #[inline]
    pub const fn d_bit_filter() -> u64 {
        0x08_08_08_08_08_08_08_08
    }

    #[inline]
    pub const fn e_bit_filter() -> u64 {
        0x10_10_10_10_10_10_10_10
    }

    #[inline]
    pub const fn f_bit_filter() -> u64 {
        0x20_20_20_20_20_20_20_20
    }

    #[inline]
    pub const fn g_bit_filter() -> u64 {
        0x40_40_40_40_40_40_40_40
    }

    #[inline]
    pub const fn h_bit_filter() -> u64 {
        0x80_80_80_80_80_80_80_80
    }
}

impl TryFrom<u8> for File {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(File::a),
            1 => Ok(File::b),
            2 => Ok(File::c),
            3 => Ok(File::d),
            4 => Ok(File::e),
            5 => Ok(File::f),
            6 => Ok(File::g),
            7 => Ok(File::h),
            _ => Err(()),
        }
    }
}

impl TryFrom<i8> for File {
    type Error = ();
    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(File::a),
            1 => Ok(File::b),
            2 => Ok(File::c),
            3 => Ok(File::d),
            4 => Ok(File::e),
            5 => Ok(File::f),
            6 => Ok(File::g),
            7 => Ok(File::h),
            _ => Err(()),
        }
    }
}

impl TryFrom<char> for File {
    type Error = ();
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'a' => Ok(File::a),
            'b' => Ok(File::b),
            'c' => Ok(File::c),
            'd' => Ok(File::d),
            'e' => Ok(File::e),
            'f' => Ok(File::f),
            'g' => Ok(File::g),
            'h' => Ok(File::h),
            _ => Err(()),
        }
    }
}
