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
        debug_assert!(width <= 64, "Width must be <= 64");
        LFSR { state, taps, width }
    }

    pub fn next(&mut self) -> u8 {
        let mut feedback = 0u8;
        for &tap in &self.taps {
            feedback ^= ((self.state >> (self.width - tap)) & 1) as u8;
        }
        let output = (self.state >> (self.width - 1)) & 1;
        self.state = ((self.state << 1) | feedback as u64) & ((1 << self.width) - 1);
        output as u8
    }
}

impl Default for LFSR {
    fn default() -> Self {
        LFSR::new(0, vec![1, 4, 3], 64)
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
