use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct GradientGenerator;

impl CaptchaGenerator for GradientGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        // Simplified: "click the LIGHTEST swatch"
        // Swatch count: 3-6 based on complexity
        let swatch_count = if difficulty.complexity < 0.3 {
            3
        } else if difficulty.complexity < 0.6 {
            4
        } else if difficulty.complexity < 0.85 {
            5
        } else {
            6
        };

        // Pick a random hue
        let hue = rng.gen_range(0..360);
        let saturation = rng.gen_range(50..80);

        // Generate lightness values with spacing based on complexity
        // At low complexity: wide spread (30-80), at high: narrow spread
        let min_lightness = 25.0_f32;
        let max_lightness = 80.0_f32;
        let range = max_lightness - min_lightness;
        // At high complexity, compress the range
        let effective_range = range * (1.0 - difficulty.complexity * 0.6);
        let upper = (max_lightness - effective_range).max(min_lightness + 1.0);
        let base_lightness = rng.gen_range(min_lightness..upper);

        let mut lightness_values: Vec<(usize, f32)> = (0..swatch_count)
            .map(|i| {
                let t = i as f32 / (swatch_count - 1).max(1) as f32;
                let l = base_lightness + t * effective_range;
                (i, l)
            })
            .collect();

        // Find the lightest one (highest lightness value)
        let lightest_idx = lightness_values
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
            .0;

        // Shuffle the display order
        for i in (1..lightness_values.len()).rev() {
            let j = rng.gen_range(0..=i);
            lightness_values.swap(i, j);
        }

        // Find where the lightest ended up after shuffle
        let correct_display_idx = lightness_values
            .iter()
            .position(|&(orig_idx, _)| orig_idx == lightest_idx)
            .unwrap() as u32;

        // SVG layout
        let swatch_size = 70.0_f32;
        let gap = 12.0_f32;
        let total_swatches_w =
            swatch_count as f32 * swatch_size + (swatch_count as f32 - 1.0) * gap;
        let width = total_swatches_w + 40.0;
        let height = 150.0_f32;

        let mut svg = String::with_capacity(2048);
        write!(
            svg,
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width:.0} {height:.0}" width="{width:.0}" height="{height:.0}">"##
        )
        .unwrap();
        write!(
            svg,
            r##"<rect width="{width:.0}" height="{height:.0}" fill="#1a1a2e"/>"##
        )
        .unwrap();

        // Prompt
        write!(
            svg,
            r##"<text x="{:.0}" y="24" font-family="sans-serif" font-size="14" fill="#cccccc" text-anchor="middle">Click the LIGHTEST swatch</text>"##,
            width / 2.0
        )
        .unwrap();

        // Render swatches
        let start_x = (width - total_swatches_w) / 2.0;
        let swatch_y = 40.0;

        for (display_i, &(_orig_idx, lightness)) in lightness_values.iter().enumerate() {
            let x = start_x + display_i as f32 * (swatch_size + gap);

            // Swatch rectangle
            write!(
                svg,
                r##"<rect x="{x:.0}" y="{swatch_y:.0}" width="{swatch_size:.0}" height="{swatch_size:.0}" rx="6" fill="hsl({hue},{saturation}%,{lightness:.0}%)" stroke="#555" stroke-width="1"/>"##
            )
            .unwrap();

            // Clickable overlay
            write!(
                svg,
                r##"<rect x="{x:.0}" y="{swatch_y:.0}" width="{swatch_size:.0}" height="{swatch_size:.0}" fill="transparent" data-index="{display_i}" style="cursor:pointer"/>"##
            )
            .unwrap();
        }

        // Noise lines
        let noise_count = (difficulty.noise * 8.0) as u32;
        for _ in 0..noise_count {
            let x1 = rng.gen_range(0.0..width);
            let y1 = rng.gen_range(35.0..height);
            let x2 = rng.gen_range(0.0..width);
            let y2 = rng.gen_range(35.0..height);
            write!(
                svg,
                r##"<line x1="{x1:.0}" y1="{y1:.0}" x2="{x2:.0}" y2="{y2:.0}" stroke="#444" stroke-width="1" opacity="0.15"/>"##
            )
            .unwrap();
        }

        svg.push_str("</svg>");

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::SelectedIndices(vec![correct_display_idx]),
            expected_solve_time_ms: expected_solve_time(difficulty.time_limit_ms),
            point_value: CaptchaType::GradientOrder.base_points(),
            captcha_type: CaptchaType::GradientOrder,
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
    fn test_gradient_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.0,
            noise: 0.0,
        };
        let gen = GradientGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert_eq!(instance.captcha_type, CaptchaType::GradientOrder);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert_eq!(indices.len(), 1);
            assert!(indices[0] < 3); // low complexity = 3 swatches
        } else {
            panic!("Expected SelectedIndices solution");
        }
        if let RenderPayload::Svg(ref s) = instance.render_data {
            assert!(s.contains("<svg"));
            assert!(s.contains("data-index"));
            assert!(s.contains("LIGHTEST"));
        } else {
            panic!("Expected SVG render data");
        }
    }

    #[test]
    fn test_gradient_seed_determinism() {
        let difficulty = DifficultyParams {
            level: 10,
            round_number: 3,
            time_limit_ms: 7000,
            complexity: 0.5,
            noise: 0.3,
        };
        let gen = GradientGenerator;

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
    fn test_gradient_correct_answer_validates() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = GradientGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert!(gen.validate(
                &instance,
                &PlayerAnswer::SelectedIndices(indices.clone())
            ));
        }
    }

    #[test]
    fn test_gradient_wrong_answer_rejects() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = GradientGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![99])));
        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![])));
        assert!(!gen.validate(&instance, &PlayerAnswer::Text("wrong".to_string())));
    }
}
