use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct BalanceGenerator;

impl CaptchaGenerator for BalanceGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        // Always use click-based "which side is heavier" variant
        // Number of weights per side: 1-3 based on complexity
        let weights_per_side = if difficulty.complexity < 0.4 {
            1
        } else if difficulty.complexity < 0.7 {
            2
        } else {
            3
        };

        // Generate weights for each side
        let mut left_weights: Vec<u32> = Vec::new();
        let mut right_weights: Vec<u32> = Vec::new();

        for _ in 0..weights_per_side {
            left_weights.push(rng.gen_range(1..=20));
            right_weights.push(rng.gen_range(1..=20));
        }

        let left_total: u32 = left_weights.iter().sum();
        let right_total: u32 = right_weights.iter().sum();

        // Make sure sides are not equal
        if left_total == right_total {
            // Bump the last weight on one side
            if rng.gen_bool(0.5) {
                left_weights[weights_per_side - 1] += rng.gen_range(1..=5);
            } else {
                right_weights[weights_per_side - 1] += rng.gen_range(1..=5);
            }
        }

        let left_total: u32 = left_weights.iter().sum();
        let right_total: u32 = right_weights.iter().sum();

        // Randomly ask "heavier" or "lighter"
        let ask_lighter = rng.gen_bool(0.5);
        let heavier_side: u32 = if left_total > right_total { 0 } else { 1 };
        let correct_side: u32 = if ask_lighter { 1 - heavier_side } else { heavier_side };

        // SVG dimensions
        let width = 400.0_f32;
        let height = 260.0_f32;

        let mut svg = String::with_capacity(4096);
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
            r##"<text x="{:.0}" y="22" font-family="sans-serif" font-size="14" fill="#cccccc" text-anchor="middle">Which side is {}? Click left or right.</text>"##,
            width / 2.0,
            if ask_lighter { "lighter" } else { "heavier" }
        )
        .unwrap();

        // Fulcrum triangle
        let fulcrum_cx = width / 2.0;
        let fulcrum_top_y = 180.0;
        let fulcrum_bot_y = 230.0;
        let fulcrum_half_w = 25.0;
        write!(
            svg,
            r##"<polygon points="{fulcrum_cx:.0},{fulcrum_top_y:.0} {:.0},{fulcrum_bot_y:.0} {:.0},{fulcrum_bot_y:.0}" fill="#555566" stroke="#777" stroke-width="1"/>"##,
            fulcrum_cx - fulcrum_half_w,
            fulcrum_cx + fulcrum_half_w
        )
        .unwrap();

        // Beam - always level so the player must compare the numbers
        let beam_y = fulcrum_top_y;
        let beam_half_w = 150.0;
        let tilt = 0.0;
        write!(
            svg,
            r##"<line x1="{:.0}" y1="{:.0}" x2="{:.0}" y2="{:.0}" stroke="#aabbcc" stroke-width="4" stroke-linecap="round"/>"##,
            fulcrum_cx - beam_half_w,
            beam_y + tilt,
            fulcrum_cx + beam_half_w,
            beam_y - tilt
        )
        .unwrap();

        // Draw weights on left side
        let left_base_x = fulcrum_cx - beam_half_w + 20.0;
        let left_base_y = beam_y + tilt;
        let weight_radius = 18.0;
        let spacing = (beam_half_w - 40.0) / weights_per_side.max(1) as f32;

        for (i, &w) in left_weights.iter().enumerate() {
            let cx = left_base_x + i as f32 * spacing;
            let cy = left_base_y - weight_radius - 5.0;
            // Weight circle
            write!(
                svg,
                r##"<circle cx="{cx:.0}" cy="{cy:.0}" r="{weight_radius:.0}" fill="#4477aa" stroke="#6699cc" stroke-width="1.5"/>"##
            )
            .unwrap();
            // Weight number
            write!(
                svg,
                r##"<text x="{cx:.0}" y="{:.0}" font-family="sans-serif" font-size="14" font-weight="bold" fill="#ffffff" text-anchor="middle" dominant-baseline="central">{w}</text>"##,
                cy
            )
            .unwrap();
        }

        // Draw weights on right side
        let right_base_x = fulcrum_cx + 30.0;
        let right_base_y = beam_y - tilt;

        for (i, &w) in right_weights.iter().enumerate() {
            let cx = right_base_x + i as f32 * spacing;
            let cy = right_base_y - weight_radius - 5.0;
            // Weight circle
            write!(
                svg,
                r##"<circle cx="{cx:.0}" cy="{cy:.0}" r="{weight_radius:.0}" fill="#aa4477" stroke="#cc6699" stroke-width="1.5"/>"##
            )
            .unwrap();
            // Weight number
            write!(
                svg,
                r##"<text x="{cx:.0}" y="{:.0}" font-family="sans-serif" font-size="14" font-weight="bold" fill="#ffffff" text-anchor="middle" dominant-baseline="central">{w}</text>"##,
                cy
            )
            .unwrap();
        }

        // Clickable left region
        write!(
            svg,
            r##"<rect x="0" y="35" width="{:.0}" height="{:.0}" fill="transparent" data-index="0" style="cursor:pointer"/>"##,
            width / 2.0,
            height - 35.0
        )
        .unwrap();
        // Clickable right region
        write!(
            svg,
            r##"<rect x="{:.0}" y="35" width="{:.0}" height="{:.0}" fill="transparent" data-index="1" style="cursor:pointer"/>"##,
            width / 2.0,
            width / 2.0,
            height - 35.0
        )
        .unwrap();

        // Side labels
        write!(
            svg,
            r##"<text x="{:.0}" y="245" font-family="sans-serif" font-size="12" fill="#888" text-anchor="middle">LEFT</text>"##,
            width / 4.0
        )
        .unwrap();
        write!(
            svg,
            r##"<text x="{:.0}" y="245" font-family="sans-serif" font-size="12" fill="#888" text-anchor="middle">RIGHT</text>"##,
            width * 3.0 / 4.0
        )
        .unwrap();

        // Noise lines
        let noise_count = (difficulty.noise * 8.0) as u32;
        for _ in 0..noise_count {
            let x1 = rng.gen_range(0.0..width);
            let y1 = rng.gen_range(30.0..height);
            let x2 = rng.gen_range(0.0..width);
            let y2 = rng.gen_range(30.0..height);
            write!(
                svg,
                r##"<line x1="{x1:.0}" y1="{y1:.0}" x2="{x2:.0}" y2="{y2:.0}" stroke="#444" stroke-width="1" opacity="0.15"/>"##
            )
            .unwrap();
        }

        svg.push_str("</svg>");

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::SelectedIndices(vec![correct_side]),
            expected_solve_time_ms: expected_solve_time(difficulty.time_limit_ms),
            point_value: CaptchaType::BalanceScale.base_points(),
            captcha_type: CaptchaType::BalanceScale,
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
    fn test_balance_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.0,
            noise: 0.0,
        };
        let gen = BalanceGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert_eq!(instance.captcha_type, CaptchaType::BalanceScale);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert_eq!(indices.len(), 1);
            assert!(indices[0] <= 1); // 0 = left, 1 = right
        } else {
            panic!("Expected SelectedIndices solution");
        }
        if let RenderPayload::Svg(ref s) = instance.render_data {
            assert!(s.contains("<svg"));
            assert!(s.contains("data-index"));
            assert!(s.contains("Which side is"));
        } else {
            panic!("Expected SVG render data");
        }
    }

    #[test]
    fn test_balance_seed_determinism() {
        let difficulty = DifficultyParams {
            level: 10,
            round_number: 3,
            time_limit_ms: 7000,
            complexity: 0.5,
            noise: 0.3,
        };
        let gen = BalanceGenerator;

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
    fn test_balance_correct_answer_validates() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = BalanceGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert!(gen.validate(
                &instance,
                &PlayerAnswer::SelectedIndices(indices.clone())
            ));
        }
    }

    #[test]
    fn test_balance_wrong_answer_rejects() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = BalanceGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            // Wrong side
            let wrong = if indices[0] == 0 { 1 } else { 0 };
            assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![wrong])));
        }
        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![99])));
        assert!(!gen.validate(&instance, &PlayerAnswer::Text("wrong".to_string())));
    }
}
