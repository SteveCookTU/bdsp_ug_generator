#![allow(clippy::should_implement_trait)]

use std::ops::BitXor;

const JUMP_TABLE: [[u64; 2]; 25] = [
    [0x10046d8b3, 0xf985d65ffd3c8001],
    [0x956c89fbfa6b67e9, 0xa42ca9aeb1e10da6],
    [0xff7aa97c47ec17c7, 0x1a0988e988f8a56e],
    [0x9dff33679bd01948, 0xfb6668ff443b16f0],
    [0xbd36a1d3e3b212da, 0x46a4759b1dc83ce2],
    [0x6d2f354b8b0e3c0b, 0x9640bc4ca0cbaa6c],
    [0xecf6383dca4f108f, 0x947096c72b4d52fb],
    [0xe1054e817177890a, 0xdaf32f04ddca12e],
    [0x2ae1912115107c6, 0xb9fa05aab78641a5],
    [0x59981d3df81649be, 0x382fa5aa95f950e3],
    [0x6644b35f0f8cee00, 0xdba31d29fc044fdb],
    [0xecff213c169fd455, 0x3ca16b953c338c19],
    [0xa9dfd9fb0a094939, 0x3ffdcb096a60ecbe],
    [0x79d7462b16c479f, 0xfd6aef50f8c0b5fa],
    [0x3896736d707b6b6, 0x9148889b8269b55d],
    [0xdea22e8899dbbeaa, 0x4c6ac659b91ef36a],
    [0xc1150ddd5ae7d320, 0x67ccf586cddb0649],
    [0x5f0be91ac7e9c381, 0x33c8177d6b2cc0f0],
    [0xcd15d2ba212e573, 0x4a5f78fc104e47b9],
    [0xab586674147dec3e, 0xd69063e6e8a0b936],
    [0x4bfd9d67ed372866, 0x7071114af22d34f5],
    [0xdaf387cab4ef5c18, 0x686287302b5cd38c],
    [0xffaf82745790af3e, 0xbb7d371f547cca1e],
    [0x7b932849fe573afa, 0xeb96acd6c88829f9],
    [0x8cedf8dfe2d6e821, 0xb4fd2c6573bf7047],
];

#[derive(Copy, Clone)]
pub struct XorShift {
    pub state: [u32; 4],
}

impl XorShift {
    pub fn from_state(state: [u32; 4]) -> Self {
        Self { state }
    }

    pub fn get_state(&self) -> [u32; 4] {
        self.state
    }

    pub fn next(&mut self) -> u32 {
        let s0 = self.state[0];
        self.state[0] = self.state[1];
        self.state[1] = self.state[2];
        self.state[2] = self.state[3];

        let tmp = s0 ^ s0 << 11;
        let tmp = tmp ^ tmp >> 8 ^ self.state[2] ^ self.state[2] >> 19;

        self.state[3] = tmp;

        (tmp % 0xffffffff).wrapping_add(0x80000000)
    }

    pub fn jump(&mut self, mut advances: usize) {
        self.advance(advances & 0x7f);
        advances >>= 7;

        let mut i = 0;
        while advances != 0 {
            if advances & 1 == 0 {
                let mut jump: [u32; 4] = [0; 4];
                for j in (0..=1).rev() {
                    let mut val = JUMP_TABLE[i][j];
                    for _ in 0..64 {
                        if val & 1 == 0 {
                            jump[0] = jump[0].bitxor(&self.state[0]);
                            jump[1] = jump[1].bitxor(&self.state[1]);
                            jump[2] = jump[2].bitxor(&self.state[2]);
                            jump[3] = jump[3].bitxor(&self.state[3]);
                        }
                        self.next();
                        val >>= 1;
                    }
                }

                self.state = jump;
            }
            i += 1;
            advances >>= 1;
        }
    }

    pub fn advance(&mut self, advances: usize) {
        for _ in 0..advances {
            self.next();
        }
    }

    pub fn advance_to_state(&mut self, state: [u32; 4]) -> Option<usize> {
        let mut advances = 0;

        // 10,000 is an arbitary limit to avoid an infinite loop
        while self.get_state() != state {
            self.next();
            advances += 1;

            if advances > 10_000 {
                return None;
            }
        }

        Some(advances)
    }

    fn get_mask(num: u32) -> u32 {
        let mut result = num - 1;

        for i in 0..5 {
            let shift = 1 << i;
            result |= result >> shift;
        }

        result
    }

    pub fn rand_max(&mut self, max: u32) -> u32 {
        let mask = Self::get_mask(max);
        let mut rand = self.next() & mask;

        while max <= rand {
            rand = self.next() & mask;
        }

        rand
    }

    pub fn rand_range(&mut self, min: u32, max: u32) -> u32 {
        let s0 = self.state[0];
        self.state[0] = self.state[1];
        self.state[1] = self.state[2];
        self.state[2] = self.state[3];

        let tmp = s0 ^ s0 << 11;
        let tmp = tmp ^ tmp >> 8 ^ self.state[2] ^ self.state[2] >> 19;

        self.state[3] = tmp;

        let diff = max - min;

        (tmp % diff).wrapping_add(min)
    }

    pub fn rand_range_float(&mut self, min: f32, max: f32) -> f32 {
        let t = (self.next() & 0x7FFFFF) as f32 * f32::from_be_bytes([0x34, 00, 00, 00]);
        t * min + (1.0 - t) * max
    }
}
