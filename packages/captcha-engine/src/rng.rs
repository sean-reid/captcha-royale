use hmac::{Hmac, Mac};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Derive a deterministic seed from a match secret, round number, and timestamp
pub fn derive_seed(match_secret: &[u8], round_number: u32, timestamp: u64) -> u64 {
    let mut mac =
        HmacSha256::new_from_slice(match_secret).expect("HMAC can take key of any size");
    mac.update(&round_number.to_le_bytes());
    mac.update(&timestamp.to_le_bytes());
    let result = mac.finalize().into_bytes();
    // Take first 8 bytes as u64 seed
    u64::from_le_bytes(result[..8].try_into().unwrap())
}

/// Create a deterministic RNG from a seed
pub fn rng_from_seed(seed: u64) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_determinism() {
        let secret = b"test_secret_key";
        let seed1 = derive_seed(secret, 1, 1000);
        let seed2 = derive_seed(secret, 1, 1000);
        assert_eq!(seed1, seed2);
    }

    #[test]
    fn test_different_rounds_differ() {
        let secret = b"test_secret_key";
        let seed1 = derive_seed(secret, 1, 1000);
        let seed2 = derive_seed(secret, 2, 1000);
        assert_ne!(seed1, seed2);
    }

    #[test]
    fn test_rng_determinism() {
        use rand::Rng;
        let mut rng1 = rng_from_seed(42);
        let mut rng2 = rng_from_seed(42);
        for _ in 0..100 {
            assert_eq!(rng1.gen::<u64>(), rng2.gen::<u64>());
        }
    }
}
