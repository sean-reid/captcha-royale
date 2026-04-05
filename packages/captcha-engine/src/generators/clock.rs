use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct ClockReadingGenerator;

impl CaptchaGenerator for ClockReadingGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        let c = difficulty.complexity;
        let n = difficulty.noise;

        // Generate hour (1-12)
        let hour: u32 = rng.gen_range(1..=12);

        // Generate minutes based on complexity
        let minute: u32 = if c < 0.3 {
            // Low complexity: only :00 or :30
            if rng.gen_bool(0.5) { 0 } else { 30 }
        } else if c < 0.6 {
            // Medium: :00, :15, :30, :45
            [0, 15, 30, 45][rng.gen_range(0..4)]
        } else {
            // High: any 5-minute increment
            rng.gen_range(0..12) * 5
        };

        // Format the answer as "H:MM"
        let answer_text = format!("{}:{:02}", hour, minute);

        let width = 300;
        let height = 300;
        let cx = width as f32 / 2.0;
        let cy = height as f32 / 2.0;
        let clock_radius = 120.0;

        // Optional slight rotation of the whole clock at high complexity
        let clock_rotation = if c > 0.7 {
            rng.gen_range(-15.0..15.0_f32)
        } else {
            0.0
        };

        let mut svg = String::with_capacity(4096);
        write!(
            svg,
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}">"##
        )
        .unwrap();

        // Background
        write!(
            svg,
            r##"<rect width="{width}" height="{height}" fill="#1a1a2e"/>"##
        )
        .unwrap();

        // Group with optional rotation
        write!(
            svg,
            r##"<g transform="rotate({clock_rotation:.1},{cx:.0},{cy:.0})">"##
        )
        .unwrap();

        // Clock face
        write!(
            svg,
            r##"<circle cx="{cx:.0}" cy="{cy:.0}" r="{clock_radius:.0}" fill="#f0f0e8" stroke="#333333" stroke-width="3"/>"##
        )
        .unwrap();

        // Center dot
        write!(
            svg,
            r##"<circle cx="{cx:.0}" cy="{cy:.0}" r="4" fill="#333333"/>"##
        )
        .unwrap();

        // Tick marks (all 12)
        for i in 0..12 {
            let angle = (i as f32) * 30.0 - 90.0;
            let angle_rad = angle.to_radians();
            let inner_r = clock_radius - 12.0;
            let outer_r = clock_radius - 3.0;
            let x1 = cx + inner_r * angle_rad.cos();
            let y1 = cy + inner_r * angle_rad.sin();
            let x2 = cx + outer_r * angle_rad.cos();
            let y2 = cy + outer_r * angle_rad.sin();
            let sw = if i % 3 == 0 { 3.0 } else { 1.5 };
            write!(
                svg,
                r##"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="#333333" stroke-width="{sw:.1}"/>"##
            )
            .unwrap();
        }

        // Numbers: at low complexity show 12/3/6/9, at high show all 12
        let show_all_numbers = c > 0.4;
        for i in 1..=12 {
            let show = if show_all_numbers {
                true
            } else {
                i % 3 == 0
            };
            if show {
                let angle = (i as f32) * 30.0 - 90.0;
                let angle_rad = angle.to_radians();
                let num_r = clock_radius - 25.0;
                let nx = cx + num_r * angle_rad.cos();
                let ny = cy + num_r * angle_rad.sin();
                write!(
                    svg,
                    r##"<text x="{nx:.1}" y="{ny:.1}" font-family="sans-serif" font-size="16" font-weight="bold" fill="#333333" text-anchor="middle" dominant-baseline="central">{i}</text>"##
                )
                .unwrap();
            }
        }

        // Calculate hand angles
        // Hour hand: 360/12 = 30 degrees per hour, plus minute contribution
        let hour_angle = ((hour % 12) as f32 + minute as f32 / 60.0) * 30.0 - 90.0;
        let hour_angle_rad = hour_angle.to_radians();

        // Minute hand: 360/60 = 6 degrees per minute
        let minute_angle = minute as f32 * 6.0 - 90.0;
        let minute_angle_rad = minute_angle.to_radians();

        // Hour hand: shorter and thicker
        let hour_length = clock_radius * 0.55;
        // At high complexity, hands are closer in length
        let length_diff = if c > 0.6 { 0.15 } else { 0.25 };
        let minute_length = clock_radius * (0.55 + length_diff);

        let hx = cx + hour_length * hour_angle_rad.cos();
        let hy = cy + hour_length * hour_angle_rad.sin();
        write!(
            svg,
            r##"<line x1="{cx:.0}" y1="{cy:.0}" x2="{hx:.1}" y2="{hy:.1}" stroke="#222222" stroke-width="5" stroke-linecap="round"/>"##
        )
        .unwrap();

        // Minute hand: longer and thinner
        let mx = cx + minute_length * minute_angle_rad.cos();
        let my = cy + minute_length * minute_angle_rad.sin();
        write!(
            svg,
            r##"<line x1="{cx:.0}" y1="{cy:.0}" x2="{mx:.1}" y2="{my:.1}" stroke="#222222" stroke-width="3" stroke-linecap="round"/>"##
        )
        .unwrap();

        // Noise lines at high noise
        let noise_line_count = (n * 6.0) as u32;
        for _ in 0..noise_line_count {
            let angle = rng.gen_range(0.0..360.0_f32).to_radians();
            let r1 = rng.gen_range(20.0..clock_radius * 0.8);
            let r2 = rng.gen_range(20.0..clock_radius * 0.8);
            let lx1 = cx + r1 * angle.cos();
            let ly1 = cy + r1 * angle.sin();
            let lx2 = cx + r2 * (angle + 0.5).cos();
            let ly2 = cy + r2 * (angle + 0.5).sin();
            let r = rng.gen_range(80..180);
            let g = rng.gen_range(80..180);
            let b = rng.gen_range(80..180);
            let opacity = 0.15 + rng.gen::<f32>() * 0.2;
            write!(
                svg,
                r##"<line x1="{lx1:.1}" y1="{ly1:.1}" x2="{lx2:.1}" y2="{ly2:.1}" stroke="rgb({r},{g},{b})" stroke-width="1.5" opacity="{opacity:.2}"/>"##
            )
            .unwrap();
        }

        svg.push_str("</g>");

        // Prompt
        write!(
            svg,
            r##"<text x="{}" y="{}" font-family="sans-serif" font-size="13" fill="#aaaaaa" text-anchor="middle">What time is shown? (H:MM)</text>"##,
            width / 2,
            height - 8
        )
        .unwrap();

        svg.push_str("</svg>");

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::Text(answer_text),
            expected_solve_time_ms: expected_solve_time(difficulty.time_limit_ms),
            point_value: CaptchaType::ClockReading.base_points(),
            captcha_type: CaptchaType::ClockReading,
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
    fn test_clockreading_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 4000,
            complexity: 0.0,
            noise: 0.0,
        };
        let gen = ClockReadingGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert_eq!(instance.captcha_type, CaptchaType::ClockReading);
        assert!(matches!(instance.solution, Solution::Text(_)));
        if let RenderPayload::Svg(ref s) = instance.render_data {
            assert!(s.contains("<svg"));
            assert!(s.contains("<circle"));
            assert!(s.contains("<line"));
        } else {
            panic!("Expected SVG render data");
        }
    }

    #[test]
    fn test_clockreading_seed_determinism() {
        let difficulty = DifficultyParams {
            level: 10,
            round_number: 3,
            time_limit_ms: 6000,
            complexity: 0.5,
            noise: 0.3,
        };
        let gen = ClockReadingGenerator;

        let mut rng1 = rng_from_seed(555);
        let inst1 = gen.generate(&mut rng1, &difficulty);

        let mut rng2 = rng_from_seed(555);
        let inst2 = gen.generate(&mut rng2, &difficulty);

        if let (Solution::Text(t1), Solution::Text(t2)) = (inst1.solution, inst2.solution) {
            assert_eq!(t1, t2);
        } else {
            panic!("Expected Text solutions");
        }
    }

    #[test]
    fn test_clockreading_correct_answer_validates() {
        let mut rng = rng_from_seed(77);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = ClockReadingGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::Text(ref answer) = instance.solution {
            assert!(gen.validate(&instance, &PlayerAnswer::Text(answer.clone())));
        } else {
            panic!("Expected Text solution");
        }
    }

    #[test]
    fn test_clockreading_wrong_answer_rejects() {
        let mut rng = rng_from_seed(77);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = ClockReadingGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert!(!gen.validate(&instance, &PlayerAnswer::Text("99:99".to_string())));
        assert!(!gen.validate(&instance, &PlayerAnswer::Number(0.0)));
    }

    #[test]
    fn test_clockreading_answer_format() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 4000,
            complexity: 0.0,
            noise: 0.0,
        };
        let gen = ClockReadingGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::Text(ref answer) = instance.solution {
            let parts: Vec<&str> = answer.split(':').collect();
            assert_eq!(parts.len(), 2);
            let h: u32 = parts[0].parse().unwrap();
            let m: u32 = parts[1].parse().unwrap();
            assert!((1..=12).contains(&h));
            assert!(m < 60);
            assert_eq!(parts[1].len(), 2);
        }
    }
}
