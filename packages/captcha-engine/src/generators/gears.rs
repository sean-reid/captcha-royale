use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct GearsGenerator;

/// Render a gear as a circle with teeth (zigzag outer edge)
fn render_gear(cx: f32, cy: f32, radius: f32, teeth: u32, fill: &str) -> String {
    let mut svg = String::with_capacity(512);

    let inner_r = radius * 0.75;
    let outer_r = radius;
    let tooth_count = teeth;

    // Generate gear path as polygon points
    let mut points = Vec::new();
    for i in 0..(tooth_count * 2) {
        let angle = (i as f32 / (tooth_count * 2) as f32) * std::f32::consts::TAU;
        let r = if i % 2 == 0 { outer_r } else { inner_r };
        let px = cx + angle.cos() * r;
        let py = cy + angle.sin() * r;
        points.push(format!("{px:.1},{py:.1}"));
    }

    write!(
        svg,
        r##"<polygon points="{}" fill="{fill}" stroke="#666" stroke-width="1.5"/>"##,
        points.join(" ")
    )
    .unwrap();

    // Center hub
    write!(
        svg,
        r##"<circle cx="{cx:.0}" cy="{cy:.0}" r="{:.0}" fill="#333" stroke="#555" stroke-width="1"/>"##,
        radius * 0.25
    )
    .unwrap();

    svg
}

/// Render a rotation arrow on a gear
fn render_arrow(cx: f32, cy: f32, radius: f32, clockwise: bool) -> String {
    let mut svg = String::with_capacity(256);

    // Draw a curved arrow arc
    let r = radius * 0.5;
    let start_angle = if clockwise {
        -std::f32::consts::FRAC_PI_4
    } else {
        std::f32::consts::FRAC_PI_4
    };
    let sweep_angle = if clockwise {
        std::f32::consts::PI
    } else {
        -std::f32::consts::PI
    };
    let end_angle = start_angle + sweep_angle;

    let x1 = cx + start_angle.cos() * r;
    let y1 = cy + start_angle.sin() * r;
    let x2 = cx + end_angle.cos() * r;
    let y2 = cy + end_angle.sin() * r;

    let sweep_flag = if clockwise { 1 } else { 0 };

    write!(
        svg,
        r##"<path d="M {x1:.1} {y1:.1} A {r:.1} {r:.1} 0 0 {sweep_flag} {x2:.1} {y2:.1}" fill="none" stroke="#ffcc00" stroke-width="2.5" stroke-linecap="round"/>"##
    )
    .unwrap();

    // Arrowhead at end
    let arrow_size = 6.0;
    let tangent_angle = if clockwise {
        end_angle + std::f32::consts::FRAC_PI_2
    } else {
        end_angle - std::f32::consts::FRAC_PI_2
    };
    let ax1 = x2 + (tangent_angle + 2.5).cos() * arrow_size;
    let ay1 = y2 + (tangent_angle + 2.5).sin() * arrow_size;
    let ax2 = x2 + (tangent_angle - 2.5).cos() * arrow_size;
    let ay2 = y2 + (tangent_angle - 2.5).sin() * arrow_size;

    write!(
        svg,
        r##"<polygon points="{x2:.1},{y2:.1} {ax1:.1},{ay1:.1} {ax2:.1},{ay2:.1}" fill="#ffcc00"/>"##
    )
    .unwrap();

    svg
}

impl CaptchaGenerator for GearsGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        // Gear count: 2-3 at low complexity, 4-5 at high
        let gear_count = if difficulty.complexity < 0.4 {
            rng.gen_range(2..=3)
        } else if difficulty.complexity < 0.7 {
            rng.gen_range(3..=4)
        } else {
            rng.gen_range(4..=5)
        };

        // First gear is always CW; each subsequent gear alternates
        // Last gear direction: if gear_count is odd => CW (index 0), if even => CCW (index 1)
        let last_gear_cw = gear_count % 2 == 1;
        let correct_answer: u32 = if last_gear_cw { 0 } else { 1 };

        // Layout: gears in a horizontal chain — scale radius to fit within 400px
        let max_width = 400.0_f32;
        let gear_radius = ((max_width - 80.0) / (gear_count as f32 * 1.7 + 0.3)).min(35.0);
        let teeth = 12_u32;
        let gear_spacing = gear_radius * 1.7;
        let total_gears_w = (gear_count as f32 - 1.0) * gear_spacing + gear_radius * 2.0;
        let width = max_width;
        let button_area_h = 50.0;
        let height = 200.0 + button_area_h;

        let mut svg = String::with_capacity(4096);
        write!(
            svg,
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width:.0} {height:.0}" width="{width:.0}" height="{height:.0}">"##
        )
        .unwrap();
        write!(
            svg,
            r##"<rect width="{width:.0}" height="{height:.0}" fill="#1a1a2e"/>"##
        )
        .unwrap();

        // Prompt
        write!(
            svg,
            r##"<text x="{:.0}" y="22" font-family="sans-serif" font-size="13" fill="#cccccc" text-anchor="middle">The first gear rotates clockwise. Which way does the last gear rotate?</text>"##,
            width / 2.0
        )
        .unwrap();

        let start_x = (width - total_gears_w) / 2.0 + gear_radius;
        let gear_cy = 110.0;

        // Color palette for gears
        let hues = [200, 140, 280, 30, 330];

        for i in 0..gear_count {
            let cx = start_x + i as f32 * gear_spacing;
            let cy = gear_cy;
            let hue = hues[i % hues.len()];
            let fill = format!("hsl({hue},45%,40%)");

            svg.push_str(&render_gear(cx, cy, gear_radius, teeth, &fill));

            // Arrow on first gear (CW) and label on last gear with "?"
            if i == 0 {
                svg.push_str(&render_arrow(cx, cy, gear_radius, true));
                write!(
                    svg,
                    r##"<text x="{cx:.0}" y="{:.0}" font-family="sans-serif" font-size="10" fill="#888" text-anchor="middle">1st</text>"##,
                    cy + gear_radius + 15.0
                )
                .unwrap();
            }
            if i == gear_count - 1 {
                write!(
                    svg,
                    r##"<text x="{cx:.0}" y="{:.0}" font-family="sans-serif" font-size="16" font-weight="bold" fill="#ffcc00" text-anchor="middle">?</text>"##,
                    cy + gear_radius + 18.0
                )
                .unwrap();
            }

            // Draw mesh indicator between gears
            if i > 0 {
                let prev_cx = start_x + (i - 1) as f32 * gear_spacing;
                let mid_x = (prev_cx + cx) / 2.0;
                write!(
                    svg,
                    r##"<circle cx="{mid_x:.0}" cy="{cy:.0}" r="3" fill="#666" opacity="0.5"/>"##
                )
                .unwrap();
            }
        }

        // Answer buttons at bottom
        let btn_w = 120.0;
        let btn_h = 32.0;
        let btn_y = height - button_area_h + 8.0;
        let btn_gap = 20.0;
        let btn_total_w = btn_w * 2.0 + btn_gap;
        let btn_start_x = (width - btn_total_w) / 2.0;

        // CW button (data-index="0")
        write!(
            svg,
            r##"<rect x="{btn_start_x:.0}" y="{btn_y:.0}" width="{btn_w:.0}" height="{btn_h:.0}" rx="6" fill="#2a3a5a" stroke="#5577aa" stroke-width="1.5"/>"##
        )
        .unwrap();
        write!(
            svg,
            r##"<text x="{:.0}" y="{:.0}" font-family="sans-serif" font-size="13" fill="#aaccee" text-anchor="middle" dominant-baseline="central">&#x21BB; Clockwise</text>"##,
            btn_start_x + btn_w / 2.0,
            btn_y + btn_h / 2.0
        )
        .unwrap();
        write!(
            svg,
            r##"<rect x="{btn_start_x:.0}" y="{btn_y:.0}" width="{btn_w:.0}" height="{btn_h:.0}" fill="transparent" data-index="0" style="cursor:pointer"/>"##
        )
        .unwrap();

        // CCW button (data-index="1")
        let btn2_x = btn_start_x + btn_w + btn_gap;
        write!(
            svg,
            r##"<rect x="{btn2_x:.0}" y="{btn_y:.0}" width="{btn_w:.0}" height="{btn_h:.0}" rx="6" fill="#2a3a5a" stroke="#5577aa" stroke-width="1.5"/>"##
        )
        .unwrap();
        write!(
            svg,
            r##"<text x="{:.0}" y="{:.0}" font-family="sans-serif" font-size="13" fill="#aaccee" text-anchor="middle" dominant-baseline="central">&#x21BA; Counter-CW</text>"##,
            btn2_x + btn_w / 2.0,
            btn_y + btn_h / 2.0
        )
        .unwrap();
        write!(
            svg,
            r##"<rect x="{btn2_x:.0}" y="{btn_y:.0}" width="{btn_w:.0}" height="{btn_h:.0}" fill="transparent" data-index="1" style="cursor:pointer"/>"##
        )
        .unwrap();

        // Noise
        let noise_count = (difficulty.noise * 10.0) as u32;
        for _ in 0..noise_count {
            let x1 = rng.gen_range(0.0..width);
            let y1 = rng.gen_range(30.0..(height - button_area_h));
            let x2 = rng.gen_range(0.0..width);
            let y2 = rng.gen_range(30.0..(height - button_area_h));
            write!(
                svg,
                r##"<line x1="{x1:.0}" y1="{y1:.0}" x2="{x2:.0}" y2="{y2:.0}" stroke="#444" stroke-width="1" opacity="0.15"/>"##
            )
            .unwrap();
        }

        svg.push_str("</svg>");

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::SelectedIndices(vec![correct_answer]),
            expected_solve_time_ms: expected_solve_time(difficulty.time_limit_ms),
            point_value: CaptchaType::RotationPrediction.base_points(),
            captcha_type: CaptchaType::RotationPrediction,
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
    fn test_gears_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.0,
            noise: 0.0,
        };
        let gen = GearsGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert_eq!(instance.captcha_type, CaptchaType::RotationPrediction);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert_eq!(indices.len(), 1);
            assert!(indices[0] <= 1); // 0 = CW, 1 = CCW
        } else {
            panic!("Expected SelectedIndices solution");
        }
        if let RenderPayload::Svg(ref s) = instance.render_data {
            assert!(s.contains("<svg"));
            assert!(s.contains("data-index"));
            assert!(s.contains("Clockwise"));
        } else {
            panic!("Expected SVG render data");
        }
    }

    #[test]
    fn test_gears_seed_determinism() {
        let difficulty = DifficultyParams {
            level: 10,
            round_number: 3,
            time_limit_ms: 7000,
            complexity: 0.5,
            noise: 0.3,
        };
        let gen = GearsGenerator;

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
    fn test_gears_correct_answer_validates() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = GearsGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert!(gen.validate(
                &instance,
                &PlayerAnswer::SelectedIndices(indices.clone())
            ));
        }
    }

    #[test]
    fn test_gears_wrong_answer_rejects() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = GearsGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            let wrong = if indices[0] == 0 { 1 } else { 0 };
            assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![wrong])));
        }
        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![99])));
        assert!(!gen.validate(&instance, &PlayerAnswer::Text("wrong".to_string())));
    }
}
