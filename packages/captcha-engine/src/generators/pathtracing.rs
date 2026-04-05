use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct PathTracingGenerator;

const LINE_COLORS_EASY: [&str; 4] = ["#e74c3c", "#3498db", "#2ecc71", "#f39c12"];
const LINE_COLORS_HARD: [&str; 4] = ["#c0392b", "#e74c3c", "#d35400", "#e67e22"];
const LABELS: [&str; 4] = ["A", "B", "C", "D"];

/// A cubic bezier path from top to bottom
struct TracePath {
    start_x: f32,
    end_x: f32,
    control_points: Vec<(f32, f32)>,
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn generate_path(
    rng: &mut ChaCha8Rng,
    start_x: f32,
    end_x: f32,
    top_y: f32,
    bottom_y: f32,
    width: f32,
    tangle: f32,
) -> TracePath {
    let _mid_y = (top_y + bottom_y) / 2.0;
    let spread = width * 0.3 * tangle;

    // Two control points in the middle section to create tangling
    let cp1_x = lerp(start_x, end_x, 0.33) + rng.gen_range(-spread..spread);
    let cp1_y = lerp(top_y, bottom_y, 0.33) + rng.gen_range(-20.0..20.0);
    let cp2_x = lerp(start_x, end_x, 0.66) + rng.gen_range(-spread..spread);
    let cp2_y = lerp(top_y, bottom_y, 0.66) + rng.gen_range(-20.0..20.0);

    // Clamp to reasonable bounds
    let margin = 15.0;
    let cp1_x = cp1_x.clamp(margin, width - margin);
    let cp2_x = cp2_x.clamp(margin, width - margin);

    TracePath {
        start_x,
        end_x,
        control_points: vec![(cp1_x, cp1_y), (cp2_x, cp2_y)],
    }
}

fn path_to_svg_d(path: &TracePath, top_y: f32, bottom_y: f32) -> String {
    format!(
        "M {:.1},{:.1} C {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
        path.start_x, top_y,
        path.control_points[0].0, path.control_points[0].1,
        path.control_points[1].0, path.control_points[1].1,
        path.end_x, bottom_y,
    )
}

impl CaptchaGenerator for PathTracingGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        let line_count = if difficulty.complexity < 0.5 { 3 } else { 4 };
        let tangle = 0.3 + difficulty.complexity * 0.7; // 0.3 to 1.0

        let width = 400.0_f32;
        let top_y = 60.0_f32;
        let bottom_y = 280.0_f32;
        let total_height = 340.0_f32;

        // Choose colors based on difficulty
        let colors = if difficulty.complexity < 0.6 {
            &LINE_COLORS_EASY[..line_count]
        } else {
            &LINE_COLORS_HARD[..line_count]
        };

        // Generate a random permutation for end positions
        // start_positions[i] connects to end_positions[i]
        let spacing = width / (line_count as f32 + 1.0);
        let start_xs: Vec<f32> = (0..line_count).map(|i| spacing * (i as f32 + 1.0)).collect();

        // Shuffle end positions
        let mut end_indices: Vec<usize> = (0..line_count).collect();
        for i in (1..line_count).rev() {
            let j = rng.gen_range(0..=i);
            end_indices.swap(i, j);
        }
        let end_xs: Vec<f32> = end_indices.iter().map(|&i| spacing * (i as f32 + 1.0)).collect();

        // Generate paths
        let paths: Vec<TracePath> = (0..line_count)
            .map(|i| generate_path(rng, start_xs[i], end_xs[i], top_y, bottom_y, width, tangle))
            .collect();

        // Pick which line to ask about
        let ask_line = rng.gen_range(0..line_count);
        // The answer: which numbered endpoint does line ask_line reach?
        // end_indices[ask_line] is the index of the bottom endpoint
        let correct_endpoint = end_indices[ask_line] as u32;

        // Randomly decide question variant
        let ask_start = rng.gen_bool(0.5);
        let question = if ask_start {
            format!("Where does line {} end?", LABELS[ask_line])
        } else {
            // "Which line reaches point N?" - answer is the endpoint index
            // Actually for this variant, the answer should still be the endpoint
            format!("Where does line {} end?", LABELS[ask_line])
        };

        let stroke_width = if difficulty.complexity < 0.5 { 3.0 } else { 2.5 };

        let mut svg = String::with_capacity(4096);
        write!(
            svg,
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width:.0} {total_height:.0}" width="{width:.0}" height="{total_height:.0}">"##
        ).unwrap();

        // Background
        write!(svg, r##"<rect width="{width:.0}" height="{total_height:.0}" fill="#1a1a2e"/>"##).unwrap();

        // Question text
        write!(
            svg,
            r##"<text x="{:.0}" y="22" font-family="sans-serif" font-size="14" fill="#cccccc" text-anchor="middle">{question}</text>"##,
            width / 2.0
        ).unwrap();

        // Noise — curved lines that look like real paths to create confusion
        let noise_count = 3 + (difficulty.noise * 12.0) as u32;
        for _ in 0..noise_count {
            let x1 = rng.gen_range(10.0..width - 10.0);
            let y1 = rng.gen_range(top_y..bottom_y);
            let cx = rng.gen_range(10.0..width - 10.0);
            let cy = rng.gen_range(top_y..bottom_y);
            let x2 = rng.gen_range(10.0..width - 10.0);
            let y2 = rng.gen_range(top_y..bottom_y);
            let gray = rng.gen_range(50..120);
            let sw = 1.0 + rng.gen::<f32>() * 1.5;
            let opacity = 0.15 + rng.gen::<f32>() * 0.2;
            write!(
                svg,
                r##"<path d="M {x1:.1} {y1:.1} Q {cx:.1} {cy:.1} {x2:.1} {y2:.1}" stroke="rgb({gray},{gray},{gray})" stroke-width="{sw:.1}" fill="none" opacity="{opacity:.2}"/>"##
            ).unwrap();
        }

        // Draw paths
        for i in 0..line_count {
            let d = path_to_svg_d(&paths[i], top_y, bottom_y);
            write!(
                svg,
                r##"<path d="{d}" fill="none" stroke="{}" stroke-width="{stroke_width:.1}" stroke-linecap="round"/>"##,
                colors[i]
            ).unwrap();
        }

        // Draw start labels (A, B, C, D) at top
        for i in 0..line_count {
            write!(
                svg,
                r##"<circle cx="{:.1}" cy="{:.1}" r="12" fill="#222" stroke="{}" stroke-width="2"/>"##,
                start_xs[i], top_y, colors[i]
            ).unwrap();
            write!(
                svg,
                r##"<text x="{:.1}" y="{:.1}" font-family="sans-serif" font-size="12" font-weight="bold" fill="{}" text-anchor="middle" dominant-baseline="central">{}</text>"##,
                start_xs[i], top_y, colors[i], LABELS[i]
            ).unwrap();
        }

        // Draw end labels (1, 2, 3, 4) at bottom
        let end_label_y = bottom_y + 8.0;
        for i in 0..line_count {
            let ex = spacing * (i as f32 + 1.0);
            write!(
                svg,
                r##"<circle cx="{ex:.1}" cy="{end_label_y:.1}" r="12" fill="#222" stroke="#888" stroke-width="2"/>"##,
            ).unwrap();
            write!(
                svg,
                r##"<text x="{ex:.1}" y="{end_label_y:.1}" font-family="sans-serif" font-size="12" font-weight="bold" fill="#cccccc" text-anchor="middle" dominant-baseline="central">{}</text>"##,
                i + 1
            ).unwrap();
        }

        // Answer buttons at the very bottom
        let btn_y = total_height - 20.0;
        let btn_w = 40.0_f32;
        let btn_spacing = width / (line_count as f32 + 1.0);
        for i in 0..line_count {
            let bx = btn_spacing * (i as f32 + 1.0) - btn_w / 2.0;
            write!(
                svg,
                r##"<rect x="{bx:.1}" y="{:.1}" width="{btn_w:.0}" height="24" rx="4" fill="#333" stroke="#666" stroke-width="1" data-index="{i}" style="cursor:pointer"/>"##,
                btn_y - 18.0
            ).unwrap();
            write!(
                svg,
                r##"<text x="{:.1}" y="{:.1}" font-family="sans-serif" font-size="13" fill="#cccccc" text-anchor="middle" dominant-baseline="central" pointer-events="none">{}</text>"##,
                btn_spacing * (i as f32 + 1.0), btn_y - 6.0, i + 1
            ).unwrap();
        }

        svg.push_str("</svg>");

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::SelectedIndices(vec![correct_endpoint]),
            expected_solve_time_ms: expected_solve_time(difficulty.time_limit_ms),
            point_value: CaptchaType::PathTracing.base_points(),
            captcha_type: CaptchaType::PathTracing,
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
    fn test_pathtracing_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 8000,
            complexity: 0.0,
            noise: 0.0,
        };
        let gen = PathTracingGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert_eq!(instance.captcha_type, CaptchaType::PathTracing);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert_eq!(indices.len(), 1);
            assert!(indices[0] < 4);
        } else {
            panic!("Expected SelectedIndices solution");
        }
        if let RenderPayload::Svg(ref s) = instance.render_data {
            assert!(s.contains("<svg"));
            assert!(s.contains("data-index"));
            assert!(s.contains("Where does line"));
        } else {
            panic!("Expected SVG render data");
        }
    }

    #[test]
    fn test_pathtracing_seed_determinism() {
        let difficulty = DifficultyParams {
            level: 10,
            round_number: 3,
            time_limit_ms: 9000,
            complexity: 0.5,
            noise: 0.3,
        };
        let gen = PathTracingGenerator;

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
    fn test_pathtracing_correct_answer_validates() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 8000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = PathTracingGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert!(gen.validate(
                &instance,
                &PlayerAnswer::SelectedIndices(indices.clone())
            ));
        }
    }

    #[test]
    fn test_pathtracing_wrong_answer_rejects() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 8000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = PathTracingGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![99])));
        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![])));
        assert!(!gen.validate(&instance, &PlayerAnswer::Text("wrong".to_string())));
    }
}
