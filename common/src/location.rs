use serde_derive::Serialize;
use crate::{file::File, rank::Rank};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct Location {
    pub(crate) file: File,
    pub(crate) rank: Rank,
}

impl Location {
    pub fn new(file: File, rank: Rank) -> Location {
        Location {
            file,
            rank,
        }
    }

    pub const fn failed_from_usize_message() -> &'static str {
        "Expected only one bit to be populated."
    }

    pub const fn file(&self) -> File {
        self.file
    }

    pub const fn rank(&self) -> Rank {
        self.rank
    }

    pub const fn as_u64(&self) -> u64 {
        let starting: u64 = match self.file {
            File::a => 0b00000001,
            File::b => 0b00000010,
            File::c => 0b00000100,
            File::d => 0b00001000,
            File::e => 0b00010000,
            File::f => 0b00100000,
            File::g => 0b01000000,
            File::h => 0b10000000,
        };

        let shift: u16 = match self.rank {
            Rank::One => 0,
            Rank::Two => 8,
            Rank::Three => 16,
            Rank::Four => 24,
            Rank::Five => 32,
            Rank::Six => 40,
            Rank::Seven => 48,
            Rank::Eight => 56,
        };

        return starting << shift;
    }
}

impl TryFrom<u64> for Location {
    type Error = ();

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        // https://stackoverflow.com/questions/28303832/check-if-byte-has-more-than-one-bit-set
        if value == 0 || value & (value - 1) != 0 {
            return Err(())
        }

        let rank = if value & Rank::one_bit_filter() != 0 {
            Rank::One
        } else if value & Rank::two_bit_filter() != 0 {
            Rank::Two
        } else if value &Rank::three_bit_filter() != 0 {
            Rank::Three
        } else if value & Rank::four_bit_filter() != 0 {
            Rank::Four
        } else if value & Rank::five_bit_filter() !=0 {
            Rank::Five
        } else if value & Rank::six_bit_filter() != 0 {
            Rank::Six
        } else if value & Rank::seven_bit_filter() != 0 {
            Rank::Seven
        } else if value & Rank::eight_bit_filter() != 0 {
            Rank::Eight
        } else {
            unreachable!()
        };

        let file = if value & File::a_bit_filter() != 0 {
            File::a
        } else if value & File::b_bit_filter() != 0 {
            File::b
        } else if value & File::c_bit_filter() != 0 {
            File::c
        } else if value & File::d_bit_filter() != 0 {
            File::d
        } else if value & File::e_bit_filter() != 0 {
            File::e
        } else if value & File::f_bit_filter() != 0 {
            File::f
        } else if value & File::g_bit_filter() != 0 {
            File::g
        } else if value & File::h_bit_filter() != 0 {
            File::h
        } else {
            unreachable!()
        };

        Ok(Location::new(file, rank))
    }
}

// 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000

#[cfg(test)]
mod tests {
    use crate::{File, Location, Rank};

    #[test]
    fn bit_repr() {
        assert!(Location::new(File::a, Rank::One).as_u64() == 0x1_u64);
        assert!(Location::new(File::e, Rank::Seven).as_u64() == 0x00_10_00_00_00_00_00_00);
        assert!(Location::new(File::h, Rank::Eight).as_u64() == 0x80_00_00_00_00_00_00_00);
    }
}