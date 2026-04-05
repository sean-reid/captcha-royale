use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct DotCountGenerator;

impl CaptchaGenerator for DotCountGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        let c = difficulty.complexity;
        let n = difficulty.noise;

        // Dot count scales with complexity: low 3-8, high 10-25
        let min_dots = 3 + (c * 7.0) as u32;
        let max_dots = 8 + (c * 17.0) as u32;
        let dot_count = rng.gen_range(min_dots..=max_dots);

        let width = 400;
        let height = 300;

        // Generate dot positions with spacing that decreases at higher complexity
        let min_spacing = 30.0 - c * 20.0; // 30 at easy, 10 at hard
        let min_spacing = min_spacing.max(5.0);

        let mut dots: Vec<(f32, f32)> = Vec::new();
        let margin = 30.0;

        for _ in 0..dot_count * 20 {
            if dots.len() >= dot_count as usize {
                break;
            }
            let x = rng.gen_range(margin..width as f32 - margin);
            let y = rng.gen_range(margin..height as f32 - margin);

            let ok = dots.iter().all(|(dx, dy)| {
                ((x - dx).powi(2) + (y - dy).powi(2)).sqrt() >= min_spacing
            });
            if ok {
                dots.push((x, y));
            }
        }
        // If we couldn't place enough with spacing, just place the rest randomly
        while dots.len() < dot_count as usize {
            let x = rng.gen_range(margin..width as f32 - margin);
            let y = rng.gen_range(margin..height as f32 - margin);
            dots.push((x, y));
        }

        // Dot radius
        let dot_radius = 10.0 - c * 4.0; // 10 at easy, 6 at hard
        let dot_radius = dot_radius.max(5.0);

        // Color palette for dots
        let colors = [
            "#e74c3c", "#3498db", "#2ecc71", "#f39c12", "#9b59b6",
            "#1abc9c", "#e67e22", "#ec407a", "#26c6da", "#66bb6a",
        ];

        let mut svg = String::with_capacity(4096);
        write!(
            svg,
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}">"##
        )
        .unwrap();

        // Dark background
        write!(
            svg,
            r##"<rect width="{width}" height="{height}" fill="#111122"/>"##
        )
        .unwrap();

        // Prompt
        write!(
            svg,
            r##"<text x="{}" y="20" font-family="sans-serif" font-size="14" fill="#aaaaaa" text-anchor="middle">How many dots?</text>"##,
            width / 2
        )
        .unwrap();

        // Draw noise circles (smaller, dimmer) before real dots
        let noise_count = (n * 15.0) as u32;
        for _ in 0..noise_count {
            let nx = rng.gen_range(10.0..width as f32 - 10.0);
            let ny = rng.gen_range(30.0..height as f32 - 10.0);
            let nr = rng.gen_range(1.5..dot_radius * 0.5);
            let r = rng.gen_range(40..100);
            let g = rng.gen_range(40..100);
            let b = rng.gen_range(40..100);
            let opacity = 0.1 + rng.gen::<f32>() * 0.2;
            write!(
                svg,
                r##"<circle cx="{nx:.1}" cy="{ny:.1}" r="{nr:.1}" fill="rgb({r},{g},{b})" opacity="{opacity:.2}"/>"##
            )
            .unwrap();
        }

        // Draw actual dots
        for (i, (x, y)) in dots.iter().enumerate() {
            let color = if c < 0.5 {
                // Distinct colors at low complexity
                colors[i % colors.len()]
            } else {
                // More similar colors at high complexity
                let base_idx = rng.gen_range(0..3);
                colors[base_idx]
            };
            let size_jitter = rng.gen_range(-1.5..1.5_f32) * c;
            let r = (dot_radius + size_jitter).max(3.0);
            write!(
                svg,
                r##"<circle cx="{x:.1}" cy="{y:.1}" r="{r:.1}" fill="{color}"/>"##
            )
            .unwrap();
        }

        // Extra noise: faint overlapping circles at high noise
        let overlay_noise = (n * 8.0) as u32;
        for _ in 0..overlay_noise {
            let nx = rng.gen_range(10.0..width as f32 - 10.0);
            let ny = rng.gen_range(30.0..height as f32 - 10.0);
            let nr = rng.gen_range(2.0..dot_radius * 0.7);
            let r = rng.gen_range(60..140);
            let g = rng.gen_range(60..140);
            let b = rng.gen_range(60..140);
            let opacity = 0.08 + rng.gen::<f32>() * 0.15;
            write!(
                svg,
                r##"<circle cx="{nx:.1}" cy="{ny:.1}" r="{nr:.1}" fill="rgb({r},{g},{b})" opacity="{opacity:.2}"/>"##
            )
            .unwrap();
        }

        svg.push_str("</svg>");

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::Number(dot_count as f64),
            expected_solve_time_ms: expected_solve_time(difficulty.time_limit_ms),
            point_value: CaptchaType::DotCount.base_points(),
            captcha_type: CaptchaType::DotCount,
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
    fn test_dotcount_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 4000,
            complexity: 0.0,
            noise: 0.0,
        };
        let gen = DotCountGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert_eq!(instance.captcha_type, CaptchaType::DotCount);
        assert!(matches!(instance.solution, Solution::Number(_)));
        if let RenderPayload::Svg(ref s) = instance.render_data {
            assert!(s.contains("<svg"));
            assert!(s.contains("<circle"));
        } else {
            panic!("Expected SVG render data");
        }
    }

    #[test]
    fn test_dotcount_seed_determinism() {
        let difficulty = DifficultyParams {
            level: 10,
            round_number: 3,
            time_limit_ms: 6000,
            complexity: 0.5,
            noise: 0.3,
        };
        let gen = DotCountGenerator;

        let mut rng1 = rng_from_seed(555);
        let inst1 = gen.generate(&mut rng1, &difficulty);

        let mut rng2 = rng_from_seed(555);
        let inst2 = gen.generate(&mut rng2, &difficulty);

        if let (Solution::Number(n1), Solution::Number(n2)) = (inst1.solution, inst2.solution) {
            assert!((n1 - n2).abs() < 0.001);
        } else {
            panic!("Expected Number solutions");
        }
    }

    #[test]
    fn test_dotcount_correct_answer_validates() {
        let mut rng = rng_from_seed(77);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = DotCountGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::Number(answer) = instance.solution {
            assert!(gen.validate(&instance, &PlayerAnswer::Number(answer)));
        } else {
            panic!("Expected Number solution");
        }
    }

    #[test]
    fn test_dotcount_wrong_answer_rejects() {
        let mut rng = rng_from_seed(77);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = DotCountGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert!(!gen.validate(&instance, &PlayerAnswer::Number(999.0)));
        assert!(!gen.validate(&instance, &PlayerAnswer::Text("wrong".to_string())));
    }
}
