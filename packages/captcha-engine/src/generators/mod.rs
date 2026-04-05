pub mod text;
pub mod math;
pub mod grid;
pub mod rotation;
pub mod color;
pub mod sequence;
pub mod metamorphic;
pub mod cascade;
pub mod oddity;
pub mod spatial;
pub mod multistep;
pub mod typography;
pub mod dotcount;
pub mod clock;
pub mod fraction;
pub mod graphread;
pub mod pathtracing;
pub mod booleanlogic;
pub mod jigsaw;
pub mod shadow;
pub mod matrix;

pub mod mirror;
pub mod balance;
pub mod unscramble;
pub mod gradient;
pub mod overlap;
pub mod gears;

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
        CaptchaType::SemanticOddity => Box::new(oddity::OddityGenerator),
        CaptchaType::MetamorphicCaptcha => Box::new(metamorphic::MetamorphicGenerator),
        CaptchaType::TimePressureCascade => Box::new(cascade::CascadeGenerator),
        CaptchaType::SpatialReasoning => Box::new(spatial::SpatialGenerator),
        CaptchaType::MultiStepVerification => Box::new(multistep::MultiStepGenerator),
        CaptchaType::AdversarialTypography => Box::new(typography::TypographyGenerator),
        CaptchaType::DotCount => Box::new(dotcount::DotCountGenerator),
        CaptchaType::ClockReading => Box::new(clock::ClockReadingGenerator),
        CaptchaType::FractionComparison => Box::new(fraction::FractionComparisonGenerator),
        CaptchaType::GraphReading => Box::new(graphread::GraphReadingGenerator),
        CaptchaType::PathTracing => Box::new(pathtracing::PathTracingGenerator),
        CaptchaType::BooleanLogic => Box::new(booleanlogic::BooleanLogicGenerator),
        CaptchaType::PartialOcclusion => Box::new(jigsaw::JigsawFitGenerator),
        CaptchaType::AdversarialImage => Box::new(shadow::ShadowMatchingGenerator),
        CaptchaType::CombinedModality => Box::new(matrix::MatrixPatternGenerator),
        CaptchaType::ProceduralNovelType => Box::new(text::TextGenerator), // placeholder
        CaptchaType::MirrorMatch => Box::new(mirror::MirrorGenerator),
        CaptchaType::BalanceScale => Box::new(balance::BalanceGenerator),
        CaptchaType::WordUnscramble => Box::new(unscramble::UnscrambleGenerator),
        CaptchaType::GradientOrder => Box::new(gradient::GradientGenerator),
        CaptchaType::OverlapCounting => Box::new(overlap::OverlapGenerator),
        CaptchaType::RotationPrediction => Box::new(gears::GearsGenerator),
        // TODO: implement remaining generators
        _ => Box::new(text::TextGenerator), // fallback for now
    }
}
