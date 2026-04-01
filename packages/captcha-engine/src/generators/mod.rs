pub mod text;
pub mod math;
pub mod grid;
pub mod rotation;
pub mod color;
pub mod sequence;

use rand_chacha::ChaCha8Rng;

use crate::types::{CaptchaInstance, CaptchaType, DifficultyParams, PlayerAnswer};

/// Core trait all generators implement
pub trait CaptchaGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance;
    fn validate(&self, instance: &CaptchaInstance, answer: &PlayerAnswer) -> bool;
}

/// Get the appropriate generator for a CAPTCHA type
pub fn get_generator(captcha_type: CaptchaType) -> Box<dyn CaptchaGenerator> {
    match captcha_type {
        CaptchaType::DistortedText => Box::new(text::TextGenerator),
        CaptchaType::SimpleMath => Box::new(math::MathGenerator),
        CaptchaType::ImageGrid => Box::new(grid::GridGenerator),
        CaptchaType::RotatedObject => Box::new(rotation::RotationGenerator),
        CaptchaType::ColorPerception => Box::new(color::ColorGenerator),
        CaptchaType::SequenceCompletion => Box::new(sequence::SequenceGenerator),
        // TODO: implement remaining generators
        _ => Box::new(text::TextGenerator), // fallback for now
    }
}
