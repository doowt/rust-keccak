#[derive(Debug)]
pub struct KeccakP {
    pub b: usize,
    n_r: usize,
    w: usize,
    l: usize,
    offset: [[usize; 5]; 5],
    pub state: [[u64; 5]; 5],
}

impl KeccakP {
    pub fn init(b: usize, n_r: usize) -> KeccakP {
        let (l, w) = match b {
            25 => (0, 1),
            50 => (1, 2),
            100 => (2, 4),
            200 => (3, 8),
            400 => (4, 16),
            800 => (5, 32),
            1600 => (6, 64),
            _ => panic!("Invalid choice of b (width)"),
        };

        // The offsets are from FIPS 202. Note that the offsets are
        // transposed in rho_offset because of how an array is stored
        // in rust
        // [
        //     [0, 1, 190, 28, 91],
        //     [36, 300, 6, 55, 276],
        //     [3, 10, 171, 153, 231],
        //     [105, 45, 15, 21, 136],
        //     [210, 66, 253, 120, 78],
        // ];

        let mut rho_offset: [[usize; 5]; 5] = [
            [0, 36, 3, 41, 18],
            [1, 44, 10, 45, 2],
            [62, 6, 43, 15, 61],
            [28, 55, 25, 21, 56],
            [27, 20, 39, 8, 14],
        ];

        for i in 0..5 {
            for j in 0..5 {
                rho_offset[i][j] %= w;
            }
        }

        KeccakP {
            b,
            n_r,
            w,
            l,
            offset: rho_offset,
            state: [[0_u64; 5]; 5],
        }
    }

    pub fn set_state(&mut self, byte_array: [[u64; 5]; 5]) {
        self.state = byte_array;
    }

    pub fn to_string(&self) -> String {
        let mut output_string: String = "".to_string();

        for x in 0..5 {
            for y in 0..5 {
                // Has to be reversed for output
                output_string.push_str(&format!("  ({:?},{:?}) = {:X}", x, y, &self.state[y][x]));
            }
        }
        return output_string;
    }

    pub fn permute(&mut self) {
        for i_r in (12 + 2 * self.l - self.n_r)..=(12 + 2 * self.l - 1) {
            self.theta();
            self.rho();
            self.pi();
            self.chi();
            self.iota(i_r);
        }
    }

    fn rotate_left(&self, x: u64, offset: usize) -> u64 {
        if offset == 0 {
            return x;
        }
        return (x << offset) | (x >> (self.w - offset));
    }

    fn theta(&mut self) {
        let mut C = [0_u64; 5];
        let mut D = [0_u64; 5];

        // Step 1
        for x in 0..5 {
            for y in 0..5 {
                C[x] ^= self.state[x][y];
            }
        }

        // Step 2
        for x in 0..5 {
            D[x] = C[(5 + x - 1) % 5]
                ^ ((C[(5 + x + 1) % 5] << 1) | (C[(5 + x + 1) % 5] >> (self.w - 1)) & 0x1);
        }

        // Step 3
        for x in 0..5 {
            for y in 0..5 {
                self.state[x][y] ^= D[x];
            }
        }
    }

    fn rho(&mut self) {
        let mut a_prime = [[0_u64; 5]; 5];

        // Steps 1 - 3 (use shift for speed)
        for i in 0..5 {
            for j in 0..5 {
                a_prime[i][j] = self.rotate_left(self.state[i][j], self.offset[i][j]);
            }
        }

        // Step 4
        self.set_state(a_prime);
    }

    fn pi(&mut self) {
        let mut a_prime = [[0_u64; 5]; 5];

        // Step 1
        for x in 0..5 {
            for y in 0..5 {
                a_prime[x][y] = self.state[(x + 3 * y) % 5][x];
            }
        }

        // Step 2
        self.set_state(a_prime);
    }

    fn chi(&mut self) {
        let mut a_prime = [[0_u64; 5]; 5];

        // Step 1
        for x in 0..5 {
            for y in 0..5 {
                a_prime[x][y] = self.state[x][y]
                    ^ ((self.state[(x + 1) % 5][y] ^ 0xFFFFFFFFFFFFFFFF_u64)
                        & (self.state[(x + 2) % 5][y]));
            }
        }

        // Step 2
        self.set_state(a_prime);
    }

    fn rc(t: usize) -> u64 {
        let remainder = t % 255;

        if remainder == 0 {
            return 1_u64;
        }

        // R is stored little-endian so that 0||R makes sense
        let mut R = 0x01_u16;

        // Step 3
        for _i in 0..remainder {
            R <<= 1;
            if (R >> 8 & 0x1) == 0x1 {
                R ^= 0x71;
            }
            R &= 0xFF_u16;
        }

        // Step 4
        (R & 0x1) as u64
    }

    fn iota(&mut self, i_r: usize) {
        // Step 1

        // Step 2
        let mut round_constant = 0_u64;

        for j in 0..=self.l {
            round_constant |= KeccakP::rc(j + 7 * i_r) << (i64::pow(2, j as u32) - 1);
        }

        self.state[0][0] ^= round_constant;
    }
}
