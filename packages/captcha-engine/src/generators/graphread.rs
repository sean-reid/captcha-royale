use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct GraphReadingGenerator;

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun",
    "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

impl CaptchaGenerator for GraphReadingGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        let c = difficulty.complexity;
        let n = difficulty.noise;

        // Bar count: 4 at low complexity, up to 6 at high
        let bar_count = 4 + (c * 2.0) as usize;
        let bar_count = bar_count.min(6);

        // Pick consecutive months starting from a random offset
        let start_month = rng.gen_range(0..12);
        let labels: Vec<&str> = (0..bar_count)
            .map(|i| MONTHS[(start_month + i) % 12])
            .collect();

        // Generate values; at low complexity make them very different, at high make them closer
        let base_value = rng.gen_range(20..60_u32);
        let spread = if c < 0.3 {
            40 // large spread
        } else if c < 0.6 {
            25
        } else {
            12 // small spread, values close together
        };

        // Generate unique values so there's always a single clear max/min
        let mut values: Vec<u32> = Vec::new();
        for _ in 0..bar_count {
            loop {
                let v = (base_value as i32 + rng.gen_range(-spread..=spread)).max(5) as u32;
                if !values.contains(&v) {
                    values.push(v);
                    break;
                }
            }
        }

        let max_val = *values.iter().max().unwrap();
        let min_val = *values.iter().min().unwrap();

        // Decide question: tallest or shortest
        let ask_tallest = rng.gen_bool(0.5);
        let target_val = if ask_tallest { max_val } else { min_val };
        // Find first index with that value
        let correct_index = values.iter().position(|&v| v == target_val).unwrap() as u32;

        let prompt = if ask_tallest {
            "Click the tallest bar"
        } else {
            "Click the shortest bar"
        };

        // SVG dimensions
        let width = 450;
        let height = 300;
        let chart_left = 55.0;
        let chart_right = width as f32 - 20.0;
        let chart_top = 50.0;
        let chart_bottom = height as f32 - 50.0;
        let chart_w = chart_right - chart_left;
        let chart_h = chart_bottom - chart_top;
        let bar_gap = 12.0;
        let bar_w = (chart_w - bar_gap * (bar_count as f32 + 1.0)) / bar_count as f32;

        // Y-axis scale
        let y_max = (max_val as f32 * 1.2).ceil();

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

        // Prompt text
        write!(
            svg,
            r##"<text x="{}" y="30" font-family="sans-serif" font-size="15" fill="#cccccc" text-anchor="middle">{prompt}</text>"##,
            width / 2
        )
        .unwrap();

        // Y-axis line
        write!(
            svg,
            r##"<line x1="{chart_left:.0}" y1="{chart_top:.0}" x2="{chart_left:.0}" y2="{chart_bottom:.0}" stroke="#555555" stroke-width="1"/>"##
        )
        .unwrap();

        // X-axis line
        write!(
            svg,
            r##"<line x1="{chart_left:.0}" y1="{chart_bottom:.0}" x2="{chart_right:.0}" y2="{chart_bottom:.0}" stroke="#555555" stroke-width="1"/>"##
        )
        .unwrap();

        // Y-axis labels (a few ticks)
        let tick_count = 4;
        for i in 0..=tick_count {
            let val = y_max * i as f32 / tick_count as f32;
            let y = chart_bottom - (val / y_max) * chart_h;
            write!(
                svg,
                r##"<text x="{:.0}" y="{y:.1}" font-family="sans-serif" font-size="11" fill="#888888" text-anchor="end" dominant-baseline="central">{:.0}</text>"##,
                chart_left - 8.0, val
            )
            .unwrap();
            // Grid line
            write!(
                svg,
                r##"<line x1="{chart_left:.0}" y1="{y:.1}" x2="{chart_right:.0}" y2="{y:.1}" stroke="#333333" stroke-width="0.5" stroke-dasharray="4,4"/>"##
            )
            .unwrap();
        }

        // Grid noise at high noise
        let grid_noise_count = (n * 8.0) as u32;
        for _ in 0..grid_noise_count {
            let nx1 = rng.gen_range(chart_left..chart_right);
            let ny1 = rng.gen_range(chart_top..chart_bottom);
            let nx2 = rng.gen_range(chart_left..chart_right);
            let ny2 = rng.gen_range(chart_top..chart_bottom);
            let r = rng.gen_range(60..140);
            let g = rng.gen_range(60..140);
            let b = rng.gen_range(60..140);
            let opacity = 0.05 + rng.gen::<f32>() * 0.1;
            write!(
                svg,
                r##"<line x1="{nx1:.1}" y1="{ny1:.1}" x2="{nx2:.1}" y2="{ny2:.1}" stroke="rgb({r},{g},{b})" stroke-width="0.8" opacity="{opacity:.2}"/>"##
            )
            .unwrap();
        }

        // Bar colors
        let bar_colors = ["#3498db", "#e74c3c", "#2ecc71", "#f39c12", "#9b59b6", "#1abc9c"];

        // Draw bars
        for (i, (&val, &label)) in values.iter().zip(labels.iter()).enumerate() {
            let bx = chart_left + bar_gap + i as f32 * (bar_w + bar_gap);
            let bar_h = (val as f32 / y_max) * chart_h;
            let by = chart_bottom - bar_h;
            let color = bar_colors[i % bar_colors.len()];

            // Bar rectangle
            write!(
                svg,
                r##"<rect x="{bx:.1}" y="{by:.1}" width="{bar_w:.1}" height="{bar_h:.1}" fill="{color}" rx="2"/>"##
            )
            .unwrap();

            // Clickable overlay
            write!(
                svg,
                r##"<rect x="{bx:.1}" y="{by:.1}" width="{bar_w:.1}" height="{bar_h:.1}" fill="transparent" data-index="{i}" style="cursor:pointer"/>"##
            )
            .unwrap();

            // Value label on top of bar
            write!(
                svg,
                r##"<text x="{:.1}" y="{:.1}" font-family="sans-serif" font-size="11" fill="#dddddd" text-anchor="middle">{val}</text>"##,
                bx + bar_w / 2.0,
                by - 6.0
            )
            .unwrap();

            // X-axis label
            write!(
                svg,
                r##"<text x="{:.1}" y="{:.0}" font-family="sans-serif" font-size="12" fill="#aaaaaa" text-anchor="middle">{label}</text>"##,
                bx + bar_w / 2.0,
                chart_bottom + 18.0
            )
            .unwrap();
        }

        svg.push_str("</svg>");

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::SelectedIndices(vec![correct_index]),
            expected_solve_time_ms: expected_solve_time(difficulty.time_limit_ms),
            point_value: CaptchaType::GraphReading.base_points(),
            captcha_type: CaptchaType::GraphReading,
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
    fn test_graphreading_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 4000,
            complexity: 0.0,
            noise: 0.0,
        };
        let gen = GraphReadingGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert_eq!(instance.captcha_type, CaptchaType::GraphReading);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert_eq!(indices.len(), 1);
            assert!(indices[0] < 6); // max 6 bars
        } else {
            panic!("Expected SelectedIndices solution");
        }
        if let RenderPayload::Svg(ref s) = instance.render_data {
            assert!(s.contains("<svg"));
            assert!(s.contains("data-index"));
            assert!(s.contains("Click the"));
        } else {
            panic!("Expected SVG render data");
        }
    }

    #[test]
    fn test_graphreading_seed_determinism() {
        let difficulty = DifficultyParams {
            level: 10,
            round_number: 3,
            time_limit_ms: 6000,
            complexity: 0.5,
            noise: 0.3,
        };
        let gen = GraphReadingGenerator;

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
    fn test_graphreading_correct_answer_validates() {
        let mut rng = rng_from_seed(77);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = GraphReadingGenerator;
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
    fn test_graphreading_wrong_answer_rejects() {
        let mut rng = rng_from_seed(77);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = GraphReadingGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![99])));
        assert!(!gen.validate(&instance, &PlayerAnswer::Text("wrong".to_string())));
    }
}
