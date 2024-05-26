pub trait Iso8859_1 {
    /// Decodes a slice of ISO 8859-1 encoded bytes
    /// into a UTF-8 encoded string
    fn from_8859_1(bytes: &[u8]) -> String;
    /// Encodes a UTF-8 encoded string into a vector
    /// of ISO 8859-1 encoded byte
    fn to_8859_1(&self) -> Option<Vec<u8>>;
}

impl Iso8859_1 for String {
    fn from_8859_1(bytes: &[u8]) -> String {
        // many text strings will map 1-1 with 8859-1, so start
        // with bytes.len() as a starting capacity.
        let mut result = Vec::with_capacity(bytes.len());

        for byte in bytes.to_owned() {
            if byte < 128 {
                result.push(byte);
            } else {
                // The code point value is the same, but we need to
                // move it to the 2-byte encoding of UTF8.
                result.push(((byte >> 6) & 0b00011111) | 0b11000000);
                result.push((byte & 0b00111111) | 0b10000000);
            }
        }

        unsafe { String::from_utf8_unchecked(result) }
    }

    fn to_8859_1(&self) -> Option<Vec<u8>> {
        // most characters will map 1-1 with 8859-1, so use
        // bytes.len() as a capacity.
        let mut result = Vec::with_capacity(self.len());

        let mut bytes = self.bytes();
        while let Some(byte) = bytes.next() {
            // If it's a 3-byte-encoded value, we can't decode it.
            if byte & 0b11100000 == 0b11100000 {
                return None;
            }
            if byte & 0b11000000 == 0b11000000 {
                // There's only room for 2 significant bits, so anything 
                // bigger than 0b11 is going to overflow and be wrong
                if (byte & 0b00011111) > 0b00000011 { return None; }
                let mut result_byte = (byte & 0b00000011) << 6;
                result_byte |= bytes.next()? & 0b00111111;
                result.push(result_byte);
            }
            if (byte & 0b10000000) == 0b00000000 {
                result.push(byte);
            }
        }

        return Some(result);
    }
}

#[cfg(test)]
mod tests {
    use crate::Iso8859_1;

    #[test]
    fn decodes_correctly() {
        let bytes = (0..=255).collect::<Vec<u8>>();
        let array = &bytes[..];
        let utf8 = String::from_8859_1(array);

        let mut chars = utf8.chars();
        let mut bytes_iter = bytes.iter();
        loop {
            if let Some(ch) = chars.next() {
                if let Some(byte) = bytes_iter.next() {
                    println!(
                        "8859-1: {:#04x}; UTF8: {:?};",
                        byte,
                        std::iter::once(ch)
                            .collect::<String>()
                            .as_bytes()
                            .into_iter()
                            .map(|byte| format!("{:#04x}", byte))
                            .collect::<Vec<_>>()
                    )
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        for (i, ch) in utf8.chars().enumerate() {
            match i {
                0..=0x1F => assert!(i == ch as usize, "failure at {}", i),
                0x20..=126 => assert!(ch == b" !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~"[i - 0x20] as char, "Failure at {}", i),
                127 => assert!(ch == 127 as char, "Failure at {}", i),
                128..=159 => {}
                160 => assert!(ch == '\u{00A0}', "failure at {}", i), // DELETE
                161..=172 => {
                    let num_to_skip = i - 161;
                    let mut iter = "¡¢£¤¥¦§¨©ª«¬".chars().skip(num_to_skip);
                    let ch_to_match = iter.next().unwrap();
                    assert!(ch == ch_to_match, "failure at {}. Expected {:#04x}, but got {:#04x}", i, ch_to_match as u32, ch as u32);
                }
                173 => assert!(ch == 0xAD as char, "failure at {}", i), // SHY
                174..=255 => assert!(ch == "®¯°±²³´µ¶·¸¹º»¼½¾¿ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖ×ØÙÚÛÜÝÞßàáâãäåæçèéêëìíîïðñòóôõö÷øùúûüýþÿ"
                    .chars()
                    .skip(i - 174)
                    .next()
                    .unwrap(), "failure at {}", i),
                _ => unreachable!()
            }
        }
    }

    #[test]
    fn encodes_correctly() {
        for i in 0..=255 {
            let result = String::from_8859_1(&[i]);
            let iso_8859_1 = result.to_8859_1();
            if let Some(byte) = iso_8859_1.into_iter().next() {
                println!(
                    "UTF-8: {:?}; 8859-1: {}", 
                    result.as_bytes()
                        .into_iter()
                        .map(|one_byte| format!("{:#04x}", one_byte))
                        .collect::<Vec<_>>(), 
                    byte[0]
                );
                assert!(byte[0] == i, "failed to encode {}. Got: {}", i, byte[0]);
            } else {
                println!("failed to encode {}", i);
            }
        }

        // Anything outside of the 0-255 code point range should fail, 
        // so pass \u{0101} as a test case.
        let str = String::from_utf8(vec![0xc4_u8, 0x81_u8]).unwrap();
        assert!(str.to_8859_1().is_none());
    }
}
