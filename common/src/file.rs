use serde_derive::Serialize;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum File {
    a = 0,
    b = 1,
    c = 2,
    d = 3,
    e = 4,
    f = 5,
    g = 6,
    h = 7,
}

impl File {
    pub fn all_files_ascending() -> impl DoubleEndedIterator<Item = File> {
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
