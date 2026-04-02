use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct TypographyGenerator;

/// Characters that can be used in adversarial text
const BASE_CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ";

/// Pairs of characters that look similar and can be substituted
/// Each entry: (display_chars, decoded_char)
/// e.g. displaying "rn" should be read as "m"
const VISUAL_SUBS: &[(&str, char)] = &[
    ("rn", 'M'),
    ("cl", 'D'),
    ("vv", 'W'),
    ("li", 'U'),
    ("nn", 'M'),
];

/// Characters that look similar to each other for confusion
const CONFUSABLE_PAIRS: &[(char, char)] = &[
    ('O', '0'),
    ('I', '1'),
    ('S', '5'),
    ('B', '8'),
    ('Z', '2'),
    ('G', '6'),
];

impl CaptchaGenerator for TypographyGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        // Word length: 4-7 based on complexity
        let word_len = 4 + (difficulty.complexity * 3.0) as usize;
        let word_len = word_len.min(7);

        // Generate the "true" word that the player must type
        let word: String = (0..word_len)
            .map(|_| BASE_CHARS[rng.gen_range(0..BASE_CHARS.len())] as char)
            .collect();

        let svg = render_adversarial_svg(rng, &word, difficulty);

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::Text(word),
            expected_solve_time_ms: expected_solve_time(difficulty.time_limit_ms),
            point_value: CaptchaType::AdversarialTypography.base_points(),
            captcha_type: CaptchaType::AdversarialTypography,
            time_limit_ms: difficulty.time_limit_ms,
        }
    }

    fn validate(&self, instance: &CaptchaInstance, answer: &PlayerAnswer) -> bool {
        instance.validate(answer)
    }
}

/// A single rendered "glyph" — may be a real character or a visual substitution fragment
struct Glyph {
    text: String,
    font_family: &'static str,
    font_size: f32,
    font_weight: &'static str,
    /// Extra kerning offset (can be very negative for overlap or very positive for gaps)
    kerning_offset: f32,
    rotation: f32,
    skew_x: f32,
    /// Vertical jitter
    dy: f32,
    /// Color
    r: u8,
    g: u8,
    b: u8,
    /// Whether parts of the stroke are "removed" (simulated via partial opacity)
    stroke_removal: bool,
    /// Extra strokes added to confuse
    extra_strokes: bool,
}

fn render_adversarial_svg(
    rng: &mut ChaCha8Rng,
    word: &str,
    difficulty: &DifficultyParams,
) -> String {
    let width = 450;
    let height = 160;
    let c = difficulty.complexity;

    let mut svg = String::with_capacity(8192);
    write!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}">"#
    )
    .unwrap();

    write!(
        svg,
        r##"<rect width="{width}" height="{height}" fill="#1a1a2e"/>"##
    )
    .unwrap();

    // Prompt
    write!(
        svg,
        r##"<text x="{}" y="16" font-family="sans-serif" font-size="11" fill="#888" text-anchor="middle">Type exactly what you see</text>"##,
        width / 2
    )
    .unwrap();

    let fonts = [
        "monospace",
        "serif",
        "sans-serif",
        "Georgia, serif",
        "Courier, monospace",
    ];
    let weights = ["normal", "bold", "900", "300"];

    // Build glyphs — each character gets adversarial treatment
    let mut glyphs: Vec<Glyph> = Vec::new();

    for ch in word.chars() {
        // Decide on visual substitution at higher difficulty
        let use_visual_sub = c > 0.4 && rng.gen::<f32>() < c * 0.3;
        let display_text = if use_visual_sub {
            // Find a substitution where decoded_char matches this character
            let matching_subs: Vec<&(&str, char)> = VISUAL_SUBS
                .iter()
                .filter(|(_, decoded)| *decoded == ch)
                .collect();
            if let Some(sub) = matching_subs.first() {
                sub.0.to_string()
            } else {
                ch.to_string()
            }
        } else {
            ch.to_string()
        };

        // Extreme kerning: negative = overlapping, positive = big gap
        let kerning_range = 8.0 + c * 25.0;
        let kerning_offset = rng.gen::<f32>() * kerning_range - kerning_range * 0.4;

        // Mixed font sizes — dramatic variation
        let base_size = 28.0;
        let size_variation = c * 20.0;
        let font_size = base_size + rng.gen::<f32>() * size_variation - size_variation * 0.3;

        // Different font per character
        let font_family = fonts[rng.gen_range(0..fonts.len())];
        let font_weight = weights[rng.gen_range(0..weights.len())];

        // Rotation — more extreme at high difficulty
        let rot_range = 15.0 + c * 30.0;
        let rotation = rng.gen::<f32>() * rot_range - rot_range / 2.0;

        // Skew
        let skew_x = rng.gen::<f32>() * c * 20.0 - c * 10.0;

        // Vertical jitter
        let dy = rng.gen::<f32>() * (10.0 + c * 20.0) - (5.0 + c * 10.0);

        // Color — varied per character, sometimes very similar to background
        let bright_base = 130.0 + (1.0 - c * 0.5) * 80.0;
        let bright_range = 60.0;
        let r = rng.gen_range(bright_base as u8..(bright_base as u8).saturating_add(bright_range as u8));
        let g = rng.gen_range(bright_base as u8..(bright_base as u8).saturating_add(bright_range as u8));
        let b = rng.gen_range(bright_base as u8..(bright_base as u8).saturating_add(bright_range as u8));

        // Strategic stroke removal (at high difficulty)
        let stroke_removal = c > 0.5 && rng.gen::<f32>() < c * 0.25;

        // Extra strokes (at high difficulty)
        let extra_strokes = c > 0.3 && rng.gen::<f32>() < c * 0.3;

        glyphs.push(Glyph {
            text: display_text,
            font_family,
            font_size,
            font_weight,
            kerning_offset,
            rotation,
            skew_x,
            dy,
            r,
            g,
            b,
            stroke_removal,
            extra_strokes,
        });
    }

    // --- Background noise: confusable character ghosts ---
    let ghost_count = (3.0 + c * 12.0) as u32;
    for _ in 0..ghost_count {
        let pair = CONFUSABLE_PAIRS[rng.gen_range(0..CONFUSABLE_PAIRS.len())];
        let ghost_char = if rng.gen_bool(0.5) { pair.0 } else { pair.1 };
        let gx = rng.gen_range(20..width - 20) as f32;
        let gy = rng.gen_range(30..height - 15) as f32;
        let gsize = 18.0 + rng.gen::<f32>() * 24.0;
        let font = fonts[rng.gen_range(0..fonts.len())];
        let rot = rng.gen::<f32>() * 60.0 - 30.0;
        let r = rng.gen_range(60..160);
        let g = rng.gen_range(60..160);
        let b = rng.gen_range(60..160);
        let opacity = 0.08 + rng.gen::<f32>() * (0.12 + c * 0.15);
        write!(
            svg,
            r#"<text x="{gx:.0}" y="{gy:.0}" font-family="{font}" font-size="{gsize:.0}" fill="rgb({r},{g},{b})" text-anchor="middle" dominant-baseline="central" transform="rotate({rot:.1},{gx:.0},{gy:.0})" opacity="{opacity:.2}">{ghost_char}</text>"#
        )
        .unwrap();
    }

    // --- Thick crossing lines that break OCR segmentation ---
    let line_count = 3 + (c * 6.0) as u32;
    let mid_y = height as f32 / 2.0;
    for _ in 0..line_count {
        let x1 = rng.gen_range(0..width / 3);
        let y1 = mid_y + rng.gen_range(-30.0..30.0_f32);
        let x2 = rng.gen_range(width * 2 / 3..width);
        let y2 = mid_y + rng.gen_range(-30.0..30.0_f32);
        let sw = 2.0 + rng.gen::<f32>() * (2.0 + c * 3.0);
        let r = rng.gen_range(100..210);
        let g = rng.gen_range(100..210);
        let b = rng.gen_range(100..210);
        let opacity = 0.3 + c * 0.3;
        write!(
            svg,
            r#"<line x1="{x1}" y1="{y1:.0}" x2="{x2}" y2="{y2:.0}" stroke="rgb({r},{g},{b})" stroke-width="{sw:.1}" opacity="{opacity:.2}"/>"#
        )
        .unwrap();
    }

    // --- Render the actual glyphs ---
    let total_advance: f32 = glyphs.iter().map(|g| g.font_size * 0.6 + g.kerning_offset).sum();
    let start_x = (width as f32 - total_advance) / 2.0;
    let base_y = height as f32 / 2.0 + 5.0;

    let mut cursor_x = start_x;
    for glyph in &glyphs {
        let x = cursor_x;
        let y = base_y + glyph.dy;

        // Extra strokes: random line segments near the character
        if glyph.extra_strokes {
            let sx1 = x + rng.gen::<f32>() * 15.0 - 7.0;
            let sy1 = y + rng.gen::<f32>() * 15.0 - 12.0;
            let sx2 = sx1 + rng.gen::<f32>() * 12.0 - 6.0;
            let sy2 = sy1 + rng.gen::<f32>() * 12.0 - 6.0;
            write!(
                svg,
                r#"<line x1="{sx1:.1}" y1="{sy1:.1}" x2="{sx2:.1}" y2="{sy2:.1}" stroke="rgb({},{},{})" stroke-width="2" opacity="0.7"/>"#,
                glyph.r, glyph.g, glyph.b
            )
            .unwrap();
        }

        let opacity = if glyph.stroke_removal {
            // Simulate stroke removal by reducing opacity in part
            0.5 + rng.gen::<f32>() * 0.2
        } else {
            1.0
        };

        // Escape special XML characters
        let escaped: String = glyph
            .text
            .chars()
            .map(|c| match c {
                '<' => "&lt;".to_string(),
                '>' => "&gt;".to_string(),
                '&' => "&amp;".to_string(),
                _ => c.to_string(),
            })
            .collect();

        write!(
            svg,
            r#"<text x="{x:.1}" y="{y:.1}" font-family="{}" font-size="{:.0}" font-weight="{}" fill="rgb({},{},{})" text-anchor="start" dominant-baseline="central" transform="rotate({:.1},{x:.1},{y:.1}) skewX({:.1})" opacity="{opacity:.2}">{escaped}</text>"#,
            glyph.font_family,
            glyph.font_size,
            glyph.font_weight,
            glyph.r, glyph.g, glyph.b,
            glyph.rotation,
            glyph.skew_x
        )
        .unwrap();

        // Stroke removal overlay: a small rect matching background color over part of the character
        if glyph.stroke_removal {
            let rx = x + rng.gen::<f32>() * glyph.font_size * 0.3;
            let ry = y - glyph.font_size * 0.2 + rng.gen::<f32>() * glyph.font_size * 0.3;
            let rw = glyph.font_size * 0.25;
            let rh = glyph.font_size * 0.15;
            write!(
                svg,
                r##"<rect x="{rx:.1}" y="{ry:.1}" width="{rw:.1}" height="{rh:.1}" fill="#1a1a2e" opacity="0.7"/>"##
            )
            .unwrap();
        }

        cursor_x += glyph.font_size * 0.6 + glyph.kerning_offset;
    }

    // --- Overlay: more noise on top ---
    let overlay_count = 2 + (c * 5.0) as u32;
    for _ in 0..overlay_count {
        let x1 = rng.gen_range(0..width) as f32;
        let y1 = mid_y + rng.gen_range(-35.0..35.0_f32);
        let cx1 = rng.gen_range(0..width) as f32;
        let cy1 = rng.gen_range(20..height - 20) as f32;
        let x2 = rng.gen_range(0..width) as f32;
        let y2 = mid_y + rng.gen_range(-35.0..35.0_f32);
        let sw = 1.5 + rng.gen::<f32>() * (2.0 + c * 2.5);
        let r = rng.gen_range(100..220);
        let g = rng.gen_range(100..220);
        let b = rng.gen_range(100..220);
        write!(
            svg,
            r#"<path d="M{x1:.0},{y1:.0} Q{cx1:.0},{cy1:.0} {x2:.0},{y2:.0}" stroke="rgb({r},{g},{b})" stroke-width="{sw:.1}" fill="none" opacity="{:.2}"/>"#,
            0.2 + c * 0.3
        )
        .unwrap();
    }

    // --- Noise dots ---
    let dot_count = 15 + (c * 40.0) as u32;
    for _ in 0..dot_count {
        let dx = rng.gen_range(0..width);
        let dy = rng.gen_range(0..height);
        let dr = 1.0 + rng.gen::<f32>() * (1.5 + c * 3.0);
        let r = rng.gen_range(60..200);
        let g = rng.gen_range(60..200);
        let b = rng.gen_range(60..200);
        write!(
            svg,
            r#"<circle cx="{dx}" cy="{dy}" r="{dr:.1}" fill="rgb({r},{g},{b})" opacity="{:.2}"/>"#,
            0.15 + rng.gen::<f32>() * 0.3
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
    fn test_typography_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 8000,
            complexity: 0.0,
            noise: 0.0,
        };
        let gen = TypographyGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert_eq!(instance.captcha_type, CaptchaType::AdversarialTypography);
        if let Solution::Text(ref t) = instance.solution {
            assert!(t.len() >= 4);
            // All characters should be from BASE_CHARS
            for ch in t.chars() {
                assert!(BASE_CHARS.contains(&(ch as u8)));
            }
        } else {
            panic!("Expected Text solution");
        }
        if let RenderPayload::Svg(ref s) = instance.render_data {
            assert!(s.contains("<svg"));
            assert!(s.contains("Type exactly what you see"));
        } else {
            panic!("Expected SVG render data");
        }
    }

    #[test]
    fn test_typography_seed_determinism() {
        let difficulty = DifficultyParams {
            level: 10,
            round_number: 3,
            time_limit_ms: 10000,
            complexity: 0.5,
            noise: 0.3,
        };
        let gen = TypographyGenerator;

        let mut rng1 = rng_from_seed(12345);
        let inst1 = gen.generate(&mut rng1, &difficulty);

        let mut rng2 = rng_from_seed(12345);
        let inst2 = gen.generate(&mut rng2, &difficulty);

        if let (Solution::Text(t1), Solution::Text(t2)) = (&inst1.solution, &inst2.solution) {
            assert_eq!(t1, t2);
        }
    }

    #[test]
    fn test_typography_correct_answer_validates() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 8000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = TypographyGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::Text(ref answer) = instance.solution {
            assert!(gen.validate(&instance, &PlayerAnswer::Text(answer.clone())));
            // Case insensitive
            assert!(gen.validate(&instance, &PlayerAnswer::Text(answer.to_lowercase())));
        }
    }

    #[test]
    fn test_typography_wrong_answer_rejects() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 8000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = TypographyGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert!(!gen.validate(&instance, &PlayerAnswer::Text("WRONGANSWER".to_string())));
        assert!(!gen.validate(&instance, &PlayerAnswer::Text(String::new())));
    }

    #[test]
    fn test_typography_difficulty_scaling() {
        let gen = TypographyGenerator;
        let low = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 8000,
            complexity: 0.0,
            noise: 0.0,
        };
        let high = DifficultyParams {
            level: 50,
            round_number: 10,
            time_limit_ms: 15000,
            complexity: 1.0,
            noise: 1.0,
        };

        let mut rng1 = rng_from_seed(42);
        let inst_low = gen.generate(&mut rng1, &low);
        let mut rng2 = rng_from_seed(42);
        let inst_high = gen.generate(&mut rng2, &high);

        // Higher complexity = longer word
        if let (Solution::Text(t_low), Solution::Text(t_high)) =
            (&inst_low.solution, &inst_high.solution)
        {
            assert!(t_high.len() >= t_low.len());
        }

        // Higher complexity = more noise = longer SVG
        if let (RenderPayload::Svg(svg_low), RenderPayload::Svg(svg_high)) =
            (&inst_low.render_data, &inst_high.render_data)
        {
            assert!(svg_high.len() > svg_low.len());
        }
    }
}
