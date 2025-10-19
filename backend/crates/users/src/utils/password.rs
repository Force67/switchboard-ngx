//! Password hashing and verification utilities.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use switchboard_database::UserError;

/// Hash a password using Argon2
pub fn hash_password(password: &str) -> Result<String, UserError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| UserError::DatabaseError("Password hashing failed".to_string()))?
        .to_string();

    Ok(password_hash)
}

/// Verify a password against its hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, UserError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|_| UserError::InvalidPassword)?;

    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Generate a random password
pub fn generate_random_password(length: usize) -> String {
    use rand::{distributions::Alphanumeric, Rng};

    let password: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect();

    password
}

/// Check password strength
pub fn check_password_strength(password: &str) -> PasswordStrength {
    let mut score = 0;

    // Length check
    if password.len() >= 8 {
        score += 1;
    }
    if password.len() >= 12 {
        score += 1;
    }

    // Character variety
    if password.chars().any(|c| c.is_lowercase()) {
        score += 1;
    }
    if password.chars().any(|c| c.is_uppercase()) {
        score += 1;
    }
    if password.chars().any(|c| c.is_ascii_digit()) {
        score += 1;
    }
    if password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)) {
        score += 1;
    }

    match score {
        0..=2 => PasswordStrength::Weak,
        3..=4 => PasswordStrength::Medium,
        5..=6 => PasswordStrength::Strong,
        _ => PasswordStrength::Strong,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PasswordStrength {
    Weak,
    Medium,
    Strong,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_password_strength() {
        assert_eq!(check_password_strength("123"), PasswordStrength::Weak);
        assert_eq!(check_password_strength("password"), PasswordStrength::Weak);
        assert_eq!(check_password_strength("Password123"), PasswordStrength::Medium);
        assert_eq!(check_password_strength("Password123!@#"), PasswordStrength::Strong);
    }

    #[test]
    fn test_random_password_generation() {
        let password1 = generate_random_password(12);
        let password2 = generate_random_password(12);

        assert_eq!(password1.len(), 12);
        assert_eq!(password2.len(), 12);
        assert_ne!(password1, password2);
    }
}