use std::vec::IntoIter;

use crate::{file::File, rank::Rank, Player};
use arr_deque::ArrDeque;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub(crate) file: File,
    pub(crate) rank: Rank,
}

impl Location {
    pub const fn new(file: File, rank: Rank) -> Self {
        Self { file, rank }
    }

    pub const fn king_starting(player: &Player) -> Self {
        Self::new(File::king_starting(), Rank::castle(player))
    }

    pub fn all_locations() -> impl Iterator<Item = Self> {
        Rank::all_ranks_ascending()
            .flat_map(|rank| File::all_files_ascending().map(move |file| Location::new(file, rank)))
    }

    pub const fn failed_from_usize_message() -> &'static str {
        "Expected only one bit to be populated."
    }

    pub fn from_bitboard(bitboard: u64) -> ArrDeque<Location, 64> {
        if bitboard & u64::MAX == 0 {
            return ArrDeque::new()
        }

        let mut arr_deque = ArrDeque::new();

        if bitboard & 0x1 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::a, Rank::One))
                .is_ok());
        }
        if bitboard & 0x2 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::b, Rank::One))
                .is_ok());
        }
        if bitboard & 0x4 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::c, Rank::One))
                .is_ok());
        }
        if bitboard & 0x8 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::d, Rank::One))
                .is_ok());
        }
        if bitboard & 0x10 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::e, Rank::One))
                .is_ok());
        }
        if bitboard & 0x20 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::f, Rank::One))
                .is_ok());
        }
        if bitboard & 0x40 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::g, Rank::One))
                .is_ok());
        }
        if bitboard & 0x80 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::h, Rank::One))
                .is_ok());
        }
        if bitboard & 0x100 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::a, Rank::Two))
                .is_ok());
        }
        if bitboard & 0x200 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::b, Rank::Two))
                .is_ok());
        }
        if bitboard & 0x400 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::c, Rank::Two))
                .is_ok());
        }
        if bitboard & 0x800 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::d, Rank::Two))
                .is_ok());
        }
        if bitboard & 0x1000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::e, Rank::Two))
                .is_ok());
        }
        if bitboard & 0x2000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::f, Rank::Two))
                .is_ok());
        }
        if bitboard & 0x4000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::g, Rank::Two))
                .is_ok());
        }
        if bitboard & 0x8000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::h, Rank::Two))
                .is_ok());
        }
        if bitboard & 0x10000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::a, Rank::Three))
                .is_ok());
        }
        if bitboard & 0x20000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::b, Rank::Three))
                .is_ok());
        }
        if bitboard & 0x40000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::c, Rank::Three))
                .is_ok());
        }
        if bitboard & 0x80000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::d, Rank::Three))
                .is_ok());
        }
        if bitboard & 0x100000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::e, Rank::Three))
                .is_ok());
        }
        if bitboard & 0x200000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::f, Rank::Three))
                .is_ok());
        }
        if bitboard & 0x400000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::g, Rank::Three))
                .is_ok());
        }
        if bitboard & 0x800000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::h, Rank::Three))
                .is_ok());
        }
        if bitboard & 0x1000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::a, Rank::Four))
                .is_ok());
        }
        if bitboard & 0x2000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::b, Rank::Four))
                .is_ok());
        }
        if bitboard & 0x4000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::c, Rank::Four))
                .is_ok());
        }
        if bitboard & 0x8000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::d, Rank::Four))
                .is_ok());
        }
        if bitboard & 0x10000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::e, Rank::Four))
                .is_ok());
        }
        if bitboard & 0x20000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::f, Rank::Four))
                .is_ok());
        }
        if bitboard & 0x40000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::g, Rank::Four))
                .is_ok());
        }
        if bitboard & 0x80000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::h, Rank::Four))
                .is_ok());
        }
        if bitboard & 0x100000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::a, Rank::Five))
                .is_ok());
        }
        if bitboard & 0x200000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::b, Rank::Five))
                .is_ok());
        }
        if bitboard & 0x400000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::c, Rank::Five))
                .is_ok());
        }
        if bitboard & 0x800000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::d, Rank::Five))
                .is_ok());
        }
        if bitboard & 0x1000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::e, Rank::Five))
                .is_ok());
        }
        if bitboard & 0x2000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::f, Rank::Five))
                .is_ok());
        }
        if bitboard & 0x4000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::g, Rank::Five))
                .is_ok());
        }
        if bitboard & 0x8000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::h, Rank::Five))
                .is_ok());
        }
        if bitboard & 0x10000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::a, Rank::Six))
                .is_ok());
        }
        if bitboard & 0x20000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::b, Rank::Six))
                .is_ok());
        }
        if bitboard & 0x40000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::c, Rank::Six))
                .is_ok());
        }
        if bitboard & 0x80000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::d, Rank::Six))
                .is_ok());
        }
        if bitboard & 0x100000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::e, Rank::Six))
                .is_ok());
        }
        if bitboard & 0x200000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::f, Rank::Six))
                .is_ok());
        }
        if bitboard & 0x400000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::g, Rank::Six))
                .is_ok());
        }
        if bitboard & 0x800000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::h, Rank::Six))
                .is_ok());
        }
        if bitboard & 0x1000000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::a, Rank::Seven))
                .is_ok());
        }
        if bitboard & 0x2000000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::b, Rank::Seven))
                .is_ok());
        }
        if bitboard & 0x4000000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::c, Rank::Seven))
                .is_ok());
        }
        if bitboard & 0x8000000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::d, Rank::Seven))
                .is_ok());
        }
        if bitboard & 0x10000000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::e, Rank::Seven))
                .is_ok());
        }
        if bitboard & 0x20000000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::f, Rank::Seven))
                .is_ok());
        }
        if bitboard & 0x40000000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::g, Rank::Seven))
                .is_ok());
        }
        if bitboard & 0x80000000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::h, Rank::Seven))
                .is_ok());
        }
        if bitboard & 0x100000000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::a, Rank::Eight))
                .is_ok());
        }
        if bitboard & 0x200000000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::b, Rank::Eight))
                .is_ok());
        }
        if bitboard & 0x400000000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::c, Rank::Eight))
                .is_ok());
        }
        if bitboard & 0x800000000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::d, Rank::Eight))
                .is_ok());
        }
        if bitboard & 0x1000000000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::e, Rank::Eight))
                .is_ok());
        }
        if bitboard & 0x2000000000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::f, Rank::Eight))
                .is_ok());
        }
        if bitboard & 0x4000000000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::g, Rank::Eight))
                .is_ok());
        }
        if bitboard & 0x8000000000000000 != 0 {
            assert!(arr_deque
                .push_back(Location::new(File::h, Rank::Eight))
                .is_ok());
        }

        return arr_deque;
    }

    pub const fn file(&self) -> File {
        self.file
    }

    pub const fn rank(&self) -> Rank {
        self.rank
    }

    pub const fn as_u64(&self) -> u64 {
        self.file.bit_filter() & self.rank.bit_filter()
    }
}

impl TryFrom<u64> for Location {
    type Error = ();

    #[inline]
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0x0000000000000001 => Ok(Location::new(File::a, Rank::One)),
            0x0000000000000002 => Ok(Location::new(File::b, Rank::One)),
            0x0000000000000004 => Ok(Location::new(File::c, Rank::One)),
            0x0000000000000008 => Ok(Location::new(File::d, Rank::One)),
            0x0000000000000010 => Ok(Location::new(File::e, Rank::One)),
            0x0000000000000020 => Ok(Location::new(File::f, Rank::One)),
            0x0000000000000040 => Ok(Location::new(File::g, Rank::One)),
            0x0000000000000080 => Ok(Location::new(File::h, Rank::One)),
            0x0000000000000100 => Ok(Location::new(File::a, Rank::Two)),
            0x0000000000000200 => Ok(Location::new(File::b, Rank::Two)),
            0x0000000000000400 => Ok(Location::new(File::c, Rank::Two)),
            0x0000000000000800 => Ok(Location::new(File::d, Rank::Two)),
            0x0000000000001000 => Ok(Location::new(File::e, Rank::Two)),
            0x0000000000002000 => Ok(Location::new(File::f, Rank::Two)),
            0x0000000000004000 => Ok(Location::new(File::g, Rank::Two)),
            0x0000000000008000 => Ok(Location::new(File::h, Rank::Two)),
            0x0000000000010000 => Ok(Location::new(File::a, Rank::Three)),
            0x0000000000020000 => Ok(Location::new(File::b, Rank::Three)),
            0x0000000000040000 => Ok(Location::new(File::c, Rank::Three)),
            0x0000000000080000 => Ok(Location::new(File::d, Rank::Three)),
            0x0000000000100000 => Ok(Location::new(File::e, Rank::Three)),
            0x0000000000200000 => Ok(Location::new(File::f, Rank::Three)),
            0x0000000000400000 => Ok(Location::new(File::g, Rank::Three)),
            0x0000000000800000 => Ok(Location::new(File::h, Rank::Three)),
            0x0000000001000000 => Ok(Location::new(File::a, Rank::Four)),
            0x0000000002000000 => Ok(Location::new(File::b, Rank::Four)),
            0x0000000004000000 => Ok(Location::new(File::c, Rank::Four)),
            0x0000000008000000 => Ok(Location::new(File::d, Rank::Four)),
            0x0000000010000000 => Ok(Location::new(File::e, Rank::Four)),
            0x0000000020000000 => Ok(Location::new(File::f, Rank::Four)),
            0x0000000040000000 => Ok(Location::new(File::g, Rank::Four)),
            0x0000000080000000 => Ok(Location::new(File::h, Rank::Four)),
            0x0000000100000000 => Ok(Location::new(File::a, Rank::Five)),
            0x0000000200000000 => Ok(Location::new(File::b, Rank::Five)),
            0x0000000400000000 => Ok(Location::new(File::c, Rank::Five)),
            0x0000000800000000 => Ok(Location::new(File::d, Rank::Five)),
            0x0000001000000000 => Ok(Location::new(File::e, Rank::Five)),
            0x0000002000000000 => Ok(Location::new(File::f, Rank::Five)),
            0x0000004000000000 => Ok(Location::new(File::g, Rank::Five)),
            0x0000008000000000 => Ok(Location::new(File::h, Rank::Five)),
            0x0000010000000000 => Ok(Location::new(File::a, Rank::Six)),
            0x0000020000000000 => Ok(Location::new(File::b, Rank::Six)),
            0x0000040000000000 => Ok(Location::new(File::c, Rank::Six)),
            0x0000080000000000 => Ok(Location::new(File::d, Rank::Six)),
            0x0000100000000000 => Ok(Location::new(File::e, Rank::Six)),
            0x0000200000000000 => Ok(Location::new(File::f, Rank::Six)),
            0x0000400000000000 => Ok(Location::new(File::g, Rank::Six)),
            0x0000800000000000 => Ok(Location::new(File::h, Rank::Six)),
            0x0001000000000000 => Ok(Location::new(File::a, Rank::Seven)),
            0x0002000000000000 => Ok(Location::new(File::b, Rank::Seven)),
            0x0004000000000000 => Ok(Location::new(File::c, Rank::Seven)),
            0x0008000000000000 => Ok(Location::new(File::d, Rank::Seven)),
            0x0010000000000000 => Ok(Location::new(File::e, Rank::Seven)),
            0x0020000000000000 => Ok(Location::new(File::f, Rank::Seven)),
            0x0040000000000000 => Ok(Location::new(File::g, Rank::Seven)),
            0x0080000000000000 => Ok(Location::new(File::h, Rank::Seven)),
            0x0100000000000000 => Ok(Location::new(File::a, Rank::Eight)),
            0x0200000000000000 => Ok(Location::new(File::b, Rank::Eight)),
            0x0400000000000000 => Ok(Location::new(File::c, Rank::Eight)),
            0x0800000000000000 => Ok(Location::new(File::d, Rank::Eight)),
            0x1000000000000000 => Ok(Location::new(File::e, Rank::Eight)),
            0x2000000000000000 => Ok(Location::new(File::f, Rank::Eight)),
            0x4000000000000000 => Ok(Location::new(File::g, Rank::Eight)),
            0x8000000000000000 => Ok(Location::new(File::h, Rank::Eight)),
            _ => return Err(()),
        }
    }
}

impl ToString for Location {
    fn to_string(&self) -> String {
        let mut result = String::with_capacity(2);
        result.push(self.file.as_char());
        result.push(self.rank.as_char());
        return result;
    }
}

#[cfg(test)]
mod tests {
    use crate::{File, Location, Rank};

    #[test]
    fn bit_repr() {
        assert!(Location::new(File::a, Rank::One).as_u64() == 0x1_u64);
        assert!(Location::new(File::e, Rank::Seven).as_u64() == 0x00_10_00_00_00_00_00_00);
        assert!(Location::new(File::h, Rank::Eight).as_u64() == 0x80_00_00_00_00_00_00_00);
    }

    #[test]
    fn try_from_gets_correct_locations() {
        assert_eq!(
            Location::new(File::a, Rank::Three),
            Location::try_from(File::a_bit_filter() & Rank::three_bit_filter()).unwrap()
        );
        assert_eq!(
            Location::new(File::g, Rank::Seven),
            Location::try_from(File::g_bit_filter() & Rank::seven_bit_filter()).unwrap()
        );
    }
}
