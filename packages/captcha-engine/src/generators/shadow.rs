use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct ShadowMatchingGenerator;

#[derive(Debug, Clone)]
enum Primitive {
    Circle { cx: f32, cy: f32, r: f32 },
    Rect { x: f32, y: f32, w: f32, h: f32 },
    Triangle { x: f32, y: f32, size: f32 },
}

impl Primitive {
    fn svg_colored(&self, fill: &str, stroke: &str) -> String {
        match self {
            Primitive::Circle { cx, cy, r } => {
                format!(
                    r##"<circle cx="{cx:.1}" cy="{cy:.1}" r="{r:.1}" fill="{fill}" stroke="{stroke}" stroke-width="1.5"/>"##
                )
            }
            Primitive::Rect { x, y, w, h } => {
                format!(
                    r##"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" fill="{fill}" stroke="{stroke}" stroke-width="1.5"/>"##
                )
            }
            Primitive::Triangle { x, y, size } => {
                let half = size / 2.0;
                // Equilateral triangle pointing up
                let x1 = *x;
                let y1 = y - half * 0.866;
                let x2 = x - half;
                let y2 = y + half * 0.5;
                let x3 = x + half;
                let y3 = y + half * 0.5;
                format!(
                    r##"<polygon points="{x1:.1},{y1:.1} {x2:.1},{y2:.1} {x3:.1},{y3:.1}" fill="{fill}" stroke="{stroke}" stroke-width="1.5"/>"##
                )
            }
        }
    }

    fn svg_silhouette(&self) -> String {
        self.svg_colored("#111111", "none")
    }
}

#[derive(Debug, Clone)]
struct CompoundShape {
    primitives: Vec<Primitive>,
}

impl CompoundShape {
    fn generate(rng: &mut ChaCha8Rng, cx: f32, cy: f32, scale: f32) -> Self {
        let count = rng.gen_range(2..=3);
        let mut primitives = Vec::with_capacity(count);

        for i in 0..count {
            // Offset each primitive slightly from center
            let offset_x = if i == 0 { 0.0 } else { rng.gen_range(-scale * 0.4..scale * 0.4) };
            let offset_y = if i == 0 {
                0.0
            } else {
                rng.gen_range(-scale * 0.5..scale * 0.3)
            };

            let px = cx + offset_x;
            let py = cy + offset_y;
            let psize = scale * rng.gen_range(0.3..0.6);

            let kind = rng.gen_range(0..3);
            let prim = match kind {
                0 => Primitive::Circle {
                    cx: px,
                    cy: py,
                    r: psize,
                },
                1 => Primitive::Rect {
                    x: px - psize,
                    y: py - psize * 0.7,
                    w: psize * 2.0,
                    h: psize * 1.4,
                },
                _ => Primitive::Triangle {
                    x: px,
                    y: py,
                    size: psize * 2.0,
                },
            };
            primitives.push(prim);
        }

        CompoundShape { primitives }
    }

    fn draw_colored(&self, svg: &mut String) {
        let colors = ["#3498db", "#2ecc71", "#e74c3c", "#f39c12", "#9b59b6"];
        for (i, prim) in self.primitives.iter().enumerate() {
            let color = colors[i % colors.len()];
            svg.push_str(&prim.svg_colored(color, "#ffffff"));
        }
    }


    /// Create a distractor by modifying one primitive
    fn make_distractor(&self, rng: &mut ChaCha8Rng, severity: f32) -> CompoundShape {
        let mut new_prims = self.primitives.clone();
        let idx = rng.gen_range(0..new_prims.len());

        let mutation = rng.gen_range(0..3);
        match mutation {
            0 => {
                // Shift position
                let shift = 8.0 + severity * 15.0;
                let dx = rng.gen_range(-shift..shift);
                let dy = rng.gen_range(-shift..shift);
                match &mut new_prims[idx] {
                    Primitive::Circle { cx, cy, .. } => { *cx += dx; *cy += dy; }
                    Primitive::Rect { x, y, .. } => { *x += dx; *y += dy; }
                    Primitive::Triangle { x, y, .. } => { *x += dx; *y += dy; }
                }
            }
            1 => {
                // Resize
                let factor = if severity < 0.5 {
                    rng.gen_range(0.6..0.8)
                } else {
                    rng.gen_range(0.75..0.9)
                };
                let grow = rng.gen_bool(0.5);
                let f = if grow { 1.0 / factor } else { factor };
                match &mut new_prims[idx] {
                    Primitive::Circle { r, .. } => { *r *= f; }
                    Primitive::Rect { w, h, .. } => { *w *= f; *h *= f; }
                    Primitive::Triangle { size, .. } => { *size *= f; }
                }
            }
            _ => {
                // Replace with different shape type
                let (cx, cy, size) = match &new_prims[idx] {
                    Primitive::Circle { cx, cy, r } => (*cx, *cy, *r),
                    Primitive::Rect { x, y, w, h } => (x + w / 2.0, y + h / 2.0, w.min(*h) / 2.0),
                    Primitive::Triangle { x, y, size } => (*x, *y, size / 2.0),
                };
                let new_kind = rng.gen_range(0..3);
                new_prims[idx] = match new_kind {
                    0 => Primitive::Circle { cx, cy, r: size },
                    1 => Primitive::Rect { x: cx - size, y: cy - size * 0.7, w: size * 2.0, h: size * 1.4 },
                    _ => Primitive::Triangle { x: cx, y: cy, size: size * 2.0 },
                };
            }
        }

        CompoundShape { primitives: new_prims }
    }
}

impl CaptchaGenerator for ShadowMatchingGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        let option_count = if difficulty.complexity < 0.4 {
            3
        } else if difficulty.complexity < 0.7 {
            4
        } else {
            5
        };

        let width = 400.0_f32;
        let shape_area_height = 150.0_f32;
        let option_size = 80.0_f32;
        let total_height = shape_area_height + option_size + 90.0;

        // Generate the compound shape centered in the top area
        let shape_cx = width / 2.0;
        let shape_cy = 40.0 + shape_area_height / 2.0;
        let shape_scale = 45.0;
        let correct_shape = CompoundShape::generate(rng, shape_cx, shape_cy, shape_scale);

        // Generate distractors
        let correct_idx = rng.gen_range(0..option_count);
        let distractor_severity = 1.0 - difficulty.complexity * 0.6; // harder = subtler changes

        let mut options: Vec<CompoundShape> = Vec::with_capacity(option_count);
        for i in 0..option_count {
            if i == correct_idx {
                options.push(correct_shape.clone());
            } else {
                options.push(correct_shape.make_distractor(rng, distractor_severity));
            }
        }

        let mut svg = String::with_capacity(8192);
        write!(
            svg,
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width:.0} {total_height:.0}" width="{width:.0}" height="{total_height:.0}">"##
        ).unwrap();

        // Background
        write!(svg, r##"<rect width="{width:.0}" height="{total_height:.0}" fill="#1a1a2e"/>"##).unwrap();

        // Prompt
        write!(
            svg,
            r##"<text x="{:.0}" y="22" font-family="sans-serif" font-size="14" fill="#cccccc" text-anchor="middle">Which silhouette matches the shape above?</text>"##,
            width / 2.0
        ).unwrap();

        // Noise
        let noise_count = (difficulty.noise * 6.0) as u32;
        for _ in 0..noise_count {
            let x1 = rng.gen_range(0.0..width);
            let y1 = rng.gen_range(30.0..shape_area_height + 40.0);
            let x2 = rng.gen_range(0.0..width);
            let y2 = rng.gen_range(30.0..shape_area_height + 40.0);
            let gray = rng.gen_range(30..80);
            write!(
                svg,
                r##"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="rgb({gray},{gray},{gray})" stroke-width="1" opacity="0.2"/>"##
            ).unwrap();
        }

        // Draw the colored compound shape
        correct_shape.draw_colored(&mut svg);

        // Separator line
        let sep_y = shape_area_height + 40.0;
        write!(
            svg,
            r##"<line x1="20" y1="{sep_y:.0}" x2="{:.0}" y2="{sep_y:.0}" stroke="#444" stroke-width="1"/>"##,
            width - 20.0
        ).unwrap();

        // Label
        let label_y = sep_y + 16.0;
        write!(
            svg,
            r##"<text x="{:.0}" y="{label_y:.0}" font-family="sans-serif" font-size="12" fill="#999" text-anchor="middle">Select the matching silhouette:</text>"##,
            width / 2.0
        ).unwrap();

        // Draw silhouette options
        let opts_top = label_y + 12.0;
        let option_spacing = width / (option_count as f32 + 1.0);

        for (i, option_shape) in options.iter().enumerate().take(option_count) {
            let opt_cx = option_spacing * (i as f32 + 1.0);
            let opt_cy = opts_top + option_size / 2.0;

            // Scale the shape to fit in the option area
            let scale_factor = option_size / (shape_area_height * 0.7);
            for prim in &option_shape.primitives {
                let shifted = match prim {
                    Primitive::Circle { cx, cy, r } => {
                        let ncx = opt_cx + (*cx - shape_cx) * scale_factor;
                        let ncy = opt_cy + (*cy - shape_cy) * scale_factor;
                        Primitive::Circle { cx: ncx, cy: ncy, r: r * scale_factor }
                    }
                    Primitive::Rect { x, y, w, h } => {
                        // Center of rect
                        let rcx = x + w / 2.0;
                        let rcy = y + h / 2.0;
                        let ncx = opt_cx + (rcx - shape_cx) * scale_factor;
                        let ncy = opt_cy + (rcy - shape_cy) * scale_factor;
                        let nw = w * scale_factor;
                        let nh = h * scale_factor;
                        Primitive::Rect { x: ncx - nw / 2.0, y: ncy - nh / 2.0, w: nw, h: nh }
                    }
                    Primitive::Triangle { x, y, size } => {
                        let nx = opt_cx + (*x - shape_cx) * scale_factor;
                        let ny = opt_cy + (*y - shape_cy) * scale_factor;
                        Primitive::Triangle { x: nx, y: ny, size: size * scale_factor }
                    }
                };
                svg.push_str(&shifted.svg_silhouette());
            }

            // Option label
            write!(
                svg,
                r##"<text x="{opt_cx:.1}" y="{:.1}" font-family="sans-serif" font-size="11" fill="#888" text-anchor="middle">{}</text>"##,
                opts_top + option_size + 14.0,
                (b'A' + i as u8) as char
            ).unwrap();

            // Clickable overlay with data-index
            let overlay_x = opt_cx - option_size / 2.0;
            write!(
                svg,
                r##"<rect x="{overlay_x:.1}" y="{opts_top:.1}" width="{option_size:.0}" height="{option_size:.0}" fill="transparent" data-index="{i}" style="cursor:pointer"/>"##
            ).unwrap();
        }

        svg.push_str("</svg>");

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::SelectedIndices(vec![correct_idx as u32]),
            expected_solve_time_ms: expected_solve_time(difficulty.time_limit_ms),
            point_value: CaptchaType::AdversarialImage.base_points(),
            captcha_type: CaptchaType::AdversarialImage,
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
    fn test_shadow_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 8000,
            complexity: 0.0,
            noise: 0.0,
        };
        let gen = ShadowMatchingGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert_eq!(instance.captcha_type, CaptchaType::AdversarialImage);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert_eq!(indices.len(), 1);
            assert!(indices[0] < 5);
        } else {
            panic!("Expected SelectedIndices solution");
        }
        if let RenderPayload::Svg(ref s) = instance.render_data {
            assert!(s.contains("<svg"));
            assert!(s.contains("data-index"));
            assert!(s.contains("Which silhouette"));
        } else {
            panic!("Expected SVG render data");
        }
    }

    #[test]
    fn test_shadow_seed_determinism() {
        let difficulty = DifficultyParams {
            level: 10,
            round_number: 3,
            time_limit_ms: 9000,
            complexity: 0.5,
            noise: 0.3,
        };
        let gen = ShadowMatchingGenerator;

        let mut rng1 = rng_from_seed(12345);
        let inst1 = gen.generate(&mut rng1, &difficulty);

        let mut rng2 = rng_from_seed(12345);
        let inst2 = gen.generate(&mut rng2, &difficulty);

        if let (Solution::SelectedIndices(i1), Solution::SelectedIndices(i2)) =
            (&inst1.solution, &inst2.solution)
        {
            assert_eq!(i1, i2);
        }
    }

    #[test]
    fn test_shadow_correct_answer_validates() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 8000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = ShadowMatchingGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert!(gen.validate(
                &instance,
                &PlayerAnswer::SelectedIndices(indices.clone())
            ));
        }
    }

    #[test]
    fn test_shadow_wrong_answer_rejects() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 8000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = ShadowMatchingGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![99])));
        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![])));
        assert!(!gen.validate(&instance, &PlayerAnswer::Text("wrong".to_string())));
    }
}
