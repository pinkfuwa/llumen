use argon2;

const SALT_LEN: usize = 16;

#[derive(Default)]
pub struct Hasher {
    config: argon2::Config<'static>,
}

impl Hasher {
    pub fn verify_password(&self, hash: &str, password: &str) -> bool {
        argon2::verify_encoded(hash, password.as_bytes()).unwrap_or(false)
    }
    pub fn hash_password(&self, password: &str) -> String {
        let mut salt = [0u8; SALT_LEN];
        getrandom::fill(&mut salt).unwrap();

        let hash = argon2::hash_encoded(password.as_bytes(), &salt, &self.config).unwrap();

        return hash;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifies_the_original_password_and_rejects_incorrect_passwords() {
        let hasher = Hasher::default();
        let hash = hasher.hash_password("correct password");

        assert!(hasher.verify_password(&hash, "correct password"));
        for password in [
            "",
            "incorrect password",
            "Correct password",
            "correct password ",
        ] {
            assert!(!hasher.verify_password(&hash, password));
        }
    }

    #[test]
    fn uses_a_fresh_salt_for_each_password_hash() {
        let hasher = Hasher::default();
        let first = hasher.hash_password("same password");
        let second = hasher.hash_password("same password");

        assert_ne!(first, second);
        assert!(hasher.verify_password(&first, "same password"));
        assert!(hasher.verify_password(&second, "same password"));
    }

    #[test]
    fn verifies_unicode_passwords_without_normalizing_them() {
        let hasher = Hasher::default();
        let password = "密碼🙂é";
        let hash = hasher.hash_password(password);

        assert!(hasher.verify_password(&hash, password));
        assert!(!hasher.verify_password(&hash, "密碼🙂e\u{301}"));
        assert!(!hasher.verify_password(&hash, "密碼🙂"));
    }

    #[test]
    fn rejects_malformed_stored_hashes_without_panicking() {
        let hasher = Hasher::default();
        for hash in [
            "",
            "not an argon2 hash",
            "$argon2id$v=19$m=bad,t=3,p=1$salt$hash",
        ] {
            assert!(!hasher.verify_password(hash, "password"));
        }
    }
}
