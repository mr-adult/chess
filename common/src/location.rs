use std::vec::IntoIter;

use crate::{file::File, rank::Rank};
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub(crate) file: File,
    pub(crate) rank: Rank,
}

impl Location {
    pub fn new(file: File, rank: Rank) -> Location {
        Location { file, rank }
    }

    pub fn all_locations() -> impl Iterator<Item = Location> {
        Rank::all_ranks_ascending()
            .flat_map(|rank| File::all_files_ascending().map(move |file| Location::new(file, rank)))
    }

    pub const fn failed_from_usize_message() -> &'static str {
        "Expected only one bit to be populated."
    }

    pub fn from_bitboard(bitboard: u64) -> IntoIter<Location> {
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
            println!("Value:\n{:?}", value);
            return Err(());
        }

        let rank = if value & Rank::one_bit_filter() != 0 {
            Rank::One
        } else if value & Rank::two_bit_filter() != 0 {
            Rank::Two
        } else if value & Rank::three_bit_filter() != 0 {
            Rank::Three
        } else if value & Rank::four_bit_filter() != 0 {
            Rank::Four
        } else if value & Rank::five_bit_filter() != 0 {
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
