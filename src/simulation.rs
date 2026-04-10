use std::time::{SystemTime, UNIX_EPOCH};

pub struct Rng(u32);
impl Rng {
    #[inline(always)]
    pub fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let seed = (nanos as u32) ^ ((nanos >> 32) as u32);
        Self(seed | 1)
    }
    
    #[inline(always)]
    pub fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
}

#[inline(always)]
pub fn simulate_step(buf: &mut [u8], w: usize, h: usize, rng: &mut Rng) {
    for _ in 0..(w / 9).max(1) {
        let idx = (rng.next() as usize % w) + w * (h - 1);
        if idx < buf.len() {
            buf[idx] = 65;
        }
    }

    let size = w * h;
    let limit = if size + w + 1 <= buf.len() { size } else { buf.len().saturating_sub(w + 1) };
    for i in 0..limit {
        let sum = buf[i] as u32
            + buf[i + 1] as u32
            + buf[i + w] as u32
            + buf[i + w + 1] as u32;
        buf[i] = (sum >> 2) as u8;
    }
}