use std::vec::IntoIter;

use crate::{file::File, rank::Rank};
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub(crate) file: File,
    pub(crate) rank: Rank,
}

impl Location {
    pub const fn new(file: File, rank: Rank) -> Self {
        Self { file, rank }
    }

    pub fn all_locations() -> impl Iterator<Item = Self> {
        Rank::all_ranks_ascending()
            .flat_map(|rank| File::all_files_ascending().map(move |file| Location::new(file, rank)))
    }

    pub const fn failed_from_usize_message() -> &'static str {
        "Expected only one bit to be populated."
    }

    pub fn from_bitboard(bitboard: u64) -> IntoIter<Self> {
        if bitboard & u64::MAX == 0 {
            return Vec::new().into_iter();
        }

        let mut ranks = Vec::with_capacity(8); // Pre-allocate for the worst case (1 piece per rank).
        if bitboard & Rank::one_through_four_bit_filter() != 0 {
            if bitboard & Rank::one_or_two_bit_filter() != 0 {
                if bitboard & Rank::one_bit_filter() != 0 {
                    ranks.push(Rank::One);
                }
                if bitboard & Rank::two_bit_filter() != 0 {
                    ranks.push(Rank::Two);
                }
            }
            if bitboard & Rank::three_or_four_bit_filter() != 0 {
                if bitboard & Rank::three_bit_filter() != 0 {
                    ranks.push(Rank::Three);
                }
                if bitboard & Rank::four_bit_filter() != 0 {
                    ranks.push(Rank::Four);
                }
            }
        }
        if bitboard & Rank::five_through_eight_bit_filter() != 0 {
            if bitboard & Rank::five_or_six_bit_filter() != 0 {
                if bitboard & Rank::five_bit_filter() != 0 {
                    ranks.push(Rank::Five);
                }
                if bitboard & Rank::six_bit_filter() != 0 {
                    ranks.push(Rank::Six);
                }
            }
            if bitboard & Rank::seven_or_eight_bit_filter() != 0 {
                if bitboard & Rank::seven_bit_filter() != 0 {
                    ranks.push(Rank::Seven);
                }
                if bitboard & Rank::eight_bit_filter() != 0 {
                    ranks.push(Rank::Eight);
                }
            }
        }

        // pre-allocate for the expected worst case (8 pawns)
        let mut locations = Vec::with_capacity(8);
        for rank in ranks {
            let filtered_to_rank = bitboard & rank.bit_filter();
            if filtered_to_rank & File::a_through_d_bit_filter() != 0 {
                if filtered_to_rank & File::a_or_b_bit_filter() != 0 {
                    if filtered_to_rank & File::a_bit_filter() != 0 {
                        locations.push(Location {
                            file: File::a,
                            rank,
                        });
                    }
                    if filtered_to_rank & File::b_bit_filter() != 0 {
                        locations.push(Location {
                            file: File::b,
                            rank,
                        });
                    }
                }
                if filtered_to_rank & File::c_or_d_bit_filter() != 0 {
                    if filtered_to_rank & File::c_bit_filter() != 0 {
                        locations.push(Location {
                            file: File::c,
                            rank,
                        });
                    }
                    if filtered_to_rank & File::d_bit_filter() != 0 {
                        locations.push(Location {
                            file: File::d,
                            rank,
                        });
                    }
                }
            }
            if filtered_to_rank & File::e_through_h_bit_filter() != 0 {
                if filtered_to_rank & File::e_or_f_bit_filter() != 0 {
                    if filtered_to_rank & File::e_bit_filter() != 0 {
                        locations.push(Location {
                            file: File::e,
                            rank,
                        });
                    }
                    if filtered_to_rank & File::f_bit_filter() != 0 {
                        locations.push(Location {
                            file: File::f,
                            rank,
                        });
                    }
                }
                if filtered_to_rank & File::g_or_h_bit_filter() != 0 {
                    if filtered_to_rank & File::g_bit_filter() != 0 {
                        locations.push(Location {
                            file: File::g,
                            rank,
                        });
                    }
                    if filtered_to_rank & File::h_bit_filter() != 0 {
                        locations.push(Location {
                            file: File::h,
                            rank,
                        });
                    }
                }
            }
        }

        return locations.into_iter();
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

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        // https://stackoverflow.com/questions/28303832/check-if-byte-has-more-than-one-bit-set
        if value == 0 || value & (value - 1) != 0 {
            return Err(());
        }

        let mut iter = Location::from_bitboard(value);
        if let Some(value) = iter.next() {
            let value = value;
            if iter.next().is_some() {
                return Err(());
            }
            return Ok(value);
        }
        return Err(());
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
