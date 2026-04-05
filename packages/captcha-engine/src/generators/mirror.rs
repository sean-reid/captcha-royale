use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct MirrorGenerator;

/// An asymmetric polygon shape defined by relative points
#[derive(Debug, Clone)]
struct Shape {
    /// Points as (x, y) offsets from center, normalized to -1..1
    points: Vec<(f32, f32)>,
}

impl Shape {
    fn render(&self, cx: f32, cy: f32, scale: f32, fill: &str, mirror: bool) -> String {
        let pts: Vec<String> = self
            .points
            .iter()
            .map(|(x, y)| {
                let mx = if mirror { -x } else { *x };
                format!("{:.1},{:.1}", cx + mx * scale, cy + y * scale)
            })
            .collect();
        format!(
            r##"<polygon points="{}" fill="{fill}" stroke="#aaa" stroke-width="1"/>"##,
            pts.join(" ")
        )
    }

    fn render_rotated(&self, cx: f32, cy: f32, scale: f32, fill: &str, angle_deg: f32) -> String {
        let pts: Vec<String> = self
            .points
            .iter()
            .map(|(x, y)| format!("{:.1},{:.1}", cx + x * scale, cy + y * scale))
            .collect();
        format!(
            r##"<polygon points="{}" fill="{fill}" stroke="#aaa" stroke-width="1" transform="rotate({angle_deg:.1},{cx:.1},{cy:.1})"/>"##,
            pts.join(" ")
        )
    }

    fn render_distorted(
        &self,
        cx: f32,
        cy: f32,
        scale: f32,
        fill: &str,
        rng: &mut ChaCha8Rng,
        amount: f32,
    ) -> String {
        let pts: Vec<String> = self
            .points
            .iter()
            .map(|(x, y)| {
                let dx = rng.gen_range(-amount..amount);
                let dy = rng.gen_range(-amount..amount);
                format!(
                    "{:.1},{:.1}",
                    cx + (x + dx) * scale,
                    cy + (y + dy) * scale
                )
            })
            .collect();
        format!(
            r##"<polygon points="{}" fill="{fill}" stroke="#aaa" stroke-width="1"/>"##,
            pts.join(" ")
        )
    }
}

fn generate_asymmetric_shape(rng: &mut ChaCha8Rng, variant: u32) -> Shape {
    let points = match variant % 6 {
        0 => {
            // Arrow pointing right
            vec![
                (0.0, -0.6),
                (0.6, 0.0),
                (0.0, 0.6),
                (0.0, 0.2),
                (-0.7, 0.2),
                (-0.7, -0.2),
                (0.0, -0.2),
            ]
        }
        1 => {
            // L-shape
            vec![
                (-0.6, -0.7),
                (-0.2, -0.7),
                (-0.2, 0.0),
                (0.6, 0.0),
                (0.6, 0.4),
                (-0.6, 0.4),
            ]
        }
        2 => {
            // Irregular pentagon (flag-like)
            vec![
                (-0.5, -0.6),
                (0.7, -0.3),
                (0.4, 0.1),
                (0.7, 0.5),
                (-0.5, 0.5),
            ]
        }
        3 => {
            // Hook shape
            vec![
                (-0.3, -0.7),
                (0.3, -0.7),
                (0.3, -0.1),
                (0.6, -0.1),
                (0.6, 0.3),
                (-0.1, 0.3),
                (-0.1, 0.6),
                (-0.5, 0.6),
                (-0.5, -0.1),
                (-0.3, -0.1),
            ]
        }
        4 => {
            // Asymmetric triangle
            vec![(-0.6, 0.5), (0.0, -0.7), (0.7, 0.3)]
        }
        _ => {
            // Zigzag / lightning bolt
            vec![
                (-0.2, -0.7),
                (0.3, -0.7),
                (0.0, -0.1),
                (0.4, -0.1),
                (-0.1, 0.7),
                (-0.1, 0.1),
                (-0.5, 0.1),
            ]
        }
    };

    // Add slight random jitter to make each instance unique
    let points = points
        .into_iter()
        .map(|(x, y)| {
            let jx = rng.gen_range(-0.05..0.05_f32);
            let jy = rng.gen_range(-0.05..0.05_f32);
            (x + jx, y + jy)
        })
        .collect();

    Shape { points }
}

impl CaptchaGenerator for MirrorGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        let option_count = if difficulty.complexity < 0.5 {
            3
        } else if difficulty.complexity < 0.8 {
            4
        } else {
            5
        };

        let variant = rng.gen_range(0..6_u32);
        let shape = generate_asymmetric_shape(rng, variant);

        let correct_idx = rng.gen_range(0..option_count as u32);

        // Layout constants
        let ref_size = 80.0;
        let opt_size = 60.0;
        let ref_area_w = 140.0;
        let opt_area_w = option_count as f32 * (opt_size + 15.0) + 15.0;
        let total_w = ref_area_w + 10.0 + opt_area_w;
        let total_h = 160.0;

        let hue = rng.gen_range(0..360);
        let fill_ref = format!("hsl({hue},60%,55%)");
        let fill_opt = format!("hsl({hue},60%,55%)");

        let mut svg = String::with_capacity(4096);
        write!(
            svg,
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {total_w:.0} {total_h:.0}" width="{total_w:.0}" height="{total_h:.0}">"##
        )
        .unwrap();
        write!(
            svg,
            r##"<rect width="{total_w:.0}" height="{total_h:.0}" fill="#1a1a2e"/>"##
        )
        .unwrap();

        // Prompt
        write!(
            svg,
            r##"<text x="{:.0}" y="20" font-family="sans-serif" font-size="13" fill="#cccccc" text-anchor="middle">Select the horizontal mirror of the shape on the left</text>"##,
            total_w / 2.0
        )
        .unwrap();

        // Reference shape
        let ref_cx = ref_area_w / 2.0;
        let ref_cy = 95.0;
        write!(
            svg,
            r##"<rect x="5" y="30" width="{:.0}" height="120" rx="6" fill="#222244" stroke="#555" stroke-width="1"/>"##,
            ref_area_w - 10.0
        )
        .unwrap();
        write!(
            svg,
            r##"<text x="{ref_cx:.0}" y="48" font-family="sans-serif" font-size="11" fill="#888" text-anchor="middle">Reference</text>"##
        )
        .unwrap();
        svg.push_str(&shape.render(ref_cx, ref_cy, ref_size / 2.0, &fill_ref, false));

        // Divider
        let div_x = ref_area_w + 5.0;
        write!(
            svg,
            r##"<line x1="{div_x:.0}" y1="30" x2="{div_x:.0}" y2="150" stroke="#555" stroke-width="1" stroke-dasharray="4,3"/>"##
        )
        .unwrap();

        // Options
        let opt_start_x = ref_area_w + 10.0;
        for i in 0..option_count {
            let cx = opt_start_x + 15.0 + i as f32 * (opt_size + 15.0) + opt_size / 2.0;
            let cy = 95.0;

            // Option background
            write!(
                svg,
                r##"<rect x="{:.0}" y="55" width="{opt_size:.0}" height="{opt_size:.0}" rx="4" fill="#1e1e3a" stroke="#444" stroke-width="1"/>"##,
                cx - opt_size / 2.0
            )
            .unwrap();

            if i as u32 == correct_idx {
                // Correct: horizontal mirror
                svg.push_str(&shape.render(cx, cy, opt_size / 2.5, &fill_opt, true));
            } else {
                // Distractor: rotation or distortion
                let distractor_type = rng.gen_range(0..3_u32);
                match distractor_type {
                    0 => {
                        // Rotated version (not mirrored)
                        let angle = rng.gen_range(30.0..330.0_f32);
                        svg.push_str(&shape.render_rotated(
                            cx,
                            cy,
                            opt_size / 2.5,
                            &fill_opt,
                            angle,
                        ));
                    }
                    1 => {
                        // Vertical mirror instead of horizontal
                        let pts: Vec<String> = shape
                            .points
                            .iter()
                            .map(|(x, y)| {
                                format!(
                                    "{:.1},{:.1}",
                                    cx + x * (opt_size / 2.5),
                                    cy + (-y) * (opt_size / 2.5)
                                )
                            })
                            .collect();
                        write!(
                            svg,
                            r##"<polygon points="{}" fill="{fill_opt}" stroke="#aaa" stroke-width="1"/>"##,
                            pts.join(" ")
                        )
                        .unwrap();
                    }
                    _ => {
                        // Distorted version
                        let amount = 0.1 + difficulty.complexity * 0.15;
                        svg.push_str(&shape.render_distorted(
                            cx,
                            cy,
                            opt_size / 2.5,
                            &fill_opt,
                            rng,
                            amount,
                        ));
                    }
                }
            }

            // Clickable overlay
            write!(
                svg,
                r##"<rect x="{:.0}" y="55" width="{opt_size:.0}" height="{opt_size:.0}" fill="transparent" data-index="{i}" style="cursor:pointer"/>"##,
                cx - opt_size / 2.0
            )
            .unwrap();
        }

        // Add noise at high complexity
        let noise_count = (difficulty.noise * 10.0) as u32;
        for _ in 0..noise_count {
            let x1 = rng.gen_range(0.0..total_w);
            let y1 = rng.gen_range(30.0..total_h);
            let x2 = rng.gen_range(0.0..total_w);
            let y2 = rng.gen_range(30.0..total_h);
            write!(
                svg,
                r##"<line x1="{x1:.0}" y1="{y1:.0}" x2="{x2:.0}" y2="{y2:.0}" stroke="#555" stroke-width="1" opacity="0.15"/>"##
            )
            .unwrap();
        }

        svg.push_str("</svg>");

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::SelectedIndices(vec![correct_idx]),
            expected_solve_time_ms: expected_solve_time(difficulty.time_limit_ms),
            point_value: CaptchaType::MirrorMatch.base_points(),
            captcha_type: CaptchaType::MirrorMatch,
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
    fn test_mirror_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.0,
            noise: 0.0,
        };
        let gen = MirrorGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert_eq!(instance.captcha_type, CaptchaType::MirrorMatch);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert_eq!(indices.len(), 1);
            assert!(indices[0] < 3); // low complexity = 3 options
        } else {
            panic!("Expected SelectedIndices solution");
        }
        if let RenderPayload::Svg(ref s) = instance.render_data {
            assert!(s.contains("<svg"));
            assert!(s.contains("data-index"));
            assert!(s.contains("Reference"));
        } else {
            panic!("Expected SVG render data");
        }
    }

    #[test]
    fn test_mirror_seed_determinism() {
        let difficulty = DifficultyParams {
            level: 10,
            round_number: 3,
            time_limit_ms: 7000,
            complexity: 0.5,
            noise: 0.3,
        };
        let gen = MirrorGenerator;

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
    fn test_mirror_correct_answer_validates() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = MirrorGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert!(gen.validate(
                &instance,
                &PlayerAnswer::SelectedIndices(indices.clone())
            ));
        }
    }

    #[test]
    fn test_mirror_wrong_answer_rejects() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = MirrorGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![99])));
        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![])));
        assert!(!gen.validate(&instance, &PlayerAnswer::Text("wrong".to_string())));
    }
}
