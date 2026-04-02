use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct MetamorphicGenerator;

/// Shape types — no circles (rotation is invisible on circles)
#[derive(Debug, Clone, Copy)]
enum ShapeKind {
    Square,
    Triangle,
    Diamond,
    Pentagon,
    Star,
    Arrow,
}

const SHAPE_KINDS: [ShapeKind; 6] = [
    ShapeKind::Square,
    ShapeKind::Triangle,
    ShapeKind::Diamond,
    ShapeKind::Pentagon,
    ShapeKind::Star,
    ShapeKind::Arrow,
];

const COLORS: &[&str] = &[
    "#e74c3c", "#3498db", "#2ecc71", "#f39c12", "#9b59b6", "#1abc9c", "#e67e22", "#e84393",
];

impl CaptchaGenerator for MetamorphicGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        let shape_count = 4 + (difficulty.complexity * 4.0) as usize;
        let shape_count = shape_count.min(8);
        let odd_index = rng.gen_range(0..shape_count);

        let svg = build_svg(rng, shape_count, odd_index, difficulty);

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::SelectedIndices(vec![odd_index as u32]),
            expected_solve_time_ms: expected_solve_time(difficulty.time_limit_ms),
            point_value: CaptchaType::MetamorphicCaptcha.base_points(),
            captcha_type: CaptchaType::MetamorphicCaptcha,
            time_limit_ms: difficulty.time_limit_ms,
        }
    }

    fn validate(&self, instance: &CaptchaInstance, answer: &PlayerAnswer) -> bool {
        instance.validate(answer)
    }
}

fn build_svg(
    rng: &mut ChaCha8Rng,
    count: usize,
    odd_index: usize,
    difficulty: &DifficultyParams,
) -> String {
    let width = 500;
    let height = 420;
    let c = difficulty.complexity;

    let mut svg = String::with_capacity(8192);
    write!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}">"#
    )
    .unwrap();

    write!(svg, r##"<rect width="{width}" height="{height}" fill="#1a1a2e"/>"##).unwrap();

    // Pick the challenge type — what makes the odd one different
    let challenge = rng.gen_range(0..3);
    let prompt = match challenge {
        0 => "All shapes rotate the same way. Which one rotates DIFFERENTLY?",
        1 => "All shapes pulse at the same speed. Which one is OFF-BEAT?",
        _ => "All shapes drift in the same direction. Which one drifts DIFFERENTLY?",
    };

    write!(
        svg,
        r##"<text x="{}" y="28" font-family="sans-serif" font-size="14" fill="#aaaacc" text-anchor="middle">{prompt}</text>"##,
        width / 2
    )
    .unwrap();

    // Shared animation parameters (all normal shapes share these)
    let normal_duration = 2.0 + rng.gen::<f32>() * 2.0;
    let normal_direction: f32 = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
    let normal_amplitude = 10.0 + rng.gen::<f32>() * 15.0;

    // Odd shape gets different params — subtler difference at high complexity
    let odd_duration = if c > 0.6 {
        normal_duration * (1.3 + rng.gen::<f32>() * 0.4) // slightly different speed
    } else {
        normal_duration * (0.3 + rng.gen::<f32>() * 0.3) // obviously different speed
    };
    let odd_direction = -normal_direction; // opposite direction
    let odd_amplitude = if c > 0.6 {
        normal_amplitude * (0.7 + rng.gen::<f32>() * 0.3)
    } else {
        normal_amplitude * (1.5 + rng.gen::<f32>() * 1.0)
    };

    let cols = if count <= 4 { 2 } else { 4 };
    let rows = count.div_ceil(cols);
    let cell_w = width as f32 / cols as f32;
    let cell_h = (height as f32 - 70.0) / rows as f32;
    let start_y = 50.0;

    // All shapes are the same kind (so the difference is purely in animation)
    let kind = SHAPE_KINDS[rng.gen_range(0..SHAPE_KINDS.len())];

    for i in 0..count {
        let col = i % cols;
        let row = i / cols;
        let cx = cell_w * (col as f32 + 0.5);
        let cy = start_y + cell_h * (row as f32 + 0.5);
        let size = 22.0 + rng.gen::<f32>() * 8.0;
        let color = COLORS[i % COLORS.len()];

        let is_odd = i == odd_index;
        let (dur, dir, amp) = if is_odd {
            (odd_duration, odd_direction, odd_amplitude)
        } else {
            (normal_duration, normal_direction, normal_amplitude)
        };

        // Draw shape with animation
        let shape_svg = render_shape(kind, cx, cy, size, color);

        write!(svg, r#"<g>"#).unwrap();
        svg.push_str(&shape_svg);

        match challenge {
            0 => {
                // Rotation — all rotate one direction, odd rotates opposite
                let to = 360.0 * dir;
                let odd_to = 360.0 * (-dir);
                write!(
                    svg,
                    r#"<animateTransform attributeName="transform" type="rotate" from="0 {cx:.0} {cy:.0}" to="{:.0} {cx:.0} {cy:.0}" dur="{dur:.1}s" repeatCount="indefinite"/>"#,
                    if is_odd { odd_to } else { to }
                )
                .unwrap();
            }
            1 => {
                // Pulse — use translate to simulate scale from center
                // Move to origin, scale, move back
                // Animate opacity as a visible pulse
                write!(
                    svg,
                    r#"<animate attributeName="opacity" values="0.9;0.3;0.9" dur="{dur:.1}s" repeatCount="indefinite"/>"#,
                )
                .unwrap();
                // Also animate a slight translate bounce for visual effect
                let bounce = if is_odd { -amp * 0.3 } else { amp * 0.3 };
                write!(
                    svg,
                    r#"<animateTransform attributeName="transform" type="translate" values="0,0;0,{bounce:.0};0,0" dur="{dur:.1}s" repeatCount="indefinite"/>"#,
                )
                .unwrap();
            }
            _ => {
                // Drift — all drift one direction, odd drifts opposite
                let dx = amp * dir;
                let odd_dx = odd_amplitude * (-dir);
                write!(
                    svg,
                    r#"<animateTransform attributeName="transform" type="translate" values="0,0;{:.0},0;0,0" dur="{dur:.1}s" repeatCount="indefinite"/>"#,
                    if is_odd { odd_dx } else { dx }
                )
                .unwrap();
            }
        }

        svg.push_str("</g>");

        // Clickable overlay
        write!(
            svg,
            r#"<rect x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}" fill="transparent" data-index="{i}" style="cursor:pointer"/>"#,
            cx - cell_w / 2.0 + 4.0,
            cy - cell_h / 2.0 + 4.0,
            cell_w - 8.0,
            cell_h - 8.0,
        )
        .unwrap();
    }

    svg.push_str("</svg>");
    svg
}

fn render_shape(kind: ShapeKind, cx: f32, cy: f32, size: f32, color: &str) -> String {
    match kind {
        ShapeKind::Square => {
            format!(
                r#"<rect x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}" fill="{color}" opacity="0.9"/>"#,
                cx - size, cy - size, size * 2.0, size * 2.0,
            )
        }
        ShapeKind::Triangle => {
            format!(
                r#"<polygon points="{cx:.0},{:.0} {:.0},{:.0} {:.0},{:.0}" fill="{color}" opacity="0.9"/>"#,
                cy - size * 1.2, cx - size, cy + size, cx + size, cy + size,
            )
        }
        ShapeKind::Diamond => {
            format!(
                r#"<polygon points="{cx:.0},{:.0} {:.0},{cy:.0} {cx:.0},{:.0} {:.0},{cy:.0}" fill="{color}" opacity="0.9"/>"#,
                cy - size * 1.3, cx + size, cy + size * 1.3, cx - size,
            )
        }
        ShapeKind::Pentagon => {
            let mut points = String::new();
            for j in 0..5 {
                let angle = std::f32::consts::PI * 2.0 * j as f32 / 5.0 - std::f32::consts::FRAC_PI_2;
                let px = cx + angle.cos() * size;
                let py = cy + angle.sin() * size;
                if !points.is_empty() { points.push(' '); }
                write!(points, "{px:.0},{py:.0}").unwrap();
            }
            format!(r#"<polygon points="{points}" fill="{color}" opacity="0.9"/>"#)
        }
        ShapeKind::Star => {
            let mut points = String::new();
            for j in 0..10 {
                let angle = std::f32::consts::PI * 2.0 * j as f32 / 10.0 - std::f32::consts::FRAC_PI_2;
                let r = if j % 2 == 0 { size } else { size * 0.45 };
                let px = cx + angle.cos() * r;
                let py = cy + angle.sin() * r;
                if !points.is_empty() { points.push(' '); }
                write!(points, "{px:.0},{py:.0}").unwrap();
            }
            format!(r#"<polygon points="{points}" fill="{color}" opacity="0.9"/>"#)
        }
        ShapeKind::Arrow => {
            // Upward-pointing arrow
            let hw = size * 0.7;
            let hh = size * 1.2;
            let shaft_w = size * 0.3;
            format!(
                r#"<polygon points="{cx:.0},{:.0} {:.0},{cy:.0} {:.0},{cy:.0} {:.0},{:.0} {:.0},{:.0} {:.0},{cy:.0} {:.0},{cy:.0}" fill="{color}" opacity="0.9"/>"#,
                cy - hh,
                cx - hw, // left of arrowhead
                cx - shaft_w, // left of shaft at center
                cx - shaft_w, cy + hh, // bottom left
                cx + shaft_w, cy + hh, // bottom right
                cx + shaft_w, // right of shaft at center
                cx + hw, // right of arrowhead
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::rng_from_seed;

    #[test]
    fn test_metamorphic_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let d = DifficultyParams { level: 50, round_number: 1, time_limit_ms: 10000, complexity: 0.5, noise: 0.4 };
        let gen = MetamorphicGenerator;
        let inst = gen.generate(&mut rng, &d);
        assert_eq!(inst.captcha_type, CaptchaType::MetamorphicCaptcha);
        if let RenderPayload::Svg(ref s) = inst.render_data {
            assert!(s.contains("<svg"));
            assert!(s.contains("animateTransform"));
            assert!(s.contains("data-index"));
        } else {
            panic!("Expected SVG");
        }
    }

    #[test]
    fn test_metamorphic_seed_determinism() {
        let d = DifficultyParams { level: 50, round_number: 3, time_limit_ms: 12000, complexity: 0.6, noise: 0.5 };
        let gen = MetamorphicGenerator;
        let mut r1 = rng_from_seed(123);
        let i1 = gen.generate(&mut r1, &d);
        let mut r2 = rng_from_seed(123);
        let i2 = gen.generate(&mut r2, &d);
        if let (Solution::SelectedIndices(a), Solution::SelectedIndices(b)) = (&i1.solution, &i2.solution) {
            assert_eq!(a, b);
        }
    }

    #[test]
    fn test_metamorphic_correct_validates() {
        let mut rng = rng_from_seed(42);
        let d = DifficultyParams { level: 50, round_number: 1, time_limit_ms: 10000, complexity: 0.5, noise: 0.4 };
        let gen = MetamorphicGenerator;
        let inst = gen.generate(&mut rng, &d);
        if let Solution::SelectedIndices(ref idx) = inst.solution {
            assert!(gen.validate(&inst, &PlayerAnswer::SelectedIndices(idx.clone())));
        }
    }

    #[test]
    fn test_metamorphic_wrong_rejects() {
        let mut rng = rng_from_seed(42);
        let d = DifficultyParams { level: 50, round_number: 1, time_limit_ms: 10000, complexity: 0.5, noise: 0.4 };
        let gen = MetamorphicGenerator;
        let inst = gen.generate(&mut rng, &d);
        assert!(!gen.validate(&inst, &PlayerAnswer::SelectedIndices(vec![999])));
    }

    #[test]
    fn test_metamorphic_difficulty_scaling() {
        let gen = MetamorphicGenerator;
        let lo = DifficultyParams { level: 1, round_number: 1, time_limit_ms: 10000, complexity: 0.0, noise: 0.0 };
        let hi = DifficultyParams { level: 100, round_number: 20, time_limit_ms: 20000, complexity: 1.0, noise: 1.0 };
        let mut r1 = rng_from_seed(42);
        let i1 = gen.generate(&mut r1, &lo);
        let mut r2 = rng_from_seed(42);
        let i2 = gen.generate(&mut r2, &hi);
        if let (RenderPayload::Svg(s1), RenderPayload::Svg(s2)) = (&i1.render_data, &i2.render_data) {
            assert!(s2.len() > s1.len());
        }
    }
}
