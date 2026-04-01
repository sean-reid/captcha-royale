use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct MathGenerator;

#[derive(Debug, Clone, Copy)]
enum MathOp {
    Add,
    Sub,
    Mul,
}

impl MathOp {
    fn symbol(&self) -> char {
        match self {
            MathOp::Add => '+',
            MathOp::Sub => '-',
            MathOp::Mul => '×',
        }
    }
}

impl CaptchaGenerator for MathGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        // Operator count scales from 1 to 3
        let op_count = 1 + (difficulty.complexity * 2.0) as u32;
        let op_count = op_count.min(3);

        // Operand range scales with complexity
        let max_operand = 10 + (difficulty.complexity * 90.0) as i32;
        let use_parens = difficulty.complexity > 0.5 && op_count > 1;

        let (expression, answer) = generate_expression(rng, op_count, max_operand, use_parens);
        let svg = render_math_svg(rng, &expression, difficulty);

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::Number(answer),
            expected_solve_time_ms: expected_solve_time(difficulty.time_limit_ms),
            point_value: CaptchaType::SimpleMath.base_points(),
            captcha_type: CaptchaType::SimpleMath,
            time_limit_ms: difficulty.time_limit_ms,
        }
    }

    fn validate(&self, instance: &CaptchaInstance, answer: &PlayerAnswer) -> bool {
        instance.validate(answer)
    }
}

fn random_op(rng: &mut ChaCha8Rng, allow_mul: bool) -> MathOp {
    if allow_mul {
        match rng.gen_range(0..3) {
            0 => MathOp::Add,
            1 => MathOp::Sub,
            _ => MathOp::Mul,
        }
    } else {
        match rng.gen_range(0..2) {
            0 => MathOp::Add,
            _ => MathOp::Sub,
        }
    }
}

fn generate_expression(
    rng: &mut ChaCha8Rng,
    op_count: u32,
    max_operand: i32,
    use_parens: bool,
) -> (String, f64) {
    if op_count == 1 {
        let a = rng.gen_range(1..=max_operand);
        let op = random_op(rng, max_operand <= 20);
        let b = match op {
            MathOp::Mul => rng.gen_range(1..=12.min(max_operand)),
            _ => rng.gen_range(1..=max_operand),
        };
        let result = match op {
            MathOp::Add => a as f64 + b as f64,
            MathOp::Sub => a as f64 - b as f64,
            MathOp::Mul => a as f64 * b as f64,
        };
        (format!("{} {} {}", a, op.symbol(), b), result)
    } else if op_count == 2 {
        let a = rng.gen_range(1..=max_operand);
        let b = rng.gen_range(1..=max_operand.min(30));
        let c = rng.gen_range(1..=max_operand.min(30));
        let op1 = random_op(rng, false);
        let op2 = random_op(rng, false);

        if use_parens && rng.gen_bool(0.5) {
            // (a op b) op c
            let inner = match op1 {
                MathOp::Add => a as f64 + b as f64,
                MathOp::Sub => a as f64 - b as f64,
                MathOp::Mul => a as f64 * b as f64,
            };
            let result = match op2 {
                MathOp::Add => inner + c as f64,
                MathOp::Sub => inner - c as f64,
                MathOp::Mul => inner * c as f64,
            };
            (
                format!("({} {} {}) {} {}", a, op1.symbol(), b, op2.symbol(), c),
                result,
            )
        } else {
            // Standard left-to-right: a op b op c
            let r1 = match op1 {
                MathOp::Add => a as f64 + b as f64,
                MathOp::Sub => a as f64 - b as f64,
                MathOp::Mul => a as f64 * b as f64,
            };
            let result = match op2 {
                MathOp::Add => r1 + c as f64,
                MathOp::Sub => r1 - c as f64,
                MathOp::Mul => r1 * c as f64,
            };
            (
                format!("{} {} {} {} {}", a, op1.symbol(), b, op2.symbol(), c),
                result,
            )
        }
    } else {
        // 3 operators
        let a = rng.gen_range(1..=max_operand.min(20));
        let b = rng.gen_range(1..=max_operand.min(20));
        let c = rng.gen_range(1..=max_operand.min(20));
        let d = rng.gen_range(1..=max_operand.min(20));
        let op1 = random_op(rng, false);
        let op2 = random_op(rng, false);
        let op3 = random_op(rng, false);

        // Evaluate left to right
        let mut result = a as f64;
        result = apply_op(result, b as f64, op1);
        result = apply_op(result, c as f64, op2);
        result = apply_op(result, d as f64, op3);

        (
            format!(
                "{} {} {} {} {} {} {}",
                a,
                op1.symbol(),
                b,
                op2.symbol(),
                c,
                op3.symbol(),
                d
            ),
            result,
        )
    }
}

fn apply_op(a: f64, b: f64, op: MathOp) -> f64 {
    match op {
        MathOp::Add => a + b,
        MathOp::Sub => a - b,
        MathOp::Mul => a * b,
    }
}

fn render_math_svg(rng: &mut ChaCha8Rng, expression: &str, difficulty: &DifficultyParams) -> String {
    let width = 400;
    let height = 120;
    let c = difficulty.complexity;

    let mut svg = String::with_capacity(2048);
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

    // Thick bezier noise curves
    let curve_count = 4 + (c * 7.0) as u32;
    for _ in 0..curve_count {
        let x1 = rng.gen_range(-10..width + 10);
        let y1 = rng.gen_range(10..height - 10);
        let cx1 = rng.gen_range(0..width);
        let cy1 = rng.gen_range(0..height);
        let x2 = rng.gen_range(-10..width + 10);
        let y2 = rng.gen_range(10..height - 10);
        let sw = 2.0 + rng.gen::<f32>() * (2.0 + c * 3.0);
        let r = rng.gen_range(120..230);
        let g = rng.gen_range(120..230);
        let b = rng.gen_range(120..230);
        let opacity = 0.4 + rng.gen::<f32>() * 0.35;
        write!(
            svg,
            r#"<path d="M{x1},{y1} Q{cx1},{cy1} {x2},{y2}" stroke="rgb({r},{g},{b})" stroke-width="{sw:.1}" fill="none" opacity="{opacity:.2}"/>"#
        )
        .unwrap();
    }

    // Strikethrough lines across the text band
    let strike_count = 2 + (c * 4.0) as u32;
    let mid_y = height as f32 / 2.0;
    for _ in 0..strike_count {
        let x1 = rng.gen_range(0..width / 3);
        let y1 = mid_y + rng.gen_range(-18.0..18.0_f32);
        let x2 = rng.gen_range(width * 2 / 3..width);
        let y2 = mid_y + rng.gen_range(-18.0..18.0_f32);
        let sw = 2.0 + rng.gen::<f32>() * (1.5 + c * 2.0);
        let r = rng.gen_range(140..235);
        let g = rng.gen_range(140..235);
        let b = rng.gen_range(140..235);
        write!(
            svg,
            r#"<line x1="{x1}" y1="{y1:.0}" x2="{x2}" y2="{y2:.0}" stroke="rgb({r},{g},{b})" stroke-width="{sw:.1}" opacity="{:.2}"/>"#,
            0.4 + c * 0.3
        )
        .unwrap();
    }

    // Decoy numbers/operators — placed in the text band so they overlap
    let decoy_chars = b"0123456789+-x=";
    let decoy_count = (3.0 + c * 7.0) as u32;
    for _ in 0..decoy_count {
        let ch = decoy_chars[rng.gen_range(0..decoy_chars.len())] as char;
        let dx = rng.gen_range(20..width - 20) as f32;
        let dy = mid_y + rng.gen_range(-30.0..30.0_f32);
        let size = 20.0 + rng.gen::<f32>() * 18.0;
        let rot = rng.gen::<f32>() * 50.0 - 25.0;
        let r = rng.gen_range(110..210);
        let g = rng.gen_range(110..210);
        let b = rng.gen_range(110..210);
        let opacity = 0.2 + rng.gen::<f32>() * (0.15 + c * 0.25);
        write!(
            svg,
            r#"<text x="{dx:.0}" y="{dy:.0}" font-family="monospace" font-size="{size:.0}" font-weight="bold" fill="rgb({r},{g},{b})" text-anchor="middle" dominant-baseline="central" transform="rotate({rot:.0},{dx:.0},{dy:.0})" opacity="{opacity:.2}">{ch}</text>"#
        )
        .unwrap();
    }

    // Render expression with distortion
    let prompt = format!("{} = ?", expression);
    let char_spacing = width as f32 / (prompt.len() as f32 + 2.0);
    let warp = 4.0 + c * 14.0;

    let wave_freq = 0.4 + rng.gen::<f32>() * 1.2;
    let wave_phase = rng.gen::<f32>() * std::f32::consts::PI * 2.0;

    for (i, ch) in prompt.chars().enumerate() {
        let x = char_spacing * (i as f32 + 1.0);
        let wave_y = (wave_freq * i as f32 + wave_phase).sin() * warp * 0.5;
        let jitter_y = rng.gen::<f32>() * warp * 0.6 - warp * 0.3;
        let y = height as f32 / 2.0 + wave_y + jitter_y;
        let rotation = rng.gen::<f32>() * warp * 1.2 - warp * 0.6;
        let skew_x = rng.gen::<f32>() * c * 12.0 - c * 6.0;
        let bright = 165.0 + (1.0 - c * 0.25) * 75.0;
        let r = rng.gen_range(bright as u32..(bright as u32 + 35).min(255));
        let g = rng.gen_range(bright as u32..(bright as u32 + 35).min(255));
        let b_val = rng.gen_range(bright as u32..(bright as u32 + 35).min(255));

        let escaped = match ch {
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '&' => "&amp;".to_string(),
            _ => ch.to_string(),
        };
        write!(
            svg,
            r#"<text x="{x:.1}" y="{y:.1}" font-family="monospace" font-size="32" font-weight="bold" fill="rgb({r},{g},{b_val})" text-anchor="middle" dominant-baseline="central" transform="rotate({rotation:.1},{x:.1},{y:.1}) skewX({skew_x:.1})">{escaped}</text>"#,
        )
        .unwrap();
    }

    // Overlay noise on top of text
    let overlay_count = 1 + (c * 2.0) as u32;
    for _ in 0..overlay_count {
        let x1 = rng.gen_range(0..width);
        let y1 = mid_y + rng.gen_range(-25.0..25.0_f32);
        let cx1 = rng.gen_range(0..width);
        let cy1 = rng.gen_range(20..height - 20);
        let x2 = rng.gen_range(0..width);
        let y2 = mid_y + rng.gen_range(-25.0..25.0_f32);
        let sw = 1.5 + rng.gen::<f32>() * 2.0;
        let r = rng.gen_range(130..220);
        let g = rng.gen_range(130..220);
        let b = rng.gen_range(130..220);
        write!(
            svg,
            r#"<path d="M{x1},{y1:.0} Q{cx1},{cy1} {x2},{y2:.0}" stroke="rgb({r},{g},{b})" stroke-width="{sw:.1}" fill="none" opacity="{:.2}"/>"#,
            0.25 + c * 0.3
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
    fn test_math_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 4000,
            complexity: 0.0,
            noise: 0.0,
        };
        let gen = MathGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert_eq!(instance.captcha_type, CaptchaType::SimpleMath);
        assert!(matches!(instance.solution, Solution::Number(_)));
        assert!(matches!(instance.render_data, RenderPayload::Svg(_)));
    }

    #[test]
    fn test_math_correct_answer_validates() {
        let mut rng = rng_from_seed(77);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = MathGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::Number(answer) = instance.solution {
            assert!(gen.validate(&instance, &PlayerAnswer::Number(answer)));
        }
    }

    #[test]
    fn test_math_wrong_answer_rejects() {
        let mut rng = rng_from_seed(77);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = MathGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert!(!gen.validate(&instance, &PlayerAnswer::Number(999999.0)));
    }

    #[test]
    fn test_math_seed_determinism() {
        let difficulty = DifficultyParams {
            level: 10,
            round_number: 3,
            time_limit_ms: 7000,
            complexity: 0.5,
            noise: 0.3,
        };
        let gen = MathGenerator;

        let mut rng1 = rng_from_seed(555);
        let inst1 = gen.generate(&mut rng1, &difficulty);

        let mut rng2 = rng_from_seed(555);
        let inst2 = gen.generate(&mut rng2, &difficulty);

        if let (Solution::Number(n1), Solution::Number(n2)) = (inst1.solution, inst2.solution) {
            assert!((n1 - n2).abs() < 0.001);
        }
    }
}
