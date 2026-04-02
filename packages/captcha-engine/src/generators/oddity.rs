use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct OddityGenerator;

/// HSL color for shape generation
#[derive(Debug, Clone, Copy)]
struct Hsl {
    h: f32,
    s: f32,
    l: f32,
}

impl Hsl {
    fn to_css(self) -> String {
        let (r, g, b) = hsl_to_rgb(self.h, self.s, self.l);
        format!("rgb({r},{g},{b})")
    }
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let h = h / 360.0;
    let s = s / 100.0;
    let l = l / 100.0;

    if s == 0.0 {
        let v = (l * 255.0) as u8;
        return (v, v, v);
    }

    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;

    let r = hue_channel(p, q, h + 1.0 / 3.0);
    let g = hue_channel(p, q, h);
    let b = hue_channel(p, q, h - 1.0 / 3.0);

    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn hue_channel(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 0.5 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OddShape {
    Circle,
    Rect,
    Triangle,
    Pentagon,
    Hexagon,
}

const ALL_SHAPES: [OddShape; 5] = [
    OddShape::Circle,
    OddShape::Rect,
    OddShape::Triangle,
    OddShape::Pentagon,
    OddShape::Hexagon,
];

impl CaptchaGenerator for OddityGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        // 4-8 shapes based on complexity
        let shape_count = 4 + (difficulty.complexity * 4.0) as usize;
        let shape_count = shape_count.min(8);

        // Pick which shape is the odd one out
        let odd_index = rng.gen_range(0..shape_count);

        let c = difficulty.complexity;

        // Common attributes for the group
        let base_hue = rng.gen_range(0.0..360.0_f32);
        let base_sat = rng.gen_range(50.0..75.0_f32);
        let base_light = rng.gen_range(45.0..60.0_f32);
        let base_shape = ALL_SHAPES[rng.gen_range(0..ALL_SHAPES.len())];
        let base_size = rng.gen_range(25.0..40.0_f32);

        // Decide what makes the odd one different
        // At low difficulty: obviously different (wrong color among same-colored)
        // At high difficulty: differs in only one subtle attribute
        let hue_variation = 5.0 + c * 10.0; // Normal shapes vary slightly within this range

        // Generate shapes
        let mut shapes: Vec<(OddShape, Hsl, f32)> = Vec::with_capacity(shape_count);

        for i in 0..shape_count {
            if i == odd_index {
                // The odd one
                let (odd_shape, odd_color, odd_size) = generate_odd_attributes(
                    rng,
                    base_shape,
                    base_hue,
                    base_sat,
                    base_light,
                    base_size,
                    c,
                );
                shapes.push((odd_shape, odd_color, odd_size));
            } else {
                // Normal shape — slight variation within the group
                let h = base_hue + rng.gen_range(-hue_variation..hue_variation);
                let s = base_sat + rng.gen_range(-5.0..5.0_f32);
                let l = base_light + rng.gen_range(-3.0..3.0_f32);
                let size = base_size + rng.gen_range(-3.0..3.0_f32);
                shapes.push((base_shape, Hsl { h, s, l }, size));
            }
        }

        let svg = generate_oddity_svg(rng, &shapes, difficulty);

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::SelectedIndices(vec![odd_index as u32]),
            expected_solve_time_ms: expected_solve_time(difficulty.time_limit_ms),
            point_value: CaptchaType::SemanticOddity.base_points(),
            captcha_type: CaptchaType::SemanticOddity,
            time_limit_ms: difficulty.time_limit_ms,
        }
    }

    fn validate(&self, instance: &CaptchaInstance, answer: &PlayerAnswer) -> bool {
        instance.validate(answer)
    }
}

fn generate_odd_attributes(
    rng: &mut ChaCha8Rng,
    base_shape: OddShape,
    base_hue: f32,
    base_sat: f32,
    base_light: f32,
    base_size: f32,
    complexity: f32,
) -> (OddShape, Hsl, f32) {
    if complexity < 0.3 {
        // Easy: obviously different color (complementary hue)
        let odd_hue = (base_hue + 150.0 + rng.gen_range(-30.0..30.0_f32)) % 360.0;
        (
            base_shape,
            Hsl {
                h: odd_hue,
                s: base_sat,
                l: base_light,
            },
            base_size,
        )
    } else if complexity < 0.6 {
        // Medium: different shape OR noticeably different color
        if rng.gen_bool(0.5) {
            // Different shape, same color
            let mut odd_shape = ALL_SHAPES[rng.gen_range(0..ALL_SHAPES.len())];
            while odd_shape == base_shape {
                odd_shape = ALL_SHAPES[rng.gen_range(0..ALL_SHAPES.len())];
            }
            (
                odd_shape,
                Hsl {
                    h: base_hue + rng.gen_range(-8.0..8.0_f32),
                    s: base_sat,
                    l: base_light,
                },
                base_size,
            )
        } else {
            // Same shape, shifted hue (60-90 degrees off)
            let shift = 60.0 + rng.gen_range(0.0..30.0_f32);
            let odd_hue = (base_hue + shift) % 360.0;
            (
                base_shape,
                Hsl {
                    h: odd_hue,
                    s: base_sat,
                    l: base_light,
                },
                base_size,
            )
        }
    } else {
        // Hard: subtle single-attribute difference
        let attr = rng.gen_range(0..3);
        match attr {
            0 => {
                // Subtle hue shift (20-35 degrees)
                let shift = 20.0 + rng.gen_range(0.0..15.0_f32);
                let odd_hue = (base_hue + shift) % 360.0;
                (
                    base_shape,
                    Hsl {
                        h: odd_hue,
                        s: base_sat,
                        l: base_light,
                    },
                    base_size,
                )
            }
            1 => {
                // Subtle size difference
                let size_diff = 8.0 + rng.gen_range(0.0..5.0_f32);
                (
                    base_shape,
                    Hsl {
                        h: base_hue + rng.gen_range(-5.0..5.0_f32),
                        s: base_sat,
                        l: base_light,
                    },
                    base_size + size_diff,
                )
            }
            _ => {
                // Subtle lightness difference
                let l_diff = 10.0 + rng.gen_range(0.0..8.0_f32);
                (
                    base_shape,
                    Hsl {
                        h: base_hue + rng.gen_range(-5.0..5.0_f32),
                        s: base_sat,
                        l: base_light + l_diff,
                    },
                    base_size,
                )
            }
        }
    }
}

fn generate_oddity_svg(
    rng: &mut ChaCha8Rng,
    shapes: &[(OddShape, Hsl, f32)],
    difficulty: &DifficultyParams,
) -> String {
    let width = 500;
    let height = 400;
    let c = difficulty.complexity;
    let shape_count = shapes.len();

    let mut svg = String::with_capacity(8192);
    write!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}">"#
    )
    .unwrap();

    // Background
    write!(
        svg,
        r##"<rect width="{width}" height="{height}" fill="#1a1a2e"/>"##
    )
    .unwrap();

    // Instruction text
    write!(
        svg,
        r##"<text x="{}" y="30" font-family="sans-serif" font-size="16" fill="#aaaacc" text-anchor="middle">Which one doesn't belong?</text>"##,
        width / 2
    )
    .unwrap();

    // Layout shapes in a grid
    let cols = if shape_count <= 4 { 2 } else { 4 };
    let rows = shape_count.div_ceil(cols);
    let cell_w = width as f32 / cols as f32;
    let cell_h = (height as f32 - 60.0) / rows as f32;
    let start_y = 50.0;

    for (i, (shape_kind, color, size)) in shapes.iter().enumerate() {
        let col = i % cols;
        let row = i / cols;
        let cx = cell_w * (col as f32 + 0.5);
        let cy = start_y + cell_h * (row as f32 + 0.5);
        let css_color = color.to_css();

        draw_odd_shape(&mut svg, *shape_kind, cx, cy, *size, &css_color);

        // Transparent clickable overlay with data-index
        write!(
            svg,
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="transparent" data-index="{i}"/>"#,
            cx - cell_w / 2.0 + 5.0,
            cy - cell_h / 2.0 + 5.0,
            cell_w - 10.0,
            cell_h - 10.0,
        )
        .unwrap();
    }

    // Noise dots
    let dot_count = 10 + (c * 30.0) as u32;
    for _ in 0..dot_count {
        let dx = rng.gen_range(0..width);
        let dy = rng.gen_range(50..height);
        let dr = 1.0 + rng.gen::<f32>() * (1.0 + c * 2.0);
        let r = rng.gen_range(60..180);
        let g = rng.gen_range(60..180);
        let b = rng.gen_range(60..180);
        let opacity = 0.1 + rng.gen::<f32>() * 0.2;
        write!(
            svg,
            r#"<circle cx="{dx}" cy="{dy}" r="{dr:.1}" fill="rgb({r},{g},{b})" opacity="{opacity:.2}"/>"#
        )
        .unwrap();
    }

    svg.push_str("</svg>");
    svg
}

fn draw_odd_shape(svg: &mut String, kind: OddShape, cx: f32, cy: f32, size: f32, color: &str) {
    match kind {
        OddShape::Circle => {
            write!(
                svg,
                r#"<circle cx="{cx:.1}" cy="{cy:.1}" r="{size:.1}" fill="{color}" opacity="0.9"/>"#,
            )
            .unwrap();
        }
        OddShape::Rect => {
            write!(
                svg,
                r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="{color}" opacity="0.9"/>"#,
                cx - size, cy - size, size * 2.0, size * 2.0,
            )
            .unwrap();
        }
        OddShape::Triangle => {
            write!(
                svg,
                r#"<polygon points="{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}" fill="{color}" opacity="0.9"/>"#,
                cx, cy - size, cx - size, cy + size, cx + size, cy + size,
            )
            .unwrap();
        }
        OddShape::Pentagon => {
            let mut points = String::new();
            for k in 0..5 {
                let angle = std::f32::consts::PI * 2.0 * k as f32 / 5.0 - std::f32::consts::FRAC_PI_2;
                let px = cx + size * angle.cos();
                let py = cy + size * angle.sin();
                if k > 0 {
                    points.push(' ');
                }
                write!(points, "{px:.1},{py:.1}").unwrap();
            }
            write!(
                svg,
                r#"<polygon points="{points}" fill="{color}" opacity="0.9"/>"#,
            )
            .unwrap();
        }
        OddShape::Hexagon => {
            let mut points = String::new();
            for k in 0..6 {
                let angle = std::f32::consts::PI * 2.0 * k as f32 / 6.0 - std::f32::consts::FRAC_PI_2;
                let px = cx + size * angle.cos();
                let py = cy + size * angle.sin();
                if k > 0 {
                    points.push(' ');
                }
                write!(points, "{px:.1},{py:.1}").unwrap();
            }
            write!(
                svg,
                r#"<polygon points="{points}" fill="{color}" opacity="0.9"/>"#,
            )
            .unwrap();
        }
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
            time_limit_ms: 10000,
            complexity: 0.5,
            noise: 0.3,
        }
    }

    #[test]
    fn test_oddity_determinism() {
        let difficulty = default_difficulty();
        let gen = OddityGenerator;

        let mut rng1 = rng_from_seed(42);
        let inst1 = gen.generate(&mut rng1, &difficulty);

        let mut rng2 = rng_from_seed(42);
        let inst2 = gen.generate(&mut rng2, &difficulty);

        if let (Solution::SelectedIndices(a), Solution::SelectedIndices(b)) =
            (&inst1.solution, &inst2.solution)
        {
            assert_eq!(a, b);
        } else {
            panic!("Expected SelectedIndices solution");
        }
    }

    #[test]
    fn test_oddity_valid_instance() {
        let mut rng = rng_from_seed(99);
        let difficulty = default_difficulty();
        let gen = OddityGenerator;
        let instance = gen.generate(&mut rng, &difficulty);

        assert_eq!(instance.captcha_type, CaptchaType::SemanticOddity);
        if let RenderPayload::Svg(ref s) = instance.render_data {
            assert!(s.contains("<svg"));
            assert!(s.contains("data-index"));
            assert!(s.contains("#1a1a2e"));
            assert!(s.contains("doesn't belong"));
        } else {
            panic!("Expected SVG render data");
        }
    }

    #[test]
    fn test_oddity_correct_validates() {
        let mut rng = rng_from_seed(55);
        let difficulty = default_difficulty();
        let gen = OddityGenerator;
        let instance = gen.generate(&mut rng, &difficulty);

        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert!(gen.validate(&instance, &PlayerAnswer::SelectedIndices(indices.clone())));
        } else {
            panic!("Expected SelectedIndices solution");
        }
    }

    #[test]
    fn test_oddity_wrong_rejects() {
        let mut rng = rng_from_seed(55);
        let difficulty = default_difficulty();
        let gen = OddityGenerator;
        let instance = gen.generate(&mut rng, &difficulty);

        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![99])));
        assert!(!gen.validate(&instance, &PlayerAnswer::Text("wrong".to_string())));
    }

    #[test]
    fn test_oddity_difficulty_scaling() {
        let gen = OddityGenerator;

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
            time_limit_ms: 15000,
            complexity: 1.0,
            noise: 1.0,
        };

        let mut rng_low = rng_from_seed(42);
        let inst_low = gen.generate(&mut rng_low, &low);
        let mut rng_high = rng_from_seed(42);
        let inst_high = gen.generate(&mut rng_high, &high);

        // High difficulty should have more shapes (more data-index attributes)
        if let (RenderPayload::Svg(ref svg_low), RenderPayload::Svg(ref svg_high)) =
            (&inst_low.render_data, &inst_high.render_data)
        {
            let count_low = svg_low.matches("data-index").count();
            let count_high = svg_high.matches("data-index").count();
            assert!(count_high >= count_low);
        }
    }
}
