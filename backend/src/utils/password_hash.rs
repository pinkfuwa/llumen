use argon2;

const SALT_LEN: usize = 16;

#[derive(Default)]
pub struct Hasher {
    config: argon2::Config<'static>,
}

impl Hasher {
    pub fn verify_password(&self, hash: &str, password: &str) -> bool {
        return argon2::verify_encoded(&hash, password.as_bytes()).unwrap();
    }
    pub fn hash_password(&self, password: &str) -> String {
        let mut salt = [0u8; SALT_LEN];
        getrandom::fill(&mut salt).unwrap();

        let hash = argon2::hash_encoded(password.as_bytes(), &salt, &self.config).unwrap();

        return hash;
    }
}
