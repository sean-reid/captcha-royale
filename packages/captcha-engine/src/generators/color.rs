use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct ColorGenerator;

/// Base hue families for the Ishihara-inspired grid
#[derive(Debug, Clone, Copy)]
struct HslColor {
    h: f32, // 0-360
    s: f32, // 0-100
    l: f32, // 0-100
}

impl HslColor {
    fn to_rgb(&self) -> (u8, u8, u8) {
        let h = self.h / 360.0;
        let s = self.s / 100.0;
        let l = self.l / 100.0;

        if s == 0.0 {
            let v = (l * 255.0) as u8;
            return (v, v, v);
        }

        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;

        let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
        let g = hue_to_rgb(p, q, h);
        let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

        ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
    }

    fn css(&self) -> String {
        let (r, g, b) = self.to_rgb();
        format!("rgb({r},{g},{b})")
    }
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

impl CaptchaGenerator for ColorGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        // Grid size: 3x3 to 6x6 based on complexity
        let grid_size = 3 + (difficulty.complexity * 3.0) as u32;
        let grid_size = grid_size.min(6);
        let total_cells = grid_size * grid_size;

        // Pick a base color
        let base_hue = rng.gen_range(0.0..360.0_f32);
        let base_sat = rng.gen_range(50.0..80.0_f32);
        let base_light = rng.gen_range(40.0..60.0_f32);

        // Color difference (delta_e proxy via lightness shift) decreases with difficulty
        // Easy: large shift (15-20), Hard: small shift (4-8)
        let delta = 20.0 - difficulty.complexity * 14.0;
        let delta = delta.max(4.0);

        // Also add a slight hue shift for the odd tile
        let hue_delta = 15.0 - difficulty.complexity * 10.0;
        let hue_delta = hue_delta.max(3.0);

        // Pick the odd tile
        let odd_idx = rng.gen_range(0..total_cells);

        let base_color = HslColor {
            h: base_hue,
            s: base_sat,
            l: base_light,
        };

        let odd_color = HslColor {
            h: base_hue + hue_delta,
            s: base_sat,
            l: base_light + delta,
        };

        let cell_size = 60.0;
        let gap = 4.0;
        let total_width = grid_size as f32 * (cell_size + gap) + gap;
        let total_height = total_width + 40.0; // extra for prompt

        let mut svg = String::with_capacity(2048);
        write!(
            svg,
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {total_width:.0} {total_height:.0}" width="{total_width:.0}" height="{total_height:.0}">"#
        )
        .unwrap();

        // Background
        write!(
            svg,
            r##"<rect width="{total_width:.0}" height="{total_height:.0}" fill="#1a1a2e"/>"##
        )
        .unwrap();

        // Prompt
        write!(
            svg,
            r##"<text x="{:.0}" y="24" font-family="sans-serif" font-size="14" fill="#cccccc" text-anchor="middle">Find the tile with a different shade</text>"##,
            total_width / 2.0
        )
        .unwrap();

        // Render tiles
        for i in 0..total_cells {
            let col = i % grid_size;
            let row = i / grid_size;
            let x = gap + col as f32 * (cell_size + gap);
            let y = 35.0 + gap + row as f32 * (cell_size + gap);

            // Add slight per-tile color variation for noise
            let noise_l = rng.gen_range(-1.5..1.5_f32) * difficulty.noise;
            let noise_h = rng.gen_range(-2.0..2.0_f32) * difficulty.noise;

            let color = if i == odd_idx {
                HslColor {
                    h: odd_color.h + noise_h,
                    s: odd_color.s,
                    l: odd_color.l + noise_l,
                }
            } else {
                HslColor {
                    h: base_color.h + noise_h,
                    s: base_color.s,
                    l: base_color.l + noise_l,
                }
            };

            // Draw as circle (Ishihara-style dots)
            let r = cell_size / 2.0 - 2.0;
            let cx = x + cell_size / 2.0;
            let cy = y + cell_size / 2.0;
            write!(
                svg,
                r#"<circle cx="{cx:.1}" cy="{cy:.1}" r="{r:.1}" fill="{}"/>"#,
                color.css()
            )
            .unwrap();

            // Hidden index label (for interaction)
            write!(
                svg,
                r#"<circle cx="{cx:.1}" cy="{cy:.1}" r="{r:.1}" fill="transparent" data-index="{i}"/>"#
            )
            .unwrap();
        }

        svg.push_str("</svg>");

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::SelectedIndices(vec![odd_idx]),
            expected_solve_time_ms: expected_solve_time(difficulty.time_limit_ms),
            point_value: CaptchaType::ColorPerception.base_points(),
            captcha_type: CaptchaType::ColorPerception,
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
    fn test_color_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.0,
            noise: 0.0,
        };
        let gen = ColorGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert_eq!(instance.captcha_type, CaptchaType::ColorPerception);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert_eq!(indices.len(), 1);
            assert!(indices[0] < 9); // 3x3 grid minimum
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
    fn test_color_seed_determinism() {
        let difficulty = DifficultyParams {
            level: 10,
            round_number: 3,
            time_limit_ms: 7000,
            complexity: 0.5,
            noise: 0.3,
        };
        let gen = ColorGenerator;

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
    fn test_color_correct_answer_validates() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = ColorGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert!(gen.validate(
                &instance,
                &PlayerAnswer::SelectedIndices(indices.clone())
            ));
        }
    }

    #[test]
    fn test_color_wrong_answer_rejects() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = ColorGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![99])));
        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![])));
        assert!(!gen.validate(&instance, &PlayerAnswer::Text("wrong".to_string())));
    }

    #[test]
    fn test_color_difficulty_scaling() {
        let gen = ColorGenerator;
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

        // Higher complexity = larger grid = more SVG content
        if let (RenderPayload::Svg(svg_low), RenderPayload::Svg(svg_high)) =
            (&inst_low.render_data, &inst_high.render_data)
        {
            assert!(svg_high.len() > svg_low.len());
        }

        // Odd tile index range should scale with grid size
        if let Solution::SelectedIndices(ref idx_low) = inst_low.solution {
            assert!(idx_low[0] < 9); // 3x3
        }
        if let Solution::SelectedIndices(ref idx_high) = inst_high.solution {
            assert!(idx_high[0] < 36); // 6x6
        }
    }
}
