use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// All available CAPTCHA types
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptchaType {
    // Tier 1 — Foundations
    DistortedText,
    SimpleMath,
    ImageGrid,
    SliderAlignment,
    // Tier 2 — Perceptual
    RotatedObject,
    PartialOcclusion,
    SemanticOddity,
    ToneRhythm,
    ColorPerception,
    // Tier 3 — Cognitive
    AdversarialImage,
    SequenceCompletion,
    MultiStepVerification,
    SpatialReasoning,
    ContextualReasoning,
    // Tier 4 — Nightmare
    MetamorphicCaptcha,
    CombinedModality,
    AdversarialTypography,
    ProceduralNovelType,
    TimePressureCascade,
}

impl CaptchaType {
    pub fn tier(&self) -> u32 {
        match self {
            CaptchaType::DistortedText
            | CaptchaType::SimpleMath
            | CaptchaType::ImageGrid
            | CaptchaType::SliderAlignment => 1,

            CaptchaType::RotatedObject
            | CaptchaType::PartialOcclusion
            | CaptchaType::SemanticOddity
            | CaptchaType::ToneRhythm
            | CaptchaType::ColorPerception => 2,

            CaptchaType::AdversarialImage
            | CaptchaType::SequenceCompletion
            | CaptchaType::MultiStepVerification
            | CaptchaType::SpatialReasoning
            | CaptchaType::ContextualReasoning => 3,

            CaptchaType::MetamorphicCaptcha
            | CaptchaType::CombinedModality
            | CaptchaType::AdversarialTypography
            | CaptchaType::ProceduralNovelType
            | CaptchaType::TimePressureCascade => 4,
        }
    }

    pub fn base_points(&self) -> u32 {
        self.tier() * 10
    }
}

/// Difficulty parameters passed to generators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyParams {
    pub level: u32,
    pub round_number: u32,
    pub time_limit_ms: u32,
    pub complexity: f32,
    pub noise: f32,
}

/// The render payload sent to the client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RenderPayload {
    /// SVG markup string for text/math/visual CAPTCHAs
    Svg(String),
    /// Grid of cell data for image grid CAPTCHAs
    Grid {
        cols: u32,
        rows: u32,
        cells: Vec<GridCell>,
        prompt: String,
    },
    /// Slider puzzle data
    Slider {
        background_svg: String,
        piece_svg: String,
        correct_x: f32,
        correct_y: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridCell {
    pub index: u32,
    pub svg: String,
    pub label: String,
}

/// The correct answer (kept server-side only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Solution {
    /// Exact text match (for distorted text)
    Text(String),
    /// Numeric answer (for math)
    Number(f64),
    /// Set of correct cell indices (for grids)
    SelectedIndices(Vec<u32>),
    /// Position within tolerance (for sliders)
    Position { x: f32, y: f32, tolerance: f32 },
}

/// Player's submitted answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerAnswer {
    Text(String),
    Number(f64),
    SelectedIndices(Vec<u32>),
    Position { x: f32, y: f32 },
}

/// A complete CAPTCHA instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaInstance {
    pub render_data: RenderPayload,
    pub solution: Solution,
    pub expected_solve_time_ms: u32,
    pub point_value: u32,
    pub captcha_type: CaptchaType,
    pub time_limit_ms: u32,
}

/// Scoring result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreResult {
    pub correct: bool,
    pub base_points: u32,
    pub speed_bonus: u32,
    pub total_points: u32,
}

impl CaptchaInstance {
    pub fn score(&self, answer: &PlayerAnswer, solve_time_ms: u32) -> ScoreResult {
        let correct = self.validate(answer);
        if !correct {
            return ScoreResult {
                correct: false,
                base_points: 0,
                speed_bonus: 0,
                total_points: 0,
            };
        }

        let base = self.point_value;
        let speed_bonus = if solve_time_ms < self.time_limit_ms {
            let ratio = (self.time_limit_ms - solve_time_ms) as f32 / self.time_limit_ms as f32;
            (ratio * base as f32) as u32
        } else {
            0
        };

        ScoreResult {
            correct: true,
            base_points: base,
            speed_bonus,
            total_points: base + speed_bonus,
        }
    }

    pub fn validate(&self, answer: &PlayerAnswer) -> bool {
        match (&self.solution, answer) {
            (Solution::Text(expected), PlayerAnswer::Text(given)) => {
                expected.to_lowercase() == given.to_lowercase()
            }
            (Solution::Number(expected), PlayerAnswer::Number(given)) => {
                (expected - given).abs() < 0.001
            }
            (Solution::SelectedIndices(expected), PlayerAnswer::SelectedIndices(given)) => {
                let mut exp = expected.clone();
                let mut giv = given.clone();
                exp.sort();
                giv.sort();
                exp == giv
            }
            (Solution::Position { x, y, tolerance }, PlayerAnswer::Position {
                x: gx,
                y: gy,
            }) => {
                let dist = ((x - gx).powi(2) + (y - gy).powi(2)).sqrt();
                dist <= *tolerance
            }
            _ => false,
        }
    }
}
