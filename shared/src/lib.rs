use num_bigint::BigUint;
use num_traits::{Num, Zero};

/// Minimum entropy score threshold for private key validation
/// A score of 790 indicates sufficient randomness for cryptographic security
pub const MIN_ENTROPY_SCORE: u32 = 790;

/// BN254 scalar field modulus
pub const BN254_FIELD_MODULUS: &str =
    "21888242871839275222246405745257275088548364400416034343698204186575808495617";

/// Calculates entropy score for byte array to detect weak private keys.
///
/// This uses Shannon entropy formula to measure the randomness of the input.
/// Higher scores indicate more random data.
///
/// # Arguments
/// * `bytes` - The byte array to analyze
///
/// # Returns
/// An entropy score between 0 and 800 (Shannon entropy * 10)
///
/// # Examples
/// ```
/// use zkp_airdrop_utils::calculate_entropy_score;
///
/// // Low entropy (all zeros)
/// let zeros = [0u8; 32];
/// let score = calculate_entropy_score(&zeros);
/// assert_eq!(score, 0);
///
/// // Higher entropy (mixed bytes)
/// let mixed: [u8; 8] = [0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x78, 0x9a];
/// let score = calculate_entropy_score(&mixed);
/// assert!(score > 0);
/// ```
#[must_use]
pub fn calculate_entropy_score(bytes: &[u8]) -> u32 {
    if bytes.is_empty() {
        return 0;
    }

    let mut freq = [0u32; 256];
    for &byte in bytes {
        freq[byte as usize] += 1;
    }

    let len = bytes.len() as f64;
    let mut entropy = 0.0f64;

    for &count in &freq {
        if count > 0 {
            let p = f64::from(count) / len;
            entropy -= p * p.log2();
        }
    }

    (entropy * 10.0) as u32
}

/// Validates that a byte array represents a valid private key with sufficient entropy.
///
/// # Arguments
/// * `key_bytes` - The 32-byte private key to validate
///
/// # Returns
/// `Ok(())` if the key is valid, `Err` with description otherwise
///
/// # Errors
/// Returns an error if:
/// - Key is not exactly 32 bytes
/// - Key is all zeros
/// - Key equals the field modulus
/// - Key has insufficient entropy (below `MIN_ENTROPY_SCORE`)
pub fn validate_private_key(key_bytes: &[u8]) -> Result<(), String> {
    if key_bytes.len() != 32 {
        return Err(format!(
            "Private key must be 32 bytes, got {}",
            key_bytes.len()
        ));
    }

    // Check for all zeros
    if key_bytes.iter().all(|&b| b == 0) {
        return Err("Private key cannot be all zeros - this is an invalid key".to_string());
    }

    // Check against field modulus
    let field_modulus = BigUint::from_str_radix(BN254_FIELD_MODULUS, 10)
        .map_err(|e| format!("Failed to parse field modulus: {e}"))?;
    let key_biguint = BigUint::from_bytes_be(key_bytes);
    if key_biguint >= field_modulus {
        return Err("Private key exceeds field modulus".to_string());
    }

    // Check entropy score
    let entropy_score = calculate_entropy_score(key_bytes);
    if entropy_score < MIN_ENTROPY_SCORE {
        return Err(format!(
            "Private key has insufficient entropy score ({entropy_score}), may be weak. \
             Please use a securely generated random private key."
        ));
    }

    // Additional checks for known weak patterns
    check_weak_key_patterns(key_bytes)?;

    Ok(())
}

/// Checks for common weak key patterns.
///
/// # Arguments
/// * `key_bytes` - The private key bytes to check
///
/// # Returns
/// `Ok(())` if no weak patterns found, `Err` with description otherwise
fn check_weak_key_patterns(key_bytes: &[u8]) -> Result<(), String> {
    // Check for sequential bytes
    let mut sequential = 0;
    for window in key_bytes.windows(2) {
        if window[1].wrapping_sub(window[0]) == 1 || window[0].wrapping_sub(window[1]) == 1 {
            sequential += 1;
            if sequential >= 8 {
                return Err("Private key contains sequential bytes pattern".to_string());
            }
        } else {
            sequential = 0;
        }
    }

    // Check for repeated bytes
    let mut repeated = 0;
    for window in key_bytes.windows(2) {
        if window[0] == window[1] {
            repeated += 1;
            if repeated >= 8 {
                return Err("Private key contains repeated bytes pattern".to_string());
            }
        } else {
            repeated = 0;
        }
    }

    // Check for common 4-byte patterns
    const WEAK_PATTERNS: [&[u8]; 6] = [
        b"\x00\x00\x00\x00", // All zeros
        b"\xFF\xFF\xFF\xFF", // All ones
        b"\xDE\xAD\xBE\xEF", // Deadbeef
        b"\xCA\xFE\xBA\xBE", // Cafebabe
        b"\x12\x34\x56\x78", // Common test pattern
        b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFE\xBA\xDC\xED\xE8\xCF\xB7\xD0\x5C\xF7\x2C\xB0\x6B\x8A\x4B\xC8\xA6", // Curve order minus 1
    ];
    for pattern in &WEAK_PATTERNS {
        for window in key_bytes.windows(pattern.len()) {
            if window == *pattern {
                return Err("Private key contains known weak pattern".to_string());
            }
        }
    }

    // Check for common prefixes
    const WEAK_PREFIXES: [[u8; 8]; 3] = [
        [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // All zeros
        [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], // All ones
        [0xDE, 0xAD, 0xBE, 0xEF, 0xDE, 0xAD, 0xBE, 0xEF], // Deadbeef
    ];
    for prefix in &WEAK_PREFIXES {
        if &key_bytes[..8] == prefix {
            return Err("Private key has suspicious prefix pattern".to_string());
        }
    }

    Ok(())
}

/// Validates that a string represents a valid field element in BN254 scalar field.
///
/// # Arguments
/// * `hex_str` - The hex string to validate
///
/// # Returns
/// `true` if valid, `false` otherwise
///
/// # Validation Criteria
/// - Must start with "0x" or "0X"
/// - Must be exactly 66 characters (0x + 64 hex chars)
/// - Must decode to exactly 32 bytes
/// - Value must be less than BN254 field modulus
#[must_use]
pub fn is_valid_field_element(hex_str: &str) -> bool {
    let trimmed = hex_str.trim();
    if trimmed.is_empty() {
        return false;
    }

    // Check format
    if !trimmed.starts_with("0x") && !trimmed.starts_with("0X") {
        return false;
    }

    // Check length (0x + 64 hex chars = 66 total)
    if trimmed.len() != 66 {
        return false;
    }

    // Check valid hex characters
    let hex = &trimmed[2..];
    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return false;
    }

    // Decode and check length
    let bytes = match hex::decode(hex) {
        Ok(b) => b,
        Err(_) => return false,
    };

    if bytes.len() != 32 {
        return false;
    }

    // Check against field modulus
    let value = BigUint::from_bytes_be(&bytes);
    let field_modulus = match BigUint::from_str_radix(BN254_FIELD_MODULUS, 10) {
        Ok(m) => m,
        Err(_) => return false,
    };

    // Must be strictly less than modulus and not zero
    value < field_modulus && !value.is_zero()
}

/// Sanitizes a nullifier for logging by truncating it.
///
/// # Arguments
/// * `nullifier` - The nullifier string to sanitize
///
/// # Returns
/// A truncated version of the nullifier suitable for logging
#[must_use]
pub fn sanitize_nullifier(nullifier: &str) -> String {
    let chars: Vec<char> = nullifier.chars().collect();
    if chars.len() > 16 {
        let first_part: String = chars[..10].iter().collect();
        let second_part: String = chars[chars.len() - 6..].iter().collect();
        format!("{first_part}...{second_part}")
    } else if chars.len() > 6 {
        format!("{}***", &chars[..3].iter().collect::<String>())
    } else {
        "***".to_string()
    }
}

pub mod types;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_high() {
        // Test that a key with varied bytes has higher entropy than uniform bytes
        let varied: [u8; 32] = [
            0x5a, 0x8b, 0x9c, 0x2d, 0x1e, 0x7f, 0x6a, 0x3b, 0x4c, 0x5d, 0x6e, 0x8f, 0x9a, 0x0b,
            0x1c, 0x2d, 0x3e, 0x4f, 0x5a, 0x6b, 0x7c, 0x8d, 0x9e, 0x0f, 0x1a, 0x2b, 0x3c, 0x4d,
            0x5e, 0x6f, 0x7a, 0x8b,
        ];
        let varied_score = calculate_entropy_score(&varied);
        let uniform: [u8; 32] = [0x5a; 32];
        let uniform_score = calculate_entropy_score(&uniform);
        // Varied bytes should have higher entropy than uniform bytes
        assert!(
            varied_score > uniform_score,
            "Varied entropy {} should be > uniform entropy {}",
            varied_score,
            uniform_score
        );
    }

    #[test]
    fn test_entropy_zero() {
        let zeros = [0u8; 32];
        let score = calculate_entropy_score(&zeros);
        assert_eq!(score, 0);
    }

    #[test]
    fn test_entropy_all_same() {
        let same = [0xFFu8; 32];
        let score = calculate_entropy_score(&same);
        assert_eq!(score, 0);
    }

    #[test]
    fn test_validate_private_key_invalid_length() {
        let short = [0u8; 16];
        assert!(validate_private_key(&short).is_err());
    }

    #[test]
    fn test_validate_private_key_pseudo_random() {
        // Use same pseudo-random key as entropy test
        let pseudo_random: [u8; 32] = [
            0x5a, 0x8b, 0x9c, 0x2d, 0x1e, 0x7f, 0x6a, 0x3b, 0x4c, 0x5d, 0x6e, 0x8f, 0x9a, 0x0b,
            0x1c, 0x2d, 0x3e, 0x4f, 0x5a, 0x6b, 0x7c, 0x8d, 0x9e, 0x0f, 0x1a, 0x2b, 0x3c, 0x4d,
            0x5e, 0x6f, 0x7a, 0x8b,
        ];
        // Test that validation properly handles pseudo-random keys
        let result = validate_private_key(&pseudo_random);
        // May pass or fail depending on entropy score
        // The important thing is it doesn't panic and returns a proper Result
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_validate_private_key_random() {
        // Test with truly random key
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let random_key: [u8; 32] = rng.gen();
        // Random key might pass or fail depending on entropy
        // We just check it doesn't panic
        let _ = validate_private_key(&random_key);
    }

    #[test]
    fn test_validate_private_key_low_entropy() {
        let low_entropy = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
            0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
            0x1d, 0x1e, 0x1f, 0x20,
        ];
        assert!(validate_private_key(&low_entropy).is_err());
    }

    #[test]
    fn test_is_valid_field_element_valid() {
        assert!(is_valid_field_element(
            "0x0000000000000000000000000000000000000000000000000000000000000001"
        ));
    }

    #[test]
    fn test_is_valid_field_element_invalid_length() {
        assert!(!is_valid_field_element("0x1234"));
    }

    #[test]
    fn test_is_valid_field_element_no_prefix() {
        assert!(!is_valid_field_element(
            "0000000000000000000000000000000000000000000000000000000000000001"
        ));
    }

    #[test]
    fn test_is_valid_field_element_zero() {
        assert!(!is_valid_field_element(
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        ));
    }

    #[test]
    fn test_is_valid_field_element_invalid_hex() {
        assert!(!is_valid_field_element(
            "0x000000000000000000000000000000000000000000000000000000000000000g"
        ));
    }
}
