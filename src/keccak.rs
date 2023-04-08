mod keccak_p;
use keccak_p::KeccakP;

use crate::bit_string::BitString;

#[derive(Debug)]
pub struct Keccak {
    f: KeccakP,
    rate: usize,
    buffer: BitString,
}

impl Keccak {
    pub fn init(sec_param: usize) -> Keccak {
        return Keccak {
            f: keccak_p::KeccakP::init(1600, 24),
            rate: 1600 - sec_param,
            buffer: BitString::new(),
        };
    }

    pub fn update_buffer(&mut self, string_to_join: &BitString) {
	// TODO: When buffer is big enough, start absorbing instead of
	// waiting until finalise
        self.buffer.append(string_to_join);
    }

    fn generate_padding(&self, x: usize, m: usize) -> BitString {
        let j = (m * x - m - 2) % x;
        let mut P = vec![0_u64; 0];

        for _i in 0..=(((j + 2) - 1) / 64) {
            P.push(0_u64);
        }

        P[0] |= 1_u64;

        if (j + 2) % 64 == 1 {
            P[(j + 2) / 64] |= 1_u64;
        } else {
            P[((j + 2) - 1) / 64] |= 1_u64 << (((j + 2) - 1) % 64);
        }

        let output_bitstring = BitString::from_bitstring(P, j + 2);

        return output_bitstring;
    }

    fn absorb(&mut self) {
        // Compute the number of times the permutation is required
        let n = self.buffer.length / self.rate;

        // Set the capacity
        let c = self.f.b - self.rate;

        for i in 0..n {
            // Extract the ith component of the current input
            let mut P_substring = self.buffer.extract(i * self.rate + 1, (i + 1) * self.rate);

            // Pad with 0^c (capacity)
            P_substring.change_length(P_substring.length + c);

            // XOR the padded string with the current state
            for x in 0..5 {
                for y in 0..5 {
                    self.f.state[x][y] ^= P_substring.array[5 * y + x];
                }
            }

            // Execute the endomorphism
            self.f.permute();
        }
    }

    fn squeeze(&mut self, digest_size: usize) -> BitString {
        let mut Z = BitString::new();

        // Extract the state
        let mut S = BitString::zeroes(1600);
        for x in 0..5 {
            for y in 0..5 {
                S.array[5 * y + x] = self.f.state[x][y];
            }
        }

        // Cut down state to the size of rate
        S.change_length(self.rate);

        // Append padded state to Z
        Z.append(&S);

        while Z.length < digest_size {
            // Run the permutation again
            self.f.permute();

            // Extract the state
            S.change_length(self.f.b);
            for x in 0..5 {
                for y in 0..5 {
                    S.array[5 * y + x] = self.f.state[x][y];
                }
            }

            // Pad the state to the size of rate
            S.change_length(self.rate);

            // Append padded state to Z
            Z.append(&S);
        }

        Z.change_length(digest_size);

        return Z;
    }

    pub fn finalise(&mut self, digest_size: usize) -> BitString {
        // Create the padding string and append to the message
        let padding_string = self.generate_padding(self.rate, self.buffer.length);
        self.buffer.append(&padding_string);

        self.absorb();

        let Z = self.squeeze(digest_size);

        return Z;
    }
}
