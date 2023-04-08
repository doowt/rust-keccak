#[derive(Debug, Default)]
pub struct BitString {
    pub array: Vec<u64>,
    pub length: usize,
}

impl BitString {
    pub fn new() -> BitString {
        return BitString {
            array: vec![0_u64; 0],
            length: 0,
        };
    }

    pub fn zeroes(length: usize) -> BitString {
        return BitString {
            array: vec![0_u64; (length - 1) / 64 + 1],
            length,
        };
    }

    pub fn to_string(&self) -> String {
        let mut output_string: String = "".to_string();

        if self.length > 0 && !self.array.is_empty() {
            for i in 0..self.array.len() {
                for j in 0..8 {
                    let current_byte = (self.array[i] >> (8 * j)) & 0xFF;
                    if (i * 64 + j * 8) < self.length {
                        output_string.push_str(&format!("{:02X}", current_byte));
                    }
                }
                if i < self.array.len() - 1 {
                    output_string.push(' ');
                }
            }
        }
        return output_string;
    }

    pub fn to_swapped_string(&self) -> String {
        let mut output_string: String = "".to_string();

        if self.length > 0 && !self.array.is_empty() {
            for i in 0..self.array.len() {
                for j in 0..8 {
                    let current_byte = if j == 7 {
                        self.array[self.array.len() - 1 - i] as u8
                    } else {
                        (self.array[self.array.len() - 1 - i] >> ((7 - j) * 8)) as u8
                    };
                    output_string.push_str(&format!(" {:02X}", current_byte.reverse_bits()));
                }
            }
        }
        return output_string;
    }

    pub fn from_bitstring(array: Vec<u64>, length: usize) -> BitString {
        let mut output_bitstring = BitString { array, length };
        output_bitstring.sanitise();

        return output_bitstring;
    }

    pub fn from_binary_string(input_string: &String) -> BitString {
        let mut output_bitstring: Vec<u64> = vec![0_u64; 0];
        let input_string_length = input_string.len();

        // N.B. This stores data little-endian
        for i in 0..input_string_length {
            output_bitstring[i / 64] |= ((input_string.as_bytes()[i] == 0x31) as u64) << (i % 64);

            if input_string.as_bytes()[i] != 0x30 && input_string.as_bytes()[i] != 0x31 {
                panic!("Bit string must contain only '0' or '1' characters");
            }
        }

        BitString {
            array: output_bitstring,
            length: input_string_length,
        }
    }

    pub fn from_string(input_string: &str) -> BitString {
	let output_bitstring = BitString::from_byte_array(input_string.as_bytes().to_vec());
	return output_bitstring;
    }

    pub fn from_byte_array(input_byte_array: Vec<u8>) -> BitString {
        if input_byte_array.len() == 0 {
            return BitString::new();
        }

        let new_length: usize = input_byte_array.len() * 8;

        let mut output_bitstring = BitString {
            array: vec![0_u64; (new_length - 1) / 64 + 1],
            length: new_length,
        };

        for i in 0..input_byte_array.len() {
            if (i % 8) == 0 {
                output_bitstring.array[i / 8] |= input_byte_array[i] as u64;
            } else {
                output_bitstring.array[i / 8] |= (input_byte_array[i] as u64) << (8 * (i % 8));
            }
        }

        output_bitstring.sanitise();

        return output_bitstring;
    }

    pub fn change_length(&mut self, c: usize) {
        self.length = c;
        self.sanitise();
    }

    pub fn left_shift(&mut self, shift: usize) {
        if shift > 0 {
            self.length += shift;

            self.sanitise();

            // Set the top word to the appropriate word below, shifted
            for i in ((1 + (shift - 1) / 64)..=((self.length - 1) / 64)).rev() {
                self.array[i] = (self.array[i - (shift - 1) / 64] << (shift % 64))
                    | self.array[i - 1 - (shift - 1) / 64] >> (64 - (shift % 64));
            }

            // Transfer the last word shifted
            self.array[(shift - 1) / 64] = self.array[0] << (shift % 64);

            // Zeroise below the shift
            for i in 0..((shift - 1) / 64) {
                self.array[i] = 0;
            }

            self.sanitise();
        }
    }

    pub fn right_shift(&mut self, shift: usize) {
        self.sanitise();

        if shift > self.length {
            self.array = vec![0_u64; 0];
            self.length = 0;
        } else {
            // Add zeros beyond MSBs make shift easier
            for _i in 0..=((shift - 1) / 64 + 1) {
                self.array.push(0_u64);
            }

            // Shift
            if shift % 64 == 0 {
                for i in 0..((self.length - 1) / 64 - (shift - 1) / 64) {
                    self.array[i] = self.array[i + ((shift - 1) / 64) + 1];
                }
            } else {
                for i in 0..=((shift - 1) / 64 + 1) {
                    self.array[i] = self.array[i + (shift - 1) / 64] >> (shift % 64)
                        | self.array[i + 1 + (shift - 1) / 64] << (64 - (shift % 64));
                }

                for i in ((shift - 1) / 64 + 1 + 1)..self.array.len() {
                    self.array[i] = 0_u64;
                }
            }

            self.length -= shift;

            self.sanitise();
        }
    }

    fn sanitise(&mut self) {
        if self.length == 0 {
            self.array = vec![0_u64; 0];
        } else {
            // Add words if value is too small
            while self.array.len() < ((self.length - 1) / 64 + 1) {
                self.array.push(0_u64);
            }

            // Remove words if value is too big
            while self.array.len() > ((self.length - 1) / 64 + 1) {
                self.array.pop();
            }

            // Zeroise bits beyond length
            let current_array_length = self.array.len();
            if self.length % 64 != 0 {
                let mask = 0xFFFFFFFFFFFFFFFF >> (64 - (self.length % 64));
                self.array[current_array_length - 1] &= mask;
            }
        }
    }

    pub fn append(&mut self, string_to_join: &BitString) {
        self.sanitise();

        let new_length = self.length + string_to_join.length;
        let old_array_length = self.array.len();

        if old_array_length == 0 {
            *self = string_to_join.copy();
        } else {
            if self.length % 64 == 0 {
                for i in 0..string_to_join.array.len() {
                    self.array.push(0_u64);
                    self.array[old_array_length + i] |= string_to_join.array[i];
                }
            } else {
                for i in 0..string_to_join.array.len() {
                    self.array.push(0_u64);
                    self.array[(old_array_length - 1) + i] |=
                        string_to_join.array[i] << (self.length % 64);
                }
            }
        }
        self.length = new_length;
        self.sanitise();
    }

    pub fn extract(&self, start: usize, end: usize) -> BitString {
        // TODO: Chop routine to make equally-sized chunks from a bitstring

        let mut extracted_bitstring = self.copy();

        if end > extracted_bitstring.length {
            panic!("Nothing to cut!");
        }

        if start > end {
            panic!("Start cannot be after end");
        }

        if start > extracted_bitstring.length {
            panic!("Nothing to cut");
        }

        if start > 1 {
            extracted_bitstring.right_shift(start - 1);
        }

        extracted_bitstring.length = end - start + 1;
        extracted_bitstring.sanitise();
        return extracted_bitstring;
    }

    pub fn copy(&self) -> BitString {
        let mut new_bitstring: BitString = BitString::new();

        if self.length == 0 || self.array.is_empty() {
            return new_bitstring;
        }

        // Copy value
        for i in 0..self.array.len() {
            new_bitstring.array.push(0_u64);
            new_bitstring.array[i] = self.array[i];
        }

        // Copy length
        new_bitstring.length = self.length;

        return new_bitstring;
    }
}

#[cfg(test)]
mod bitstring_tests {

    use super::*;

    #[test]
    fn test_left_shift() {
        let mut a = BitString::from_bitstring(vec![0x9_u64; 1], 4);
        a.left_shift(65);
        assert_eq!(a.array, [0x0_u64, 0x12_u64]);
        let mut b = BitString::from_bitstring(vec![0x9_u64; 1], 4);
        b.left_shift(130);
        assert_eq!(b.array, [0x0_u64, 0x0_u64, 0x24_u64]);
        let mut c = BitString::from_bitstring(vec![0x123_u64], 12);
        c.left_shift(79);
        assert_eq!(c.array, [0x0_u64, 0x918000_u64]);
        let mut d = BitString::from_bitstring(
            vec![
                0x0123456789ABCDEF_u64,
                0xFEDCBA9876543210_u64,
                0x0123456789ABCDEF_u64,
                0xFEDCBA9876543210_u64,
            ],
            256,
        );

        d.left_shift(129);
        assert_eq!(
            d.array,
            [
                0x0000000000000000_u64,
                0x0000000000000000_u64,
                0x02468ACF13579BDE_u64,
                0xFDB97530ECA86420_u64,
                0x02468ACF13579BDF_u64,
                0xFDB97530ECA86420_u64,
                0x1_u64
            ]
        );
    }
    #[test]
    fn test_right_shift() {
        let mut a = BitString::from_bitstring(vec![0x9_u64; 1], 4);
        a.right_shift(2);
        assert_eq!(a.array, [0x2_u64]);
        let mut b = BitString::from_bitstring(vec![0x9_u64, 0xF_u64], 85);
        b.right_shift(4);
        assert_eq!(b.array, [0xF000000000000000_u64, 0_u64]);
        let mut c = BitString::from_bitstring(vec![0x456_u64, 0x123_u64], 76);
        c.right_shift(4);
        assert_eq!(c.array, [0x3000000000000045_u64, 0x12_u64]);
        let mut d = BitString::from_bitstring(
            vec![
                0x0123456789ABCDEF_u64,
                0xFEDCBA9876543210_u64,
                0x0123456789ABCDEF_u64,
                0xFEDCBA9876543210_u64,
            ],
            256,
        );
        d.right_shift(129);
        assert_eq!(d.array, [0x0091A2B3C4D5E6F7_u64, 0x7F6E5D4C3B2A1908_u64]);
    }
}
