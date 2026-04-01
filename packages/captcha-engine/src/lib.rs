pub mod difficulty;
pub mod generators;
pub mod rng;
pub mod types;

use wasm_bindgen::prelude::*;

use generators::get_generator;
use rng::rng_from_seed;
use types::*;

/// Generate a CAPTCHA instance from a seed, type, and difficulty parameters.
/// This is the primary WASM entry point.
#[wasm_bindgen]
pub fn generate_captcha(
    seed: u64,
    captcha_type: CaptchaType,
    difficulty_json: &str,
) -> Result<String, JsError> {
    let difficulty: DifficultyParams =
        serde_json::from_str(difficulty_json).map_err(|e| JsError::new(&e.to_string()))?;

    let mut rng = rng_from_seed(seed);
    let generator = get_generator(captcha_type);
    let instance = generator.generate(&mut rng, &difficulty);

    serde_json::to_string(&instance).map_err(|e| JsError::new(&e.to_string()))
}

/// Validate a player's answer against a CAPTCHA instance.
/// Used server-side in the Durable Object.
#[wasm_bindgen]
pub fn validate_answer(instance_json: &str, answer_json: &str) -> Result<bool, JsError> {
    let instance: CaptchaInstance =
        serde_json::from_str(instance_json).map_err(|e| JsError::new(&e.to_string()))?;
    let answer: PlayerAnswer =
        serde_json::from_str(answer_json).map_err(|e| JsError::new(&e.to_string()))?;

    Ok(instance.validate(&answer))
}

/// Score a player's answer, returning points earned.
#[wasm_bindgen]
pub fn score_answer(
    instance_json: &str,
    answer_json: &str,
    solve_time_ms: u32,
) -> Result<String, JsError> {
    let instance: CaptchaInstance =
        serde_json::from_str(instance_json).map_err(|e| JsError::new(&e.to_string()))?;
    let answer: PlayerAnswer =
        serde_json::from_str(answer_json).map_err(|e| JsError::new(&e.to_string()))?;

    let result = instance.score(&answer, solve_time_ms);
    serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))
}

/// Derive a round seed from match secret + round number + timestamp.
#[wasm_bindgen]
pub fn derive_round_seed(match_secret: &[u8], round_number: u32, timestamp: u64) -> u64 {
    rng::derive_seed(match_secret, round_number, timestamp)
}

/// Compute difficulty parameters for a given captcha type, level, and round.
#[wasm_bindgen]
pub fn compute_difficulty_params(captcha_type: CaptchaType, level: u32, round_number: u32) -> String {
    let params = difficulty::compute_difficulty(captcha_type, level, round_number);
    serde_json::to_string(&params).unwrap()
}
