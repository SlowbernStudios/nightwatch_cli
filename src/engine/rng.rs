/// Frozen engine-revision-1 stream. Arithmetic is explicitly modulo 2^32.
#[derive(Clone, Debug)]
pub struct Seeded(u32);

impl Seeded {
    pub fn new(seed: u32) -> Self {
        Self(seed)
    }

    pub fn next_u32(&mut self) -> u32 {
        self.0 = self.0.wrapping_add(0x6d2b79f5);
        let mut value = (self.0 ^ (self.0 >> 15)).wrapping_mul(self.0 | 1);
        value ^= value.wrapping_add((value ^ (value >> 7)).wrapping_mul(value | 61));
        value ^ (value >> 14)
    }

    pub fn index(&mut self, length: usize) -> usize {
        assert!(length > 0);
        ((u64::from(self.next_u32()) * length as u64) >> 32) as usize
    }

    pub fn shuffle<T>(&mut self, values: &mut [T]) {
        for i in (1..values.len()).rev() {
            values.swap(i, self.index(i + 1));
        }
    }
}
