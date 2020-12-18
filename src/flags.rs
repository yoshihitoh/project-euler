use std::ops::{BitAnd, BitOr};

fn mask_for(pos: usize) -> u32 {
    assert!(pos < 32);
    0x01 << pos
}

#[derive(Default, Copy, Clone)]
pub struct Flags32(u32);

impl Flags32 {
    pub fn clear(&mut self) {
        self.0 = 0;
    }

    pub fn num_set(&self) -> usize {
        self.0.count_ones() as usize
    }

    pub fn get(&self, pos: usize) -> bool {
        self.0 & mask_for(pos) != 0
    }

    pub fn update(&mut self, pos: usize, value: bool) {
        if value {
            self.set(pos);
        } else {
            self.unset(pos);
        }
    }

    pub fn set(&mut self, pos: usize) {
        self.0 |= mask_for(pos);
    }

    pub fn unset(&mut self, pos: usize) {
        self.0 &= !mask_for(pos);
    }

    pub fn iter(&self) -> Flags32Iter {
        Flags32Iter {
            flags: *self,
            pos: 0,
        }
    }
}

impl BitAnd for Flags32 {
    type Output = Flags32;

    fn bitand(self, rhs: Self) -> Self::Output {
        Flags32(self.0 & rhs.0)
    }
}

impl BitOr for Flags32 {
    type Output = Flags32;

    fn bitor(self, rhs: Self) -> Self::Output {
        Flags32(self.0 | rhs.0)
    }
}

pub struct Flags32Iter {
    flags: Flags32,
    pos: usize,
}

impl Iterator for Flags32Iter {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < 32 {
            let b = self.flags.get(self.pos);
            self.pos += 1;
            Some(b)
        } else {
            None
        }
    }
}
