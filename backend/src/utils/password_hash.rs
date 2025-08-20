use std::sync::Mutex;

use argon2;

const SALT_LEN: usize = 16;

struct LFSR {
    state: u64,
    taps: Vec<u8>,
    width: u8,
}

impl LFSR {
    pub fn new(state: u64, taps: Vec<u8>, width: u8) -> Self {
        debug_assert!(width >= 1 && width <= 64, "Width must be between 1 and 64");
        LFSR { state, taps, width }
    }

    pub fn next(&mut self) -> u8 {
        let mut feedback = 0u8;
        for &tap in &self.taps {
            let shift = (self.width as u32)
                .checked_sub(tap as u32)
                .expect("tap must be <= width");
            feedback ^= ((self.state >> shift) & 1) as u8;
        }
        let output = ((self.state >> (self.width as u32 - 1)) & 1) as u8;
        let mask = match self.width {
            64 => u64::MAX,
            width => (1u64 << (width as u32)) - 1,
        };
        self.state = (((self.state << 1) | feedback as u64) & mask);
        output
    }
}

fn rand() -> i32 {
    unsafe { libc::rand() }
}

impl Default for LFSR {
    fn default() -> Self {
        let width: u8 = 64;
        let taps = (0..4)
            .map(|_| (rand() % (width as i32) + 1) as u8)
            .collect();

        let seed = ((rand() as u64) << 32) | (rand() as u64);
        let state = if seed == 0 { 1 } else { seed };
        LFSR::new(state, taps, width)
    }
}

#[derive(Default)]
pub struct Hasher {
    lfsr: Mutex<LFSR>,
    config: argon2::Config<'static>,
}

impl Hasher {
    pub fn verify_password(&self, hash: &str, password: &str) -> bool {
        return argon2::verify_encoded(&hash, password.as_bytes()).unwrap();
    }
    pub fn hash_password(&self, password: &str) -> String {
        let salt = {
            let mut lfsr = self.lfsr.lock().unwrap();
            (0..SALT_LEN).map(|_| lfsr.next()).collect::<Vec<u8>>()
        };

        let hash = argon2::hash_encoded(password.as_bytes(), &salt, &self.config).unwrap();

        return hash;
    }
}
