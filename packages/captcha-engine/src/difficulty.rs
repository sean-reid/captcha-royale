use crate::types::{CaptchaType, DifficultyParams};

/// Compute difficulty parameters from player level and round number
pub fn compute_difficulty(
    captcha_type: CaptchaType,
    level: u32,
    round_number: u32,
) -> DifficultyParams {
    // Complexity scales from 0.0 to 1.0 based on level and round
    let level_factor = (level as f32 / 100.0).min(1.0);
    let round_factor = (round_number as f32 / 20.0).min(1.0);

    // Base complexity: meaningful floor so even round 1 has some challenge
    let raw_complexity = level_factor * 0.5 + round_factor * 0.5;
    // Floor at 0.25 so there's always baseline noise/distortion
    let complexity = (0.25 + raw_complexity * 0.75).min(1.0);
    // Noise is close to complexity but with its own floor
    let noise = (0.2 + raw_complexity * 0.8).min(1.0);

    let time_limit_ms = compute_time_limit(captcha_type, complexity);

    DifficultyParams {
        level,
        round_number,
        time_limit_ms,
        complexity,
        noise,
    }
}

/// Compute time limit based on captcha type and complexity
fn compute_time_limit(captcha_type: CaptchaType, complexity: f32) -> u32 {
    let (min_time, max_time) = match captcha_type {
        // Tier 1
        CaptchaType::DistortedText => (5_000, 10_000),
        CaptchaType::SimpleMath => (4_000, 12_000),
        CaptchaType::ImageGrid => (4_000, 8_000),
        CaptchaType::SliderAlignment => (3_000, 6_000),
        // Tier 2
        CaptchaType::RotatedObject => (5_000, 11_000),
        CaptchaType::PartialOcclusion => (4_000, 12_000),
        CaptchaType::SemanticOddity => (5_000, 15_000),
        CaptchaType::ToneRhythm => (5_000, 12_000),
        CaptchaType::ColorPerception => (3_000, 9_000),
        // Tier 3
        CaptchaType::AdversarialImage => (7_000, 15_000),
        CaptchaType::SequenceCompletion => (8_000, 22_000),
        CaptchaType::MultiStepVerification => (10_000, 20_000),
        CaptchaType::SpatialReasoning => (7_000, 18_000),
        CaptchaType::ContextualReasoning => (8_000, 18_000),
        // Tier 4
        CaptchaType::MetamorphicCaptcha => (10_000, 20_000),
        CaptchaType::CombinedModality => (12_000, 25_000),
        CaptchaType::AdversarialTypography => (7_000, 18_000),
        CaptchaType::ProceduralNovelType => (15_000, 35_000),
        CaptchaType::TimePressureCascade => (2_500, 8_000),
    };

    let time = min_time as f32 + (max_time - min_time) as f32 * complexity;
    time as u32
}

/// Compute expected solve time (roughly time_limit / 1.5)
pub fn expected_solve_time(time_limit_ms: u32) -> u32 {
    (time_limit_ms as f32 / 1.5) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_scales_with_level() {
        let low = compute_difficulty(CaptchaType::DistortedText, 1, 1);
        let high = compute_difficulty(CaptchaType::DistortedText, 50, 1);
        assert!(high.complexity > low.complexity);
        assert!(high.time_limit_ms > low.time_limit_ms);
    }

    #[test]
    fn test_difficulty_scales_with_round() {
        let early = compute_difficulty(CaptchaType::SimpleMath, 10, 1);
        let late = compute_difficulty(CaptchaType::SimpleMath, 10, 15);
        assert!(late.complexity > early.complexity);
    }

    #[test]
    fn test_complexity_capped_at_one() {
        let d = compute_difficulty(CaptchaType::ImageGrid, 200, 100);
        assert!(d.complexity <= 1.0);
    }
}
