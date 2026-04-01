use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct TextGenerator;

const CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

impl CaptchaGenerator for TextGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        // Character count scales from 4 to 8 based on complexity
        let char_count = 4 + (difficulty.complexity * 4.0) as usize;
        let char_count = char_count.min(8);

        // Generate random text
        let text: String = (0..char_count)
            .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
            .collect();

        // Generate SVG with distortion
        let svg = generate_distorted_text_svg(rng, &text, difficulty);

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::Text(text),
            expected_solve_time_ms: expected_solve_time(difficulty.time_limit_ms),
            point_value: CaptchaType::DistortedText.base_points(),
            captcha_type: CaptchaType::DistortedText,
            time_limit_ms: difficulty.time_limit_ms,
        }
    }

    fn validate(&self, instance: &CaptchaInstance, answer: &PlayerAnswer) -> bool {
        instance.validate(answer)
    }
}

fn generate_distorted_text_svg(
    rng: &mut ChaCha8Rng,
    text: &str,
    difficulty: &DifficultyParams,
) -> String {
    let width = 400;
    let height = 150;
    let warp_amplitude = 10.0 + difficulty.complexity * 25.0;
    let c = difficulty.complexity;

    let mut svg = String::with_capacity(4096);
    write!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}">"#
    )
    .unwrap();

    // Background with subtle gradient noise
    write!(
        svg,
        r##"<rect width="{width}" height="{height}" fill="#1a1a2e"/>"##
    )
    .unwrap();

    // --- Layer 1: thick bezier curves that cross through the text area ---
    let curve_count = 5 + (c * 8.0) as u32;
    for _ in 0..curve_count {
        let x1 = rng.gen_range(-20..width + 20);
        let y1 = rng.gen_range(20..height - 20);
        let cx1 = rng.gen_range(0..width);
        let cy1 = rng.gen_range(0..height);
        let cx2 = rng.gen_range(0..width);
        let cy2 = rng.gen_range(0..height);
        let x2 = rng.gen_range(-20..width + 20);
        let y2 = rng.gen_range(20..height - 20);
        let sw = 2.0 + rng.gen::<f32>() * (2.0 + c * 3.0);
        let r = rng.gen_range(120..230);
        let g = rng.gen_range(120..230);
        let b = rng.gen_range(120..230);
        let opacity = 0.4 + rng.gen::<f32>() * 0.35;
        write!(
            svg,
            r#"<path d="M{x1},{y1} C{cx1},{cy1} {cx2},{cy2} {x2},{y2}" stroke="rgb({r},{g},{b})" stroke-width="{sw:.1}" fill="none" opacity="{opacity:.2}"/>"#
        )
        .unwrap();
    }

    // --- Layer 2: strikethrough lines deliberately crossing the text band ---
    let strike_count = 3 + (c * 5.0) as u32;
    let text_band_y = height as f32 / 2.0;
    for _ in 0..strike_count {
        let x1 = rng.gen_range(0..width / 4);
        let y1 = text_band_y + rng.gen_range(-20.0..20.0_f32);
        let x2 = rng.gen_range(width * 3 / 4..width);
        let y2 = text_band_y + rng.gen_range(-20.0..20.0_f32);
        let sw = 2.0 + rng.gen::<f32>() * (1.5 + c * 2.5);
        let r = rng.gen_range(140..240);
        let g = rng.gen_range(140..240);
        let b = rng.gen_range(140..240);
        let opacity = 0.45 + c * 0.3;
        write!(
            svg,
            r#"<line x1="{x1}" y1="{y1:.0}" x2="{x2}" y2="{y2:.0}" stroke="rgb({r},{g},{b})" stroke-width="{sw:.1}" opacity="{opacity:.2}"/>"#
        )
        .unwrap();
    }

    // --- Layer 3: decoy characters scattered around ---
    let decoy_count = (4.0 + c * 10.0) as u32;
    let fonts = ["monospace", "serif", "sans-serif"];
    for _ in 0..decoy_count {
        let ch = CHARS[rng.gen_range(0..CHARS.len())] as char;
        let dx = rng.gen_range(20..width - 20) as f32;
        // Bias decoys toward the text band (center ± 40px) so they overlap real chars
        let dy = (height as f32 / 2.0) + rng.gen_range(-40.0..40.0_f32);
        let font = fonts[rng.gen_range(0..fonts.len())];
        let size = 22.0 + rng.gen::<f32>() * 22.0;
        let rot = rng.gen::<f32>() * 60.0 - 30.0; // less wild rotation so they look like real chars
        // Decoy colors overlap with real character colors for confusion
        let r = rng.gen_range(100..220);
        let g = rng.gen_range(100..220);
        let b = rng.gen_range(100..220);
        let opacity = 0.2 + rng.gen::<f32>() * (0.2 + c * 0.25);
        write!(
            svg,
            r#"<text x="{dx:.0}" y="{dy:.0}" font-family="{font}" font-size="{size:.0}" font-weight="bold" fill="rgb({r},{g},{b})" text-anchor="middle" dominant-baseline="central" transform="rotate({rot:.0},{dx:.0},{dy:.0})" opacity="{opacity:.2}">{ch}</text>"#
        )
        .unwrap();
    }

    // --- Layer 4: actual characters with distortion ---
    let char_spacing = width as f32 / (text.len() as f32 + 1.0);

    // Generate a wavy baseline using a sine-like displacement
    let wave_freq = 0.5 + rng.gen::<f32>() * 1.5;
    let wave_phase = rng.gen::<f32>() * std::f32::consts::PI * 2.0;

    for (i, ch) in text.chars().enumerate() {
        let base_x = char_spacing * (i as f32 + 1.0);
        let base_y = height as f32 / 2.0;
        // Wavy baseline
        let wave_y = (wave_freq * i as f32 + wave_phase).sin() * warp_amplitude * 0.6;
        let jitter_y = rng.gen::<f32>() * warp_amplitude * 0.8 - warp_amplitude * 0.4;
        let offset_y = wave_y + jitter_y;
        let rotation = rng.gen::<f32>() * warp_amplitude * 1.5 - warp_amplitude * 0.75;
        let font_size = 34.0 + rng.gen::<f32>() * 14.0;
        let font = fonts[rng.gen_range(0..fonts.len())];
        // Skew for extra distortion
        let skew_x = rng.gen::<f32>() * c * 15.0 - c * 7.5;

        // Character color — not too bright, blends more with noise at high difficulty
        let bright = 160.0 + (1.0 - c * 0.3) * 80.0;
        let r = rng.gen_range(bright as u32..(bright as u32 + 40).min(255));
        let g = rng.gen_range(bright as u32..(bright as u32 + 40).min(255));
        let b = rng.gen_range(bright as u32..(bright as u32 + 40).min(255));

        let ty = base_y + offset_y;
        write!(
            svg,
            r#"<text x="{base_x:.1}" y="{ty:.1}" font-family="{font}" font-size="{font_size:.0}" font-weight="bold" fill="rgb({r},{g},{b})" text-anchor="middle" dominant-baseline="central" transform="rotate({rotation:.1},{base_x:.1},{ty:.1}) skewX({skew_x:.1})">{ch}</text>"#,
        )
        .unwrap();
    }

    // --- Layer 5: overlay curves on top of text for occlusion ---
    let overlay_count = 2 + (c * 4.0) as u32;
    for _ in 0..overlay_count {
        let x1 = rng.gen_range(0..width);
        let y1 = text_band_y + rng.gen_range(-25.0..25.0_f32);
        let cx1 = rng.gen_range(0..width);
        let cy1 = rng.gen_range(30..height - 30);
        let x2 = rng.gen_range(0..width);
        let y2 = text_band_y + rng.gen_range(-25.0..25.0_f32);
        let sw = 2.5 + rng.gen::<f32>() * (1.5 + c * 2.0);
        let r = rng.gen_range(150..240);
        let g = rng.gen_range(150..240);
        let b = rng.gen_range(150..240);
        let opacity = 0.35 + c * 0.35;
        write!(
            svg,
            r#"<path d="M{x1},{y1:.0} Q{cx1},{cy1} {x2},{y2:.0}" stroke="rgb({r},{g},{b})" stroke-width="{sw:.1}" fill="none" opacity="{opacity:.2}"/>"#
        )
        .unwrap();
    }

    // --- Layer 6: noise dots (larger, more visible) ---
    let dot_count = 20 + (c * 50.0) as u32;
    for _ in 0..dot_count {
        let cx = rng.gen_range(0..width);
        let cy = rng.gen_range(0..height);
        let r_val = 1.0 + rng.gen::<f32>() * (2.0 + c * 4.0);
        let r = rng.gen_range(80..220);
        let g = rng.gen_range(80..220);
        let b = rng.gen_range(80..220);
        let opacity = 0.2 + rng.gen::<f32>() * 0.4;
        write!(
            svg,
            r#"<circle cx="{cx}" cy="{cy}" r="{r_val:.1}" fill="rgb({r},{g},{b})" opacity="{opacity:.2}"/>"#
        )
        .unwrap();
    }

    svg.push_str("</svg>");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::rng_from_seed;

    #[test]
    fn test_text_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.0,
            noise: 0.0,
        };
        let gen = TextGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert_eq!(instance.captcha_type, CaptchaType::DistortedText);
        if let Solution::Text(ref t) = instance.solution {
            assert!(t.len() >= 4);
        } else {
            panic!("Expected Text solution");
        }
        if let RenderPayload::Svg(ref s) = instance.render_data {
            assert!(s.contains("<svg"));
        } else {
            panic!("Expected SVG render data");
        }
    }

    #[test]
    fn test_text_seed_determinism() {
        let difficulty = DifficultyParams {
            level: 10,
            round_number: 3,
            time_limit_ms: 7000,
            complexity: 0.5,
            noise: 0.3,
        };
        let gen = TextGenerator;

        let mut rng1 = rng_from_seed(12345);
        let inst1 = gen.generate(&mut rng1, &difficulty);

        let mut rng2 = rng_from_seed(12345);
        let inst2 = gen.generate(&mut rng2, &difficulty);

        if let (Solution::Text(t1), Solution::Text(t2)) = (&inst1.solution, &inst2.solution) {
            assert_eq!(t1, t2);
        }
    }

    #[test]
    fn test_text_correct_answer_validates() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = TextGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::Text(ref answer) = instance.solution {
            assert!(gen.validate(&instance, &PlayerAnswer::Text(answer.clone())));
            // Case insensitive
            assert!(gen.validate(&instance, &PlayerAnswer::Text(answer.to_lowercase())));
        }
    }

    #[test]
    fn test_text_wrong_answer_rejects() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = TextGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert!(!gen.validate(&instance, &PlayerAnswer::Text("WRONG".to_string())));
        assert!(!gen.validate(&instance, &PlayerAnswer::Text(String::new())));
    }

    #[test]
    fn test_text_difficulty_scaling() {
        let gen = TextGenerator;
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

        if let (Solution::Text(t_low), Solution::Text(t_high)) =
            (&inst_low.solution, &inst_high.solution)
        {
            assert!(t_high.len() >= t_low.len());
        }
    }
}
