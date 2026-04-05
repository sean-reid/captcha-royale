use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct JigsawFitGenerator;

/// A piece shape defined as a polygon (list of (x,y) offsets from center)
#[derive(Clone)]
struct PieceShape {
    points: Vec<(f32, f32)>,
}

impl PieceShape {
    fn render(&self, cx: f32, cy: f32, fill: &str, extra_attrs: &str) -> String {
        let pts: Vec<String> = self
            .points
            .iter()
            .map(|(px, py)| format!("{:.1},{:.1}", cx + px, cy + py))
            .collect();
        format!(
            r##"<polygon points="{}" fill="{}" {}/>"##,
            pts.join(" "),
            fill,
            extra_attrs
        )
    }

    /// Create a rotated copy (angle in radians)
    fn rotated(&self, angle: f32) -> PieceShape {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        PieceShape {
            points: self
                .points
                .iter()
                .map(|(x, y)| (x * cos_a - y * sin_a, x * sin_a + y * cos_a))
                .collect(),
        }
    }

    /// Create a horizontally flipped copy
    fn flipped(&self) -> PieceShape {
        PieceShape {
            points: self.points.iter().map(|(x, y)| (-x, *y)).collect(),
        }
    }

    /// Create a scaled copy
    fn scaled(&self, factor: f32) -> PieceShape {
        PieceShape {
            points: self
                .points
                .iter()
                .map(|(x, y)| (x * factor, y * factor))
                .collect(),
        }
    }
}

/// Asymmetric piece templates — rotation and flip produce visibly different shapes
fn piece_templates() -> Vec<PieceShape> {
    vec![
        // L-shape
        PieceShape {
            points: vec![
                (-15.0, -20.0), (0.0, -20.0), (0.0, 0.0),
                (20.0, 0.0), (20.0, 15.0), (-15.0, 15.0),
            ],
        },
        // T-shape
        PieceShape {
            points: vec![
                (-20.0, -10.0), (20.0, -10.0), (20.0, 5.0), (8.0, 5.0),
                (8.0, 20.0), (-8.0, 20.0), (-8.0, 5.0), (-20.0, 5.0),
            ],
        },
        // Arrow/chevron
        PieceShape {
            points: vec![
                (0.0, -22.0), (18.0, 0.0), (10.0, 0.0),
                (10.0, 18.0), (-10.0, 18.0), (-10.0, 0.0), (-18.0, 0.0),
            ],
        },
        // Parallelogram
        PieceShape {
            points: vec![
                (-10.0, -15.0), (15.0, -15.0), (10.0, 15.0), (-15.0, 15.0),
            ],
        },
        // Z/S-shape
        PieceShape {
            points: vec![
                (-18.0, -15.0), (2.0, -15.0), (2.0, -2.0), (18.0, -2.0),
                (18.0, 15.0), (-2.0, 15.0), (-2.0, 2.0), (-18.0, 2.0),
            ],
        },
        // P-shape (flag)
        PieceShape {
            points: vec![
                (-12.0, -20.0), (16.0, -20.0), (16.0, 0.0),
                (-2.0, 0.0), (-2.0, 18.0), (-12.0, 18.0),
            ],
        },
    ]
}

impl CaptchaGenerator for JigsawFitGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        let c = difficulty.complexity;
        let option_count = if c < 0.3 { 3 } else if c < 0.6 { 4 } else { 5 };

        let templates = piece_templates();

        // Pick a template for the correct piece
        let template_idx = rng.gen_range(0..templates.len());
        let correct_piece = templates[template_idx].scaled(1.2);

        // Pick a random rotation for the correct piece
        let correct_angle = rng.gen_range(0..4) as f32 * std::f32::consts::FRAC_PI_2;
        let correct_rotated = correct_piece.rotated(correct_angle);

        // Board dimensions
        let board_w = 200.0_f32;
        let board_h = 160.0_f32;
        let hole_cx = board_w / 2.0;
        let hole_cy = board_h / 2.0;

        let board_hue = rng.gen_range(0..360);
        let board_fill = format!("hsl({board_hue}, 35%, 35%)");
        let piece_fill = format!("hsl({board_hue}, 45%, 50%)");

        let width = 460.0_f32;
        let height = 340.0_f32;
        let board_x = (width - board_w) / 2.0;
        let board_y = 40.0;

        let mut svg = String::with_capacity(4096);
        write!(svg,
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width:.0} {height:.0}" width="{width:.0}" height="{height:.0}">"##
        ).unwrap();
        write!(svg, r##"<rect width="{width:.0}" height="{height:.0}" fill="#1a1a2e"/>"##).unwrap();

        // Prompt
        write!(svg,
            r##"<text x="{:.0}" y="25" font-family="sans-serif" font-size="14" fill="#cccccc" text-anchor="middle">Which piece fits the hole?</text>"##,
            width / 2.0
        ).unwrap();

        // Draw the board
        write!(svg,
            r##"<rect x="{board_x:.0}" y="{board_y:.0}" width="{board_w:.0}" height="{board_h:.0}" fill="{board_fill}" rx="6"/>"##
        ).unwrap();

        // Draw some board texture (small shapes)
        let detail_count = 6 + (c * 6.0) as u32;
        let hole_abs_cx = board_x + hole_cx;
        let hole_abs_cy = board_y + hole_cy;

        for _ in 0..detail_count {
            let dx = board_x + rng.gen_range(15.0..board_w - 15.0);
            let dy = board_y + rng.gen_range(15.0..board_h - 15.0);
            let dr = rng.gen_range(3.0..8.0_f32);
            let dh = (board_hue + rng.gen_range(-30..30_i32) + 360) % 360;
            let dist = ((dx - hole_abs_cx).powi(2) + (dy - hole_abs_cy).powi(2)).sqrt();
            if dist > 35.0 {
                write!(svg,
                    r##"<circle cx="{dx:.0}" cy="{dy:.0}" r="{dr:.0}" fill="hsl({dh}, 30%, 30%)" opacity="0.5"/>"##
                ).unwrap();
            }
        }

        // Draw the hole (dark background + dashed outline)
        svg.push_str(&correct_rotated.render(
            hole_abs_cx, hole_abs_cy, "#1a1a2e",
            r##"stroke="#999" stroke-width="2" stroke-dasharray="4,3""##,
        ));

        // Options row
        let opt_y = board_y + board_h + 35.0;
        let opt_cell = 60.0_f32;
        let opt_gap = 12.0;
        let opt_total_w = option_count as f32 * opt_cell + (option_count as f32 - 1.0) * opt_gap;
        let opt_start_x = (width - opt_total_w) / 2.0;

        write!(svg,
            r##"<text x="{:.0}" y="{:.0}" font-family="sans-serif" font-size="12" fill="#999" text-anchor="middle">Select the correct piece:</text>"##,
            width / 2.0, opt_y - 10.0
        ).unwrap();

        let correct_idx = rng.gen_range(0..option_count);

        // Build list of wrong shape indices (different templates only, no rotations of correct)
        let mut wrong_templates: Vec<usize> = (0..templates.len())
            .filter(|&i| i != template_idx)
            .collect();
        // Shuffle wrong templates
        for i in (1..wrong_templates.len()).rev() {
            let j = rng.gen_range(0..=i);
            wrong_templates.swap(i, j);
        }
        let mut wrong_idx = 0;

        for i in 0..option_count {
            let cx = opt_start_x + i as f32 * (opt_cell + opt_gap) + opt_cell / 2.0;
            let cy = opt_y + opt_cell / 2.0;

            // Cell background
            write!(svg,
                r##"<rect x="{:.0}" y="{opt_y:.0}" width="{opt_cell:.0}" height="{opt_cell:.0}" fill="#222244" stroke="#444" stroke-width="1" rx="4"/>"##,
                cx - opt_cell / 2.0
            ).unwrap();

            if i == correct_idx {
                // Show correct piece at a DIFFERENT rotation — player must mentally rotate
                let display_angle = correct_angle
                    + (rng.gen_range(1..=3) as f32) * std::f32::consts::FRAC_PI_2;
                let display_piece = correct_piece.rotated(display_angle);
                svg.push_str(&display_piece.render(cx, cy, &piece_fill, ""));
            } else {
                // Wrong pieces: always a DIFFERENT shape (never a rotation of correct)
                let ti = wrong_templates[wrong_idx % wrong_templates.len()];
                wrong_idx += 1;
                let rand_angle = rng.gen_range(0..4) as f32 * std::f32::consts::FRAC_PI_2;
                let wrong_piece = templates[ti].scaled(1.2).rotated(rand_angle);
                svg.push_str(&wrong_piece.render(cx, cy, &piece_fill, ""));
            }

            // Clickable overlay
            write!(svg,
                r##"<rect x="{:.0}" y="{opt_y:.0}" width="{opt_cell:.0}" height="{opt_cell:.0}" fill="transparent" data-index="{i}" style="cursor:pointer"/>"##,
                cx - opt_cell / 2.0
            ).unwrap();

            // Label
            write!(svg,
                r##"<text x="{cx:.0}" y="{:.0}" font-family="sans-serif" font-size="11" fill="#888" text-anchor="middle">{}</text>"##,
                opt_y + opt_cell + 14.0,
                (b'A' + i as u8) as char
            ).unwrap();
        }

        // Noise
        let noise_count = (2.0 + difficulty.noise * 6.0) as u32;
        for _ in 0..noise_count {
            let x1 = rng.gen_range(0.0..width);
            let y1 = rng.gen_range(30.0..height);
            let x2 = rng.gen_range(0.0..width);
            let y2 = rng.gen_range(30.0..height);
            write!(svg,
                r##"<line x1="{x1:.0}" y1="{y1:.0}" x2="{x2:.0}" y2="{y2:.0}" stroke="#444" stroke-width="1" opacity="0.1"/>"##
            ).unwrap();
        }

        svg.push_str("</svg>");

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::SelectedIndices(vec![correct_idx as u32]),
            expected_solve_time_ms: expected_solve_time(difficulty.time_limit_ms),
            point_value: CaptchaType::PartialOcclusion.base_points(),
            captcha_type: CaptchaType::PartialOcclusion,
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
    fn test_jigsaw_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1, round_number: 1, time_limit_ms: 6000,
            complexity: 0.0, noise: 0.0,
        };
        let gen = JigsawFitGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert_eq!(instance.captcha_type, CaptchaType::PartialOcclusion);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert_eq!(indices.len(), 1);
        } else {
            panic!("Expected SelectedIndices solution");
        }
        if let RenderPayload::Svg(ref s) = instance.render_data {
            assert!(s.contains("<svg"));
            assert!(s.contains("data-index"));
        } else {
            panic!("Expected SVG render data");
        }
    }

    #[test]
    fn test_jigsaw_seed_determinism() {
        let difficulty = DifficultyParams {
            level: 10, round_number: 3, time_limit_ms: 8000,
            complexity: 0.5, noise: 0.3,
        };
        let gen = JigsawFitGenerator;
        let mut rng1 = rng_from_seed(12345);
        let inst1 = gen.generate(&mut rng1, &difficulty);
        let mut rng2 = rng_from_seed(12345);
        let inst2 = gen.generate(&mut rng2, &difficulty);
        if let (Solution::SelectedIndices(i1), Solution::SelectedIndices(i2)) =
            (&inst1.solution, &inst2.solution) {
            assert_eq!(i1, i2);
        }
    }

    #[test]
    fn test_jigsaw_correct_answer_validates() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5, round_number: 1, time_limit_ms: 6000,
            complexity: 0.2, noise: 0.1,
        };
        let gen = JigsawFitGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert!(gen.validate(&instance, &PlayerAnswer::SelectedIndices(indices.clone())));
        }
    }

    #[test]
    fn test_jigsaw_wrong_answer_rejects() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5, round_number: 1, time_limit_ms: 6000,
            complexity: 0.2, noise: 0.1,
        };
        let gen = JigsawFitGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![99])));
        assert!(!gen.validate(&instance, &PlayerAnswer::Text("wrong".to_string())));
    }
}
