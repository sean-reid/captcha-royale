use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct OverlapGenerator;

#[derive(Debug, Clone, Copy)]
enum ShapeKind {
    Circle,
    Rectangle,
    Triangle,
}

const SHAPE_KINDS: [ShapeKind; 3] = [ShapeKind::Circle, ShapeKind::Rectangle, ShapeKind::Triangle];

fn render_shape(
    kind: ShapeKind,
    cx: f32,
    cy: f32,
    size: f32,
    fill: &str,
    opacity: f32,
) -> String {
    match kind {
        ShapeKind::Circle => {
            format!(
                r##"<circle cx="{cx:.0}" cy="{cy:.0}" r="{:.0}" fill="{fill}" opacity="{opacity:.2}" stroke="#888" stroke-width="1" stroke-opacity="{:.2}"/>"##,
                size / 2.0,
                opacity * 0.8
            )
        }
        ShapeKind::Rectangle => {
            let w = size;
            let h = size * 0.8;
            format!(
                r##"<rect x="{:.0}" y="{:.0}" width="{w:.0}" height="{h:.0}" fill="{fill}" opacity="{opacity:.2}" stroke="#888" stroke-width="1" stroke-opacity="{:.2}"/>"##,
                cx - w / 2.0,
                cy - h / 2.0,
                opacity * 0.8
            )
        }
        ShapeKind::Triangle => {
            let half = size / 2.0;
            format!(
                r##"<polygon points="{cx:.0},{:.0} {:.0},{:.0} {:.0},{:.0}" fill="{fill}" opacity="{opacity:.2}" stroke="#888" stroke-width="1" stroke-opacity="{:.2}"/>"##,
                cy - half * 0.8,
                cx - half,
                cy + half * 0.6,
                cx + half,
                cy + half * 0.6,
                opacity * 0.8
            )
        }
    }
}

impl CaptchaGenerator for OverlapGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        // Shape count based on complexity
        let shape_count = if difficulty.complexity < 0.3 {
            rng.gen_range(3..=5)
        } else if difficulty.complexity < 0.6 {
            rng.gen_range(5..=7)
        } else {
            rng.gen_range(7..=10)
        };

        let width = 400.0_f32;
        let height = 300.0_f32;
        let draw_area_top = 45.0;

        // Opacity: lower at high complexity
        let base_opacity = 0.5 - difficulty.complexity * 0.2;
        let base_opacity = base_opacity.max(0.3);

        // Color similarity increases with complexity
        let base_hue = rng.gen_range(0..360);
        let hue_spread = 300.0 * (1.0 - difficulty.complexity * 0.7);

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
            r##"<text x="{:.0}" y="24" font-family="sans-serif" font-size="14" fill="#cccccc" text-anchor="middle">How many shapes are there?</text>"##,
            width / 2.0
        )
        .unwrap();

        // Generate and render shapes
        let margin = 50.0;
        let draw_h = height - draw_area_top - 20.0;

        for i in 0..shape_count {
            let kind = SHAPE_KINDS[rng.gen_range(0..SHAPE_KINDS.len())];
            let size = rng.gen_range(40.0..80.0_f32);
            let cx = rng.gen_range(margin..(width - margin));
            let cy = rng.gen_range((draw_area_top + margin / 2.0)..(draw_area_top + draw_h));

            let hue = (base_hue as f32 + rng.gen_range(0.0..hue_spread)) % 360.0;
            let sat = rng.gen_range(50..80);
            let light = rng.gen_range(45..65);
            let fill = format!("hsl({hue:.0},{sat}%,{light}%)");

            let opacity = base_opacity + rng.gen_range(-0.05..0.05_f32);
            let opacity = opacity.clamp(0.25, 0.55);

            svg.push_str(&render_shape(kind, cx, cy, size, &fill, opacity));

            // At high complexity, add noise shapes that are very faint (should NOT be counted)
            // Actually, we should NOT add extra shapes - the count is exactly shape_count
            let _ = i; // suppress unused warning
        }

        // Noise lines and dots
        let noise_count = (difficulty.noise * 12.0) as u32;
        for _ in 0..noise_count {
            let x1 = rng.gen_range(0.0..width);
            let y1 = rng.gen_range(draw_area_top..height);
            let x2 = rng.gen_range(0.0..width);
            let y2 = rng.gen_range(draw_area_top..height);
            let gray = rng.gen_range(60..140);
            write!(
                svg,
                r##"<line x1="{x1:.0}" y1="{y1:.0}" x2="{x2:.0}" y2="{y2:.0}" stroke="rgb({gray},{gray},{gray})" stroke-width="1" opacity="0.12"/>"##
            )
            .unwrap();
        }

        svg.push_str("</svg>");

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::Number(shape_count as f64),
            expected_solve_time_ms: expected_solve_time(difficulty.time_limit_ms),
            point_value: CaptchaType::OverlapCounting.base_points(),
            captcha_type: CaptchaType::OverlapCounting,
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
    fn test_overlap_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.0,
            noise: 0.0,
        };
        let gen = OverlapGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert_eq!(instance.captcha_type, CaptchaType::OverlapCounting);
        if let Solution::Number(n) = instance.solution {
            assert!(n >= 3.0 && n <= 10.0);
        } else {
            panic!("Expected Number solution");
        }
        if let RenderPayload::Svg(ref s) = instance.render_data {
            assert!(s.contains("<svg"));
            assert!(s.contains("How many shapes"));
        } else {
            panic!("Expected SVG render data");
        }
    }

    #[test]
    fn test_overlap_seed_determinism() {
        let difficulty = DifficultyParams {
            level: 10,
            round_number: 3,
            time_limit_ms: 7000,
            complexity: 0.5,
            noise: 0.3,
        };
        let gen = OverlapGenerator;

        let mut rng1 = rng_from_seed(12345);
        let inst1 = gen.generate(&mut rng1, &difficulty);

        let mut rng2 = rng_from_seed(12345);
        let inst2 = gen.generate(&mut rng2, &difficulty);

        if let (Solution::Number(n1), Solution::Number(n2)) = (&inst1.solution, &inst2.solution) {
            assert_eq!(n1, n2);
        }
    }

    #[test]
    fn test_overlap_correct_answer_validates() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = OverlapGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::Number(n) = instance.solution {
            assert!(gen.validate(&instance, &PlayerAnswer::Number(n)));
        }
    }

    #[test]
    fn test_overlap_wrong_answer_rejects() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = OverlapGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert!(!gen.validate(&instance, &PlayerAnswer::Number(999.0)));
        assert!(!gen.validate(&instance, &PlayerAnswer::Text("wrong".to_string())));
        assert!(!gen.validate(
            &instance,
            &PlayerAnswer::SelectedIndices(vec![0])
        ));
    }
}
