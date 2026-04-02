use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct CascadeGenerator;

impl CaptchaGenerator for CascadeGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        // 4-6 sub-challenges based on complexity
        let challenge_count = 4 + (difficulty.complexity * 2.0) as usize;
        let challenge_count = challenge_count.min(6);

        // Generate random numbers for each sub-challenge
        let numbers: Vec<String> = (0..challenge_count)
            .map(|_| {
                if difficulty.complexity > 0.5 {
                    // 2-digit numbers at higher difficulty
                    rng.gen_range(10..99).to_string()
                } else {
                    // 1-digit numbers at lower difficulty
                    rng.gen_range(1..9).to_string()
                }
            })
            .collect();

        let solution_text = numbers.join("|");
        let total_time = difficulty.time_limit_ms;

        let svg = generate_cascade_svg(rng, &numbers, total_time, difficulty);

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::Text(solution_text),
            expected_solve_time_ms: expected_solve_time(total_time),
            point_value: CaptchaType::TimePressureCascade.base_points(),
            captcha_type: CaptchaType::TimePressureCascade,
            time_limit_ms: total_time,
        }
    }

    fn validate(&self, instance: &CaptchaInstance, answer: &PlayerAnswer) -> bool {
        instance.validate(answer)
    }
}

fn generate_cascade_svg(
    rng: &mut ChaCha8Rng,
    numbers: &[String],
    total_time_ms: u32,
    difficulty: &DifficultyParams,
) -> String {
    let width = 400;
    let challenge_height = 120;
    let height = challenge_height * numbers.len() as i32;
    let c = difficulty.complexity;

    let mut svg = String::with_capacity(8192);
    write!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}">"#
    )
    .unwrap();

    // Full background
    write!(
        svg,
        r##"<rect width="{width}" height="{height}" fill="#1a1a2e"/>"##
    )
    .unwrap();

    // Compute decreasing time allocations for each sub-challenge
    let count = numbers.len();
    let time_shares: Vec<f32> = (0..count)
        .map(|i| 1.0 - (i as f32 / count as f32) * 0.6)
        .collect();
    let total_shares: f32 = time_shares.iter().sum();
    let time_per_challenge: Vec<u32> = time_shares
        .iter()
        .map(|s| ((s / total_shares) * total_time_ms as f32) as u32)
        .collect();

    for (i, number) in numbers.iter().enumerate() {
        let y_offset = (i as i32) * challenge_height;

        // Separator line
        if i > 0 {
            write!(
                svg,
                r##"<line x1="0" y1="{y_offset}" x2="{width}" y2="{y_offset}" stroke="#333355" stroke-width="2"/>"##
            )
            .unwrap();
        }

        // Challenge label
        write!(
            svg,
            r##"<text x="15" y="{}" font-family="monospace" font-size="12" fill="#666688">#{}</text>"##,
            y_offset + 18,
            i + 1,
        )
        .unwrap();

        // Time indicator bar — gets shorter and redder for later challenges
        let bar_width = 100.0;
        let time_frac = time_per_challenge[i] as f32 / time_per_challenge[0] as f32;
        let bar_actual = bar_width * time_frac;
        // Color interpolation: green -> yellow -> red
        let red = ((1.0 - time_frac) * 255.0) as u8;
        let green = (time_frac * 200.0) as u8;
        write!(
            svg,
            r#"<rect x="{}" y="{}" width="{bar_actual:.0}" height="8" rx="4" fill="rgb({red},{green},50)"/>"#,
            width as f32 - bar_width - 15.0,
            y_offset + 10,
        )
        .unwrap();

        // Time label
        let time_secs = time_per_challenge[i] as f32 / 1000.0;
        write!(
            svg,
            r##"<text x="{}" y="{}" font-family="monospace" font-size="10" fill="#888899" text-anchor="end">{time_secs:.1}s</text>"##,
            width as f32 - bar_width - 20.0,
            y_offset + 18,
        )
        .unwrap();

        // Draw distorted number
        let center_x = width as f32 / 2.0;
        let center_y = y_offset as f32 + challenge_height as f32 / 2.0 + 10.0;
        draw_distorted_number(&mut svg, rng, number, center_x, center_y, c);

        // Noise layer for this challenge section
        let noise_count = 5 + (c * 15.0) as u32;
        for _ in 0..noise_count {
            let nx = rng.gen_range(10..width - 10);
            let ny = y_offset + rng.gen_range(25..challenge_height - 5);
            let nr = 1.0 + rng.gen::<f32>() * (1.5 + c * 2.0);
            let r = rng.gen_range(80..200);
            let g = rng.gen_range(80..200);
            let b = rng.gen_range(80..200);
            let opacity = 0.15 + rng.gen::<f32>() * 0.25;
            write!(
                svg,
                r#"<circle cx="{nx}" cy="{ny}" r="{nr:.1}" fill="rgb({r},{g},{b})" opacity="{opacity:.2}"/>"#
            )
            .unwrap();
        }

        // Noise lines crossing the number
        let line_count = 2 + (c * 4.0) as u32;
        for _ in 0..line_count {
            let lx1 = rng.gen_range(50..width - 50);
            let ly1 = y_offset as f32 + rng.gen_range(30.0..challenge_height as f32 - 10.0);
            let lx2 = rng.gen_range(50..width - 50);
            let ly2 = y_offset as f32 + rng.gen_range(30.0..challenge_height as f32 - 10.0);
            let r = rng.gen_range(120..220);
            let g = rng.gen_range(120..220);
            let b = rng.gen_range(120..220);
            let sw = 1.5 + rng.gen::<f32>() * (1.0 + c * 2.0);
            let opacity = 0.3 + c * 0.2;
            write!(
                svg,
                r#"<line x1="{lx1}" y1="{ly1:.0}" x2="{lx2}" y2="{ly2:.0}" stroke="rgb({r},{g},{b})" stroke-width="{sw:.1}" opacity="{opacity:.2}"/>"#
            )
            .unwrap();
        }
    }

    svg.push_str("</svg>");
    svg
}

fn draw_distorted_number(
    svg: &mut String,
    rng: &mut ChaCha8Rng,
    number: &str,
    cx: f32,
    cy: f32,
    complexity: f32,
) {
    let fonts = ["monospace", "serif", "sans-serif"];
    let char_spacing = 35.0;
    let total_width = char_spacing * (number.len() as f32 - 1.0);
    let start_x = cx - total_width / 2.0;

    for (j, ch) in number.chars().enumerate() {
        let x = start_x + j as f32 * char_spacing;
        let jitter_x = rng.gen::<f32>() * 6.0 - 3.0;
        let jitter_y = rng.gen::<f32>() * 10.0 - 5.0;
        let rotation = rng.gen::<f32>() * (10.0 + complexity * 25.0) - (5.0 + complexity * 12.5);
        let font_size = 36.0 + rng.gen::<f32>() * 12.0;
        let font = fonts[rng.gen_range(0..fonts.len())];
        let skew = rng.gen::<f32>() * complexity * 12.0 - complexity * 6.0;

        let bright = 170.0 + (1.0 - complexity * 0.3) * 60.0;
        let r = rng.gen_range(bright as u32..(bright as u32 + 30).min(255));
        let g = rng.gen_range(bright as u32..(bright as u32 + 30).min(255));
        let b = rng.gen_range(bright as u32..(bright as u32 + 30).min(255));

        let fx = x + jitter_x;
        let fy = cy + jitter_y;
        write!(
            svg,
            r#"<text x="{fx:.1}" y="{fy:.1}" font-family="{font}" font-size="{font_size:.0}" font-weight="bold" fill="rgb({r},{g},{b})" text-anchor="middle" dominant-baseline="central" transform="rotate({rotation:.1},{fx:.1},{fy:.1}) skewX({skew:.1})">{ch}</text>"#,
        )
        .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::rng_from_seed;

    fn default_difficulty() -> DifficultyParams {
        DifficultyParams {
            level: 10,
            round_number: 3,
            time_limit_ms: 6000,
            complexity: 0.5,
            noise: 0.3,
        }
    }

    #[test]
    fn test_cascade_determinism() {
        let difficulty = default_difficulty();
        let gen = CascadeGenerator;

        let mut rng1 = rng_from_seed(42);
        let inst1 = gen.generate(&mut rng1, &difficulty);

        let mut rng2 = rng_from_seed(42);
        let inst2 = gen.generate(&mut rng2, &difficulty);

        if let (Solution::Text(a), Solution::Text(b)) = (&inst1.solution, &inst2.solution) {
            assert_eq!(a, b);
        } else {
            panic!("Expected Text solution");
        }
    }

    #[test]
    fn test_cascade_valid_instance() {
        let mut rng = rng_from_seed(99);
        let difficulty = default_difficulty();
        let gen = CascadeGenerator;
        let instance = gen.generate(&mut rng, &difficulty);

        assert_eq!(instance.captcha_type, CaptchaType::TimePressureCascade);
        if let RenderPayload::Svg(ref s) = instance.render_data {
            assert!(s.contains("<svg"));
            assert!(s.contains("#1a1a2e"));
            // Should have multiple challenge sections
            assert!(s.matches("#1").count() >= 1);
            assert!(s.matches("#2").count() >= 1);
        } else {
            panic!("Expected SVG render data");
        }

        if let Solution::Text(ref sol) = instance.solution {
            // Solution should have pipe-separated numbers
            assert!(sol.contains('|'));
            let parts: Vec<&str> = sol.split('|').collect();
            assert!(parts.len() >= 4);
            // Each part should be a valid number
            for part in &parts {
                assert!(part.parse::<u32>().is_ok());
            }
        } else {
            panic!("Expected Text solution");
        }
    }

    #[test]
    fn test_cascade_correct_validates() {
        let mut rng = rng_from_seed(55);
        let difficulty = default_difficulty();
        let gen = CascadeGenerator;
        let instance = gen.generate(&mut rng, &difficulty);

        if let Solution::Text(ref answer) = instance.solution {
            assert!(gen.validate(&instance, &PlayerAnswer::Text(answer.clone())));
        } else {
            panic!("Expected Text solution");
        }
    }

    #[test]
    fn test_cascade_wrong_rejects() {
        let mut rng = rng_from_seed(55);
        let difficulty = default_difficulty();
        let gen = CascadeGenerator;
        let instance = gen.generate(&mut rng, &difficulty);

        assert!(!gen.validate(&instance, &PlayerAnswer::Text("1|2|3|4|5".to_string())));
        assert!(!gen.validate(&instance, &PlayerAnswer::Text(String::new())));
        // Wrong answer type
        assert!(!gen.validate(&instance, &PlayerAnswer::Number(42.0)));
    }

    #[test]
    fn test_cascade_difficulty_scaling() {
        let gen = CascadeGenerator;

        let low = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 4000,
            complexity: 0.0,
            noise: 0.0,
        };
        let high = DifficultyParams {
            level: 50,
            round_number: 10,
            time_limit_ms: 8000,
            complexity: 1.0,
            noise: 1.0,
        };

        let mut rng_low = rng_from_seed(42);
        let inst_low = gen.generate(&mut rng_low, &low);
        let mut rng_high = rng_from_seed(42);
        let inst_high = gen.generate(&mut rng_high, &high);

        // High difficulty should have more challenges (more pipe separators)
        if let (Solution::Text(ref sol_low), Solution::Text(ref sol_high)) =
            (&inst_low.solution, &inst_high.solution)
        {
            let count_low = sol_low.split('|').count();
            let count_high = sol_high.split('|').count();
            assert!(count_high >= count_low);
        }
    }
}
