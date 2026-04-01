use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct RotationGenerator;

/// Visual objects that have a clear "correct" orientation
#[derive(Debug, Clone, Copy)]
enum RotatableObject {
    Arrow,
    LetterR,
    LetterP,
    LetterF,
    LetterJ,
    House,
}

const ALL_OBJECTS: [RotatableObject; 6] = [
    RotatableObject::Arrow,
    RotatableObject::LetterR,
    RotatableObject::LetterP,
    RotatableObject::LetterF,
    RotatableObject::LetterJ,
    RotatableObject::House,
];

impl RotatableObject {
    fn svg_at(&self, cx: f32, cy: f32, size: f32, rotation_deg: f32, fill: &str) -> String {
        let inner = match self {
            RotatableObject::Arrow => {
                let half = size * 0.4;
                // Upward-pointing arrow
                format!(
                    r#"<polygon points="{cx:.0},{:.0} {:.0},{cy:.0} {:.0},{cy:.0} {:.0},{:.0} {:.0},{:.0} {:.0},{cy:.0} {:.0},{cy:.0}" fill="{fill}"/>"#,
                    cy - half,          // top
                    cx - half * 0.3,    // shaft left x
                    cx - half * 0.3,    // shaft left x
                    cx - half * 0.3,    // bottom-left shaft x
                    cy + half,          // bottom y
                    cx + half * 0.3,    // bottom-right shaft x
                    cy + half,          // bottom y
                    cx + half * 0.3,    // shaft right x
                    cx + half * 0.3,    // shaft right x
                )
            }
            RotatableObject::LetterR => {
                format!(
                    r#"<text x="{cx:.0}" y="{cy:.0}" font-family="sans-serif" font-size="{:.0}" font-weight="bold" fill="{fill}" text-anchor="middle" dominant-baseline="central">R</text>"#,
                    size * 0.7
                )
            }
            RotatableObject::LetterP => {
                format!(
                    r#"<text x="{cx:.0}" y="{cy:.0}" font-family="sans-serif" font-size="{:.0}" font-weight="bold" fill="{fill}" text-anchor="middle" dominant-baseline="central">P</text>"#,
                    size * 0.7
                )
            }
            RotatableObject::LetterF => {
                format!(
                    r#"<text x="{cx:.0}" y="{cy:.0}" font-family="sans-serif" font-size="{:.0}" font-weight="bold" fill="{fill}" text-anchor="middle" dominant-baseline="central">F</text>"#,
                    size * 0.7
                )
            }
            RotatableObject::LetterJ => {
                format!(
                    r#"<text x="{cx:.0}" y="{cy:.0}" font-family="sans-serif" font-size="{:.0}" font-weight="bold" fill="{fill}" text-anchor="middle" dominant-baseline="central">J</text>"#,
                    size * 0.7
                )
            }
            RotatableObject::House => {
                let half = size * 0.35;
                // Simple house: square body + triangle roof
                format!(
                    r#"<rect x="{:.0}" y="{cy:.0}" width="{:.0}" height="{:.0}" fill="{fill}"/><polygon points="{cx:.0},{:.0} {:.0},{cy:.0} {:.0},{cy:.0}" fill="{fill}"/>"#,
                    cx - half,          // body x
                    half * 2.0,         // body width
                    half,               // body height
                    cy - half * 0.6,    // roof peak y
                    cx - half,          // roof left x
                    cx + half,          // roof right x
                )
            }
        };
        format!(
            r#"<g transform="rotate({rotation_deg:.1},{cx:.0},{cy:.0})">{inner}</g>"#
        )
    }
}

impl CaptchaGenerator for RotationGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        // Object count: 4-8 based on complexity
        let object_count = 4 + (difficulty.complexity * 4.0) as usize;
        let object_count = object_count.min(8);

        // Pick which object type to use
        let object = ALL_OBJECTS[rng.gen_range(0..ALL_OBJECTS.len())];

        // The correct one is at index correct_idx with 0 rotation
        let correct_idx = rng.gen_range(0..object_count);

        // Generate rotation angles for wrong items; avoid angles close to 0
        let min_rotation = 45.0 - difficulty.complexity * 20.0; // gets harder: min goes from 45 to 25
        let min_rotation = min_rotation.max(20.0);

        let cell_size = 100.0;
        let cols = object_count.min(4) as u32;
        let rows = ((object_count as f32) / cols as f32).ceil() as u32;
        let width = cols as f32 * cell_size;
        let height = rows as f32 * cell_size;

        let mut svg = String::with_capacity(2048);
        write!(
            svg,
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width:.0} {:.0}" width="{width:.0}" height="{:.0}">"#,
            height + 40.0, height + 40.0
        )
        .unwrap();

        // Background
        write!(
            svg,
            r##"<rect width="{width:.0}" height="{:.0}" fill="#1a1a2e"/>"##,
            height + 40.0
        )
        .unwrap();

        // Prompt text
        write!(
            svg,
            r##"<text x="{:.0}" y="20" font-family="sans-serif" font-size="16" fill="#cccccc" text-anchor="middle">Select the correctly oriented object</text>"##,
            width / 2.0
        )
        .unwrap();

        // Add noise lines
        let noise_count = (5.0 + difficulty.noise * 15.0) as u32;
        for _ in 0..noise_count {
            let x1 = rng.gen::<f32>() * width;
            let y1 = rng.gen::<f32>() * (height + 40.0);
            let x2 = rng.gen::<f32>() * width;
            let y2 = rng.gen::<f32>() * (height + 40.0);
            let r = rng.gen_range(60..180);
            let g = rng.gen_range(60..180);
            let b = rng.gen_range(60..180);
            write!(
                svg,
                r#"<line x1="{x1:.0}" y1="{y1:.0}" x2="{x2:.0}" y2="{y2:.0}" stroke="rgb({r},{g},{b})" stroke-width="1" opacity="0.2"/>"#
            )
            .unwrap();
        }

        // Generate each cell
        let r_col = rng.gen_range(150..255);
        let g_col = rng.gen_range(150..255);
        let b_col = rng.gen_range(150..255);
        let fill = format!("rgb({r_col},{g_col},{b_col})");

        for i in 0..object_count {
            let col = (i % cols as usize) as f32;
            let row = (i / cols as usize) as f32;
            let cx = col * cell_size + cell_size / 2.0;
            let cy = row * cell_size + cell_size / 2.0 + 35.0; // offset for prompt

            // Cell border
            write!(
                svg,
                r##"<rect x="{:.0}" y="{:.0}" width="{cell_size:.0}" height="{cell_size:.0}" fill="none" stroke="#333" stroke-width="1"/>"##,
                col * cell_size,
                row * cell_size + 35.0
            )
            .unwrap();

            // Clickable overlay (placed after shape rendering below)

            let rotation = if i == correct_idx {
                0.0
            } else {
                // Pick a random rotation that's far enough from 0
                let sign = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
                sign * rng.gen_range(min_rotation..=(360.0 - min_rotation))
            };

            svg.push_str(&object.svg_at(cx, cy, cell_size, rotation, &fill));

            // Clickable overlay
            write!(
                svg,
                r#"<rect x="{:.0}" y="{:.0}" width="{cell_size:.0}" height="{cell_size:.0}" fill="transparent" data-index="{i}" style="cursor:pointer"/>"#,
                col * cell_size,
                row * cell_size + 35.0
            )
            .unwrap();
        }

        svg.push_str("</svg>");

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::SelectedIndices(vec![correct_idx as u32]),
            expected_solve_time_ms: expected_solve_time(difficulty.time_limit_ms),
            point_value: CaptchaType::RotatedObject.base_points(),
            captcha_type: CaptchaType::RotatedObject,
            time_limit_ms: difficulty.time_limit_ms,
        }
    }

    fn validate(&self, instance: &CaptchaInstance, answer: &PlayerAnswer) -> bool {
        instance.validate(answer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::rng_from_seed;

    #[test]
    fn test_rotation_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.0,
            noise: 0.0,
        };
        let gen = RotationGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert_eq!(instance.captcha_type, CaptchaType::RotatedObject);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert_eq!(indices.len(), 1);
            assert!(indices[0] < 4); // min object count is 4
        } else {
            panic!("Expected SelectedIndices solution");
        }
        if let RenderPayload::Svg(ref s) = instance.render_data {
            assert!(s.contains("<svg"));
        } else {
            panic!("Expected SVG render data");
        }
    }

    #[test]
    fn test_rotation_seed_determinism() {
        let difficulty = DifficultyParams {
            level: 10,
            round_number: 3,
            time_limit_ms: 7000,
            complexity: 0.5,
            noise: 0.3,
        };
        let gen = RotationGenerator;

        let mut rng1 = rng_from_seed(12345);
        let inst1 = gen.generate(&mut rng1, &difficulty);

        let mut rng2 = rng_from_seed(12345);
        let inst2 = gen.generate(&mut rng2, &difficulty);

        if let (Solution::SelectedIndices(i1), Solution::SelectedIndices(i2)) =
            (&inst1.solution, &inst2.solution)
        {
            assert_eq!(i1, i2);
        }
    }

    #[test]
    fn test_rotation_correct_answer_validates() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = RotationGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert!(gen.validate(
                &instance,
                &PlayerAnswer::SelectedIndices(indices.clone())
            ));
        }
    }

    #[test]
    fn test_rotation_wrong_answer_rejects() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = RotationGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        // Wrong index
        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![99])));
        // Empty
        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![])));
        // Wrong type
        assert!(!gen.validate(&instance, &PlayerAnswer::Text("wrong".to_string())));
    }

    #[test]
    fn test_rotation_difficulty_scaling() {
        let gen = RotationGenerator;
        let low = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.0,
            noise: 0.0,
        };
        let high = DifficultyParams {
            level: 50,
            round_number: 10,
            time_limit_ms: 10000,
            complexity: 1.0,
            noise: 1.0,
        };

        let mut rng1 = rng_from_seed(42);
        let inst_low = gen.generate(&mut rng1, &low);
        let mut rng2 = rng_from_seed(42);
        let inst_high = gen.generate(&mut rng2, &high);

        // Higher complexity should produce more objects (more SVG content)
        if let (RenderPayload::Svg(svg_low), RenderPayload::Svg(svg_high)) =
            (&inst_low.render_data, &inst_high.render_data)
        {
            assert!(svg_high.len() > svg_low.len());
        }
    }
}
