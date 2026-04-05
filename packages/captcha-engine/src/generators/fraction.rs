use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct FractionComparisonGenerator;

/// Simple fraction representation
#[derive(Debug, Clone, Copy)]
struct Fraction {
    numerator: u32,
    denominator: u32,
}

impl Fraction {
    fn value(&self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }
}

impl CaptchaGenerator for FractionComparisonGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        let c = difficulty.complexity;
        let n = difficulty.noise;

        // Generate two fractions that are different
        let (frac_a, frac_b) = generate_fraction_pair(rng, c);

        // Determine which is larger
        let larger_index: u32 = if frac_a.value() > frac_b.value() { 0 } else { 1 };

        let width = 500;
        let height = 280;
        let rect_w = 180.0;
        let rect_h = 200.0;
        let gap = 40.0;
        let start_x = (width as f32 - 2.0 * rect_w - gap) / 2.0;
        let start_y = 45.0;

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

        // Prompt
        write!(
            svg,
            r##"<text x="{}" y="28" font-family="sans-serif" font-size="15" fill="#cccccc" text-anchor="middle">Click the rectangle that is MORE filled</text>"##,
            width / 2
        )
        .unwrap();

        // Draw fraction A (left)
        draw_fraction_rect(
            &mut svg,
            rng,
            &frac_a,
            start_x,
            start_y,
            rect_w,
            rect_h,
            0,
            n,
            "#3498db",
            "#1a3a5c",
        );

        // Draw fraction B (right)
        draw_fraction_rect(
            &mut svg,
            rng,
            &frac_b,
            start_x + rect_w + gap,
            start_y,
            rect_w,
            rect_h,
            1,
            n,
            "#e74c3c",
            "#5c1a1a",
        );

        // Fraction labels below rectangles
        write!(
            svg,
            r##"<text x="{:.0}" y="{:.0}" font-family="sans-serif" font-size="14" fill="#888888" text-anchor="middle">{}/{}</text>"##,
            start_x + rect_w / 2.0,
            start_y + rect_h + 22.0,
            frac_a.numerator,
            frac_a.denominator
        )
        .unwrap();
        write!(
            svg,
            r##"<text x="{:.0}" y="{:.0}" font-family="sans-serif" font-size="14" fill="#888888" text-anchor="middle">{}/{}</text>"##,
            start_x + rect_w + gap + rect_w / 2.0,
            start_y + rect_h + 22.0,
            frac_b.numerator,
            frac_b.denominator
        )
        .unwrap();

        // Clickable overlay areas
        write!(
            svg,
            r##"<rect x="{:.0}" y="{:.0}" width="{rect_w:.0}" height="{rect_h:.0}" fill="transparent" data-index="0" style="cursor:pointer"/>"##,
            start_x, start_y
        )
        .unwrap();
        write!(
            svg,
            r##"<rect x="{:.0}" y="{:.0}" width="{rect_w:.0}" height="{rect_h:.0}" fill="transparent" data-index="1" style="cursor:pointer"/>"##,
            start_x + rect_w + gap, start_y
        )
        .unwrap();

        svg.push_str("</svg>");

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::SelectedIndices(vec![larger_index]),
            expected_solve_time_ms: expected_solve_time(difficulty.time_limit_ms),
            point_value: CaptchaType::FractionComparison.base_points(),
            captcha_type: CaptchaType::FractionComparison,
            time_limit_ms: difficulty.time_limit_ms,
        }
    }

    fn validate(&self, instance: &CaptchaInstance, answer: &PlayerAnswer) -> bool {
        instance.validate(answer)
    }
}

fn generate_fraction_pair(rng: &mut ChaCha8Rng, complexity: f32) -> (Fraction, Fraction) {
    if complexity < 0.3 {
        // Easy: obvious differences, small denominators
        let denom = rng.gen_range(2..=4_u32);
        let a_num = rng.gen_range(1..denom);
        // Make sure b is noticeably different
        let b_num = if a_num <= denom / 2 {
            rng.gen_range((denom / 2 + 1)..=denom)
        } else {
            rng.gen_range(1..=(denom / 2))
        };
        (
            Fraction { numerator: a_num, denominator: denom },
            Fraction { numerator: b_num, denominator: denom },
        )
    } else if complexity < 0.6 {
        // Medium: different denominators
        let denoms = [3, 4, 5, 6];
        let d1 = denoms[rng.gen_range(0..denoms.len())];
        let d2 = denoms[rng.gen_range(0..denoms.len())];
        let n1 = rng.gen_range(1..d1);
        let mut n2 = rng.gen_range(1..d2);
        // Ensure they're different values
        let v1 = n1 as f64 / d1 as f64;
        let v2 = n2 as f64 / d2 as f64;
        if (v1 - v2).abs() < 0.05 {
            n2 = if n2 < d2 - 1 { n2 + 1 } else { n2 - 1 };
            if n2 == 0 { n2 = 1; }
        }
        (
            Fraction { numerator: n1, denominator: d1 },
            Fraction { numerator: n2, denominator: d2 },
        )
    } else {
        // Hard: closer fractions, larger denominators
        let denoms = [6, 8, 10, 12];
        let d1 = denoms[rng.gen_range(0..denoms.len())];
        let d2 = denoms[rng.gen_range(0..denoms.len())];
        let n1 = rng.gen_range(2..d1 - 1);
        let mut n2 = rng.gen_range(2..d2 - 1);
        // Ensure they're different but close
        let v1 = n1 as f64 / d1 as f64;
        let v2 = n2 as f64 / d2 as f64;
        if (v1 - v2).abs() < 0.01 {
            n2 = if n2 < d2 - 1 { n2 + 1 } else { n2 - 1 };
            if n2 == 0 { n2 = 1; }
        }
        (
            Fraction { numerator: n1, denominator: d1 },
            Fraction { numerator: n2, denominator: d2 },
        )
    }
}

fn draw_fraction_rect(
    svg: &mut String,
    rng: &mut ChaCha8Rng,
    frac: &Fraction,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    _index: u32,
    noise: f32,
    fill_color: &str,
    empty_color: &str,
) {
    let denom = frac.denominator;
    let numer = frac.numerator;

    // Divide rectangle into `denom` horizontal rows
    let row_h = h / denom as f32;

    // Outer border
    write!(
        svg,
        r##"<rect x="{x:.0}" y="{y:.0}" width="{w:.0}" height="{h:.0}" fill="none" stroke="#555555" stroke-width="2"/>"##
    )
    .unwrap();

    // Draw rows from bottom up, shaded rows at bottom
    for i in 0..denom {
        let ry = y + (denom - 1 - i) as f32 * row_h;
        let color = if i < numer { fill_color } else { empty_color };
        write!(
            svg,
            r##"<rect x="{x:.0}" y="{ry:.1}" width="{w:.0}" height="{row_h:.1}" fill="{color}" stroke="#444444" stroke-width="0.5"/>"##
        )
        .unwrap();
    }

    // Noise: random faint rectangles
    let noise_count = (noise * 5.0) as u32;
    for _ in 0..noise_count {
        let nx = x + rng.gen_range(0.0..w);
        let ny = y + rng.gen_range(0.0..h);
        let nw = rng.gen_range(5.0..30.0_f32);
        let nh = rng.gen_range(5.0..20.0_f32);
        let r = rng.gen_range(60..160);
        let g = rng.gen_range(60..160);
        let b = rng.gen_range(60..160);
        let opacity = 0.05 + rng.gen::<f32>() * 0.1;
        write!(
            svg,
            r##"<rect x="{nx:.1}" y="{ny:.1}" width="{nw:.1}" height="{nh:.1}" fill="rgb({r},{g},{b})" opacity="{opacity:.2}"/>"##
        )
        .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::rng_from_seed;

    #[test]
    fn test_fractioncomparison_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 4000,
            complexity: 0.0,
            noise: 0.0,
        };
        let gen = FractionComparisonGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert_eq!(instance.captcha_type, CaptchaType::FractionComparison);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert_eq!(indices.len(), 1);
            assert!(indices[0] <= 1);
        } else {
            panic!("Expected SelectedIndices solution");
        }
        if let RenderPayload::Svg(ref s) = instance.render_data {
            assert!(s.contains("<svg"));
            assert!(s.contains("data-index=\"0\""));
            assert!(s.contains("data-index=\"1\""));
        } else {
            panic!("Expected SVG render data");
        }
    }

    #[test]
    fn test_fractioncomparison_seed_determinism() {
        let difficulty = DifficultyParams {
            level: 10,
            round_number: 3,
            time_limit_ms: 6000,
            complexity: 0.5,
            noise: 0.3,
        };
        let gen = FractionComparisonGenerator;

        let mut rng1 = rng_from_seed(555);
        let inst1 = gen.generate(&mut rng1, &difficulty);

        let mut rng2 = rng_from_seed(555);
        let inst2 = gen.generate(&mut rng2, &difficulty);

        if let (Solution::SelectedIndices(i1), Solution::SelectedIndices(i2)) =
            (&inst1.solution, &inst2.solution)
        {
            assert_eq!(i1, i2);
        } else {
            panic!("Expected SelectedIndices solutions");
        }
    }

    #[test]
    fn test_fractioncomparison_correct_answer_validates() {
        let mut rng = rng_from_seed(77);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = FractionComparisonGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert!(gen.validate(
                &instance,
                &PlayerAnswer::SelectedIndices(indices.clone())
            ));
        } else {
            panic!("Expected SelectedIndices solution");
        }
    }

    #[test]
    fn test_fractioncomparison_wrong_answer_rejects() {
        let mut rng = rng_from_seed(77);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = FractionComparisonGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        // The correct answer is either [0] or [1], so the wrong one is the other
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            let wrong = if indices[0] == 0 { 1 } else { 0 };
            assert!(!gen.validate(
                &instance,
                &PlayerAnswer::SelectedIndices(vec![wrong])
            ));
        }
        assert!(!gen.validate(&instance, &PlayerAnswer::Text("wrong".to_string())));
    }
}
