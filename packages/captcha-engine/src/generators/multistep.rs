use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct MultiStepGenerator;

/// The types of micro-challenges available
#[derive(Debug, Clone, Copy)]
enum MicroChallenge {
    MiniText,
    ShapeIdentify,
    MiniMath,
}

const CHALLENGES: [MicroChallenge; 3] = [
    MicroChallenge::MiniText,
    MicroChallenge::ShapeIdentify,
    MicroChallenge::MiniMath,
];

const TEXT_CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ";

/// Result of generating a single micro-challenge
struct MicroResult {
    svg_fragment: String,
    answer: String,
    height: f32,
}

fn generate_mini_text(
    rng: &mut ChaCha8Rng,
    y_offset: f32,
    width: f32,
    complexity: f32,
) -> MicroResult {
    let char_count = 3 + (complexity * 2.0) as usize;
    let char_count = char_count.min(5);
    let text: String = (0..char_count)
        .map(|_| TEXT_CHARS[rng.gen_range(0..TEXT_CHARS.len())] as char)
        .collect();

    let section_height = 80.0;
    let mut svg = String::new();

    write!(
        svg,
        r##"<text x="10" y="{:.0}" font-family="sans-serif" font-size="11" fill="#aaa">Type the distorted text:</text>"##,
        y_offset + 15.0
    )
    .unwrap();

    write!(
        svg,
        r##"<rect x="5" y="{:.0}" width="{:.0}" height="55" fill="#12121f" rx="3"/>"##,
        y_offset + 20.0,
        width - 10.0
    )
    .unwrap();

    let curve_count = 2 + (complexity * 4.0) as u32;
    for _ in 0..curve_count {
        let x1 = rng.gen_range(10.0..width - 10.0);
        let y1 = y_offset + 25.0 + rng.gen::<f32>() * 45.0;
        let cx = rng.gen_range(10.0..width - 10.0);
        let cy = y_offset + 25.0 + rng.gen::<f32>() * 45.0;
        let x2 = rng.gen_range(10.0..width - 10.0);
        let y2 = y_offset + 25.0 + rng.gen::<f32>() * 45.0;
        let r = rng.gen_range(100..200);
        let g = rng.gen_range(100..200);
        let b = rng.gen_range(100..200);
        write!(
            svg,
            r#"<path d="M{x1:.0},{y1:.0} Q{cx:.0},{cy:.0} {x2:.0},{y2:.0}" stroke="rgb({r},{g},{b})" stroke-width="1.5" fill="none" opacity="0.3"/>"#
        )
        .unwrap();
    }

    let char_spacing = (width - 40.0) / (char_count as f32 + 1.0);
    let fonts = ["monospace", "serif", "sans-serif"];
    let base_y = y_offset + 50.0;

    for (i, ch) in text.chars().enumerate() {
        let x = 20.0 + char_spacing * (i as f32 + 1.0);
        let jitter = rng.gen::<f32>() * 10.0 - 5.0;
        let rotation = rng.gen::<f32>() * 20.0 - 10.0;
        let font = fonts[rng.gen_range(0..fonts.len())];
        let r = rng.gen_range(170..240);
        let g = rng.gen_range(170..240);
        let b = rng.gen_range(170..240);
        let ty = base_y + jitter;
        write!(
            svg,
            r#"<text x="{x:.1}" y="{ty:.1}" font-family="{font}" font-size="22" font-weight="bold" fill="rgb({r},{g},{b})" text-anchor="middle" dominant-baseline="central" transform="rotate({rotation:.1},{x:.1},{ty:.1})">{ch}</text>"#,
        )
        .unwrap();
    }

    MicroResult {
        svg_fragment: svg,
        answer: text,
        height: section_height,
    }
}

/// Render a shape into SVG string at given center and return it
fn render_shape_svg(shape_idx: usize, cx: f32, cy: f32, fill: &str) -> String {
    let stroke = "stroke=\"rgb(85,85,85)\" stroke-width=\"1\"";
    match shape_idx {
        0 => format!(
            r#"<circle cx="{cx:.0}" cy="{cy:.0}" r="18" fill="{fill}" {stroke}/>"#,
        ),
        1 => format!(
            r#"<polygon points="{cx:.0},{:.0} {:.0},{:.0} {:.0},{:.0}" fill="{fill}" {stroke}/>"#,
            cy - 18.0,
            cx - 18.0,
            cy + 14.0,
            cx + 18.0,
            cy + 14.0
        ),
        2 => format!(
            r#"<rect x="{:.0}" y="{:.0}" width="32" height="32" fill="{fill}" {stroke}/>"#,
            cx - 16.0,
            cy - 16.0
        ),
        3 => format!(
            r#"<polygon points="{cx:.0},{:.0} {:.0},{cy:.0} {cx:.0},{:.0} {:.0},{cy:.0}" fill="{fill}" {stroke}/>"#,
            cy - 20.0,
            cx + 14.0,
            cy + 20.0,
            cx - 14.0
        ),
        _ => {
            let mut points = String::new();
            for i in 0..10 {
                let angle = std::f32::consts::PI * 2.0 * i as f32 / 10.0
                    - std::f32::consts::PI / 2.0;
                let r = if i % 2 == 0 { 18.0 } else { 8.0 };
                let px = cx + angle.cos() * r;
                let py = cy + angle.sin() * r;
                if !points.is_empty() {
                    points.push(' ');
                }
                write!(points, "{px:.1},{py:.1}").unwrap();
            }
            format!(
                r#"<polygon points="{points}" fill="{fill}" {stroke}/>"#,
            )
        }
    }
}

fn generate_shape_identify(
    rng: &mut ChaCha8Rng,
    y_offset: f32,
    width: f32,
    _complexity: f32,
) -> MicroResult {
    let shapes = ["circle", "triangle", "square", "diamond", "star"];
    let shape_idx = rng.gen_range(0..shapes.len());
    let shape_name = shapes[shape_idx];

    let section_height = 80.0;

    let cx = width / 2.0;
    let cy = y_offset + 48.0;
    let r_val = rng.gen_range(140..240);
    let g_val = rng.gen_range(140..240);
    let b_val = rng.gen_range(140..240);
    let fill = format!("rgb({r_val},{g_val},{b_val})");

    let rotation = rng.gen::<f32>() * 30.0 - 15.0;
    let shape_svg = render_shape_svg(shape_idx, cx, cy, &fill);

    let mut final_svg = String::new();
    write!(
        final_svg,
        r##"<text x="10" y="{:.0}" font-family="sans-serif" font-size="11" fill="#aaa">Name this shape:</text>"##,
        y_offset + 15.0
    )
    .unwrap();
    write!(
        final_svg,
        r##"<rect x="5" y="{:.0}" width="{:.0}" height="55" fill="#12121f" rx="3"/>"##,
        y_offset + 20.0,
        width - 10.0
    )
    .unwrap();
    write!(
        final_svg,
        r#"<g transform="rotate({rotation:.1},{cx:.0},{cy:.0})">{shape_svg}</g>"#
    )
    .unwrap();

    MicroResult {
        svg_fragment: final_svg,
        answer: shape_name.to_string(),
        height: section_height,
    }
}

fn generate_mini_math(
    rng: &mut ChaCha8Rng,
    y_offset: f32,
    width: f32,
    complexity: f32,
) -> MicroResult {
    let max_val = 10 + (complexity * 40.0) as i32;
    let a = rng.gen_range(1..=max_val);
    let b = rng.gen_range(1..=max_val);
    let ops = ['+', '-'];
    let op = ops[rng.gen_range(0..ops.len())];
    let answer = match op {
        '+' => a + b,
        '-' => a - b,
        _ => unreachable!(),
    };

    let expression = format!("{} {} {} = ?", a, op, b);

    let section_height = 80.0;
    let mut svg = String::new();

    write!(
        svg,
        r##"<text x="10" y="{:.0}" font-family="sans-serif" font-size="11" fill="#aaa">Solve:</text>"##,
        y_offset + 15.0
    )
    .unwrap();

    write!(
        svg,
        r##"<rect x="5" y="{:.0}" width="{:.0}" height="55" fill="#12121f" rx="3"/>"##,
        y_offset + 20.0,
        width - 10.0
    )
    .unwrap();

    let char_spacing = (width - 40.0) / (expression.len() as f32 + 1.0);
    let base_y = y_offset + 50.0;

    for (i, ch) in expression.chars().enumerate() {
        let x = 20.0 + char_spacing * (i as f32 + 1.0);
        let jitter = rng.gen::<f32>() * 6.0 - 3.0;
        let rotation = rng.gen::<f32>() * 8.0 - 4.0;
        let r = rng.gen_range(170..240);
        let g = rng.gen_range(170..240);
        let b_val = rng.gen_range(170..240);
        let escaped = match ch {
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '&' => "&amp;".to_string(),
            _ => ch.to_string(),
        };
        let ty = base_y + jitter;
        write!(
            svg,
            r#"<text x="{x:.1}" y="{ty:.1}" font-family="monospace" font-size="24" font-weight="bold" fill="rgb({r},{g},{b_val})" text-anchor="middle" dominant-baseline="central" transform="rotate({rotation:.1},{x:.1},{ty:.1})">{escaped}</text>"#,
        )
        .unwrap();
    }

    let dot_count = 3 + (complexity * 8.0) as u32;
    for _ in 0..dot_count {
        let dx = rng.gen_range(10.0..width - 10.0);
        let dy = y_offset + 22.0 + rng.gen::<f32>() * 50.0;
        let r = rng.gen_range(80..180);
        let g = rng.gen_range(80..180);
        let b = rng.gen_range(80..180);
        write!(
            svg,
            r#"<circle cx="{dx:.0}" cy="{dy:.0}" r="1.5" fill="rgb({r},{g},{b})" opacity="0.3"/>"#
        )
        .unwrap();
    }

    MicroResult {
        svg_fragment: svg,
        answer: answer.to_string(),
        height: section_height,
    }
}

impl CaptchaGenerator for MultiStepGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        let chain_len = if difficulty.complexity > 0.5 { 3 } else { 2 };

        let width = 400.0_f32;

        let mut challenge_pool: Vec<MicroChallenge> = CHALLENGES.to_vec();
        for i in (1..challenge_pool.len()).rev() {
            let j = rng.gen_range(0..=i);
            challenge_pool.swap(i, j);
        }

        let mut answers: Vec<String> = Vec::new();
        let mut fragments: Vec<String> = Vec::new();
        let mut y_cursor = 30.0_f32;

        for challenge in challenge_pool.iter().take(chain_len) {
            let result = match challenge {
                MicroChallenge::MiniText => {
                    generate_mini_text(rng, y_cursor, width, difficulty.complexity)
                }
                MicroChallenge::ShapeIdentify => {
                    generate_shape_identify(rng, y_cursor, width, difficulty.complexity)
                }
                MicroChallenge::MiniMath => {
                    generate_mini_math(rng, y_cursor, width, difficulty.complexity)
                }
            };

            answers.push(result.answer);
            fragments.push(result.svg_fragment);
            y_cursor += result.height + 5.0;
        }

        let total_height = y_cursor + 10.0;
        let combined_answer = answers.join("|");

        let mut svg = String::with_capacity(8192);
        write!(
            svg,
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width:.0} {total_height:.0}" width="{width:.0}" height="{total_height:.0}">"#
        )
        .unwrap();

        write!(
            svg,
            r##"<rect width="{width:.0}" height="{total_height:.0}" fill="#1a1a2e"/>"##
        )
        .unwrap();

        write!(
            svg,
            r##"<text x="{:.0}" y="18" font-family="sans-serif" font-size="13" fill="#cccccc" text-anchor="middle">Solve all challenges (separate answers with |)</text>"##,
            width / 2.0
        )
        .unwrap();

        for (idx, fragment) in fragments.iter().enumerate() {
            svg.push_str(fragment);
            if idx < fragments.len() - 1 {
                let sep_y = 30.0 + (idx + 1) as f32 * 85.0;
                write!(
                    svg,
                    r##"<line x1="20" y1="{sep_y:.0}" x2="{:.0}" y2="{sep_y:.0}" stroke="#333" stroke-width="1" stroke-dasharray="4,4"/>"##,
                    width - 20.0
                )
                .unwrap();
            }
        }

        svg.push_str("</svg>");

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::Text(combined_answer),
            expected_solve_time_ms: expected_solve_time(difficulty.time_limit_ms),
            point_value: CaptchaType::MultiStepVerification.base_points(),
            captcha_type: CaptchaType::MultiStepVerification,
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
    fn test_multistep_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 12000,
            complexity: 0.0,
            noise: 0.0,
        };
        let gen = MultiStepGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert_eq!(instance.captcha_type, CaptchaType::MultiStepVerification);
        if let Solution::Text(ref t) = instance.solution {
            assert!(t.contains('|'), "Answer should contain | separator: {}", t);
            let parts: Vec<&str> = t.split('|').collect();
            assert!(parts.len() >= 2);
        } else {
            panic!("Expected Text solution");
        }
        if let RenderPayload::Svg(ref s) = instance.render_data {
            assert!(s.contains("<svg"));
            assert!(s.contains("Solve all challenges"));
        } else {
            panic!("Expected SVG render data");
        }
    }

    #[test]
    fn test_multistep_seed_determinism() {
        let difficulty = DifficultyParams {
            level: 10,
            round_number: 3,
            time_limit_ms: 15000,
            complexity: 0.5,
            noise: 0.3,
        };
        let gen = MultiStepGenerator;

        let mut rng1 = rng_from_seed(12345);
        let inst1 = gen.generate(&mut rng1, &difficulty);

        let mut rng2 = rng_from_seed(12345);
        let inst2 = gen.generate(&mut rng2, &difficulty);

        if let (Solution::Text(t1), Solution::Text(t2)) = (&inst1.solution, &inst2.solution) {
            assert_eq!(t1, t2);
        }
    }

    #[test]
    fn test_multistep_correct_answer_validates() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 12000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = MultiStepGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::Text(ref answer) = instance.solution {
            assert!(gen.validate(&instance, &PlayerAnswer::Text(answer.clone())));
            assert!(gen.validate(&instance, &PlayerAnswer::Text(answer.to_lowercase())));
        }
    }

    #[test]
    fn test_multistep_wrong_answer_rejects() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 12000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = MultiStepGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert!(!gen.validate(
            &instance,
            &PlayerAnswer::Text("WRONG|WRONG|WRONG".to_string())
        ));
        assert!(!gen.validate(&instance, &PlayerAnswer::Text(String::new())));
    }

    #[test]
    fn test_multistep_difficulty_scaling() {
        let gen = MultiStepGenerator;
        let low = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 12000,
            complexity: 0.0,
            noise: 0.0,
        };
        let high = DifficultyParams {
            level: 50,
            round_number: 10,
            time_limit_ms: 20000,
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
            let parts_low = t_low.split('|').count();
            let parts_high = t_high.split('|').count();
            assert!(parts_high >= parts_low);
        }
    }
}
