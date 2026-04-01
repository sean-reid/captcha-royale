use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct SequenceGenerator;

/// Pattern types for sequences
#[derive(Debug, Clone, Copy)]
enum PatternType {
    Rotation,
    ColorCycle,
    SizeChange,
}

const PATTERN_TYPES: [PatternType; 3] = [
    PatternType::Rotation,
    PatternType::ColorCycle,
    PatternType::SizeChange,
];

/// Hue cycle palette
const HUE_STEPS: [f32; 6] = [0.0, 60.0, 120.0, 180.0, 240.0, 300.0];

/// Shape for the sequence items
#[derive(Debug, Clone, Copy)]
enum SeqShape {
    Triangle,
    Square,
    Pentagon,
}

const SEQ_SHAPES: [SeqShape; 3] = [SeqShape::Triangle, SeqShape::Square, SeqShape::Pentagon];

impl SeqShape {
    fn svg_at(&self, cx: f32, cy: f32, size: f32, rotation: f32, fill: &str) -> String {
        let inner = match self {
            SeqShape::Triangle => {
                let r = size * 0.4;
                let mut points = String::new();
                for i in 0..3 {
                    let angle = std::f32::consts::PI * 2.0 * i as f32 / 3.0
                        - std::f32::consts::PI / 2.0;
                    let px = cx + angle.cos() * r;
                    let py = cy + angle.sin() * r;
                    if !points.is_empty() {
                        points.push(' ');
                    }
                    write!(points, "{px:.1},{py:.1}").unwrap();
                }
                format!(r#"<polygon points="{points}" fill="{fill}"/>"#)
            }
            SeqShape::Square => {
                let half = size * 0.35;
                format!(
                    r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="{fill}"/>"#,
                    cx - half,
                    cy - half,
                    half * 2.0,
                    half * 2.0
                )
            }
            SeqShape::Pentagon => {
                let r = size * 0.4;
                let mut points = String::new();
                for i in 0..5 {
                    let angle = std::f32::consts::PI * 2.0 * i as f32 / 5.0
                        - std::f32::consts::PI / 2.0;
                    let px = cx + angle.cos() * r;
                    let py = cy + angle.sin() * r;
                    if !points.is_empty() {
                        points.push(' ');
                    }
                    write!(points, "{px:.1},{py:.1}").unwrap();
                }
                format!(r#"<polygon points="{points}" fill="{fill}"/>"#)
            }
        };
        format!(
            r#"<g transform="rotate({rotation:.1},{cx:.1},{cy:.1})">{inner}</g>"#
        )
    }
}

fn hsl_to_css(h: f32, s: f32, l: f32) -> String {
    // Simple HSL to RGB
    let h = h % 360.0;
    let s = s / 100.0;
    let l = l / 100.0;

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let r = ((r + m) * 255.0) as u8;
    let g = ((g + m) * 255.0) as u8;
    let b = ((b + m) * 255.0) as u8;
    format!("rgb({r},{g},{b})")
}

impl CaptchaGenerator for SequenceGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        // Sequence length: 3-5 shown items based on complexity
        let seq_len = 3 + (difficulty.complexity * 2.0) as usize;
        let seq_len = seq_len.min(5);

        // Number of answer options: 3-5
        let option_count = 3 + (difficulty.complexity * 2.0) as usize;
        let option_count = option_count.min(5);

        // Pick pattern type and shape
        let pattern = PATTERN_TYPES[rng.gen_range(0..PATTERN_TYPES.len())];
        let shape = SEQ_SHAPES[rng.gen_range(0..SEQ_SHAPES.len())];

        // Generate the sequence parameters
        let base_hue = HUE_STEPS[rng.gen_range(0..HUE_STEPS.len())];
        let base_rotation = 0.0_f32;
        let base_size = 40.0_f32;

        // Step amounts per sequence position
        let rotation_step = rng.gen_range(30.0..90.0_f32);
        let hue_step = rng.gen_range(30.0..60.0_f32);
        let size_step = rng.gen_range(5.0..12.0_f32);

        // Build sequence items (seq_len shown + 1 correct next)
        let total_items = seq_len + 1;

        struct SeqItem {
            rotation: f32,
            hue: f32,
            size: f32,
        }

        let items: Vec<SeqItem> = (0..total_items)
            .map(|i| {
                let idx = i as f32;
                match pattern {
                    PatternType::Rotation => SeqItem {
                        rotation: base_rotation + rotation_step * idx,
                        hue: base_hue,
                        size: base_size,
                    },
                    PatternType::ColorCycle => SeqItem {
                        rotation: base_rotation,
                        hue: (base_hue + hue_step * idx) % 360.0,
                        size: base_size,
                    },
                    PatternType::SizeChange => SeqItem {
                        rotation: base_rotation,
                        hue: base_hue,
                        size: base_size + size_step * idx,
                    },
                }
            })
            .collect();

        // The correct answer is the last item
        let correct_item = &items[seq_len];

        // Generate wrong options by perturbing the correct answer
        let correct_option_idx = rng.gen_range(0..option_count);

        let cell_size = 80.0;
        let gap = 10.0;

        // Layout: sequence row + "?" + options row
        let seq_width = (seq_len as f32 + 1.0) * (cell_size + gap) + gap; // +1 for "?"
        let opt_width = option_count as f32 * (cell_size + gap) + gap;
        let width = seq_width.max(opt_width);
        let height = cell_size * 2.0 + gap * 3.0 + 60.0; // two rows + prompt + label

        let mut svg = String::with_capacity(4096);
        write!(
            svg,
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width:.0} {height:.0}" width="{width:.0}" height="{height:.0}">"#
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
            r##"<text x="{:.0}" y="22" font-family="sans-serif" font-size="14" fill="#cccccc" text-anchor="middle">What comes next in the sequence?</text>"##,
            width / 2.0
        )
        .unwrap();

        // Add noise
        let noise_count = (3.0 + difficulty.noise * 10.0) as u32;
        for _ in 0..noise_count {
            let x1 = rng.gen::<f32>() * width;
            let y1 = rng.gen::<f32>() * height;
            let x2 = rng.gen::<f32>() * width;
            let y2 = rng.gen::<f32>() * height;
            let r = rng.gen_range(40..140);
            let g = rng.gen_range(40..140);
            let b = rng.gen_range(40..140);
            write!(
                svg,
                r#"<line x1="{x1:.0}" y1="{y1:.0}" x2="{x2:.0}" y2="{y2:.0}" stroke="rgb({r},{g},{b})" stroke-width="1" opacity="0.15"/>"#
            )
            .unwrap();
        }

        // Draw sequence items
        let seq_offset_x = (width - seq_width) / 2.0 + gap;
        let row1_y = 35.0 + gap + cell_size / 2.0;

        for (i, item) in items.iter().enumerate().take(seq_len) {
            let cx = seq_offset_x + i as f32 * (cell_size + gap) + cell_size / 2.0;
            let fill = hsl_to_css(item.hue, 70.0, 55.0);

            // Cell border
            write!(
                svg,
                r##"<rect x="{:.0}" y="{:.0}" width="{cell_size:.0}" height="{cell_size:.0}" fill="none" stroke="#444" stroke-width="1" rx="4"/>"##,
                cx - cell_size / 2.0,
                row1_y - cell_size / 2.0
            )
            .unwrap();

            svg.push_str(&shape.svg_at(cx, row1_y, item.size, item.rotation, &fill));
        }

        // Question mark cell
        let qx = seq_offset_x + seq_len as f32 * (cell_size + gap) + cell_size / 2.0;
        write!(
            svg,
            r##"<rect x="{:.0}" y="{:.0}" width="{cell_size:.0}" height="{cell_size:.0}" fill="none" stroke="#666" stroke-width="2" rx="4" stroke-dasharray="6,3"/>"##,
            qx - cell_size / 2.0,
            row1_y - cell_size / 2.0
        )
        .unwrap();
        write!(
            svg,
            r##"<text x="{qx:.0}" y="{row1_y:.0}" font-family="sans-serif" font-size="32" fill="#888" text-anchor="middle" dominant-baseline="central">?</text>"##
        )
        .unwrap();

        // Options label
        let row2_label_y = 35.0 + gap + cell_size + gap + 15.0;
        write!(
            svg,
            r##"<text x="{:.0}" y="{row2_label_y:.0}" font-family="sans-serif" font-size="12" fill="#999" text-anchor="middle">Select the correct next item:</text>"##,
            width / 2.0
        )
        .unwrap();

        // Draw option items
        let opt_offset_x = (width - opt_width) / 2.0 + gap;
        let row2_y = row2_label_y + 15.0 + cell_size / 2.0;

        for i in 0..option_count {
            let cx = opt_offset_x + i as f32 * (cell_size + gap) + cell_size / 2.0;

            let (opt_rotation, opt_hue, opt_size) = if i == correct_option_idx {
                (correct_item.rotation, correct_item.hue, correct_item.size)
            } else {
                // Generate a wrong option by perturbing
                match pattern {
                    PatternType::Rotation => {
                        let wrong_rot = correct_item.rotation
                            + rotation_step * rng.gen_range(1..=3) as f32
                                * if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
                        (wrong_rot, correct_item.hue, correct_item.size)
                    }
                    PatternType::ColorCycle => {
                        let wrong_hue = (correct_item.hue
                            + hue_step * rng.gen_range(1..=3) as f32
                                * if rng.gen_bool(0.5) { 1.0 } else { -1.0 })
                            % 360.0;
                        let wrong_hue = if wrong_hue < 0.0 {
                            wrong_hue + 360.0
                        } else {
                            wrong_hue
                        };
                        (correct_item.rotation, wrong_hue, correct_item.size)
                    }
                    PatternType::SizeChange => {
                        let wrong_size = correct_item.size
                            + size_step * rng.gen_range(1..=3) as f32
                                * if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
                        let wrong_size = wrong_size.max(15.0);
                        (correct_item.rotation, correct_item.hue, wrong_size)
                    }
                }
            };

            let fill = hsl_to_css(opt_hue, 70.0, 55.0);

            // Cell border
            let cell_x = cx - cell_size / 2.0;
            let cell_y = row2_y - cell_size / 2.0;
            write!(
                svg,
                r##"<rect x="{cell_x:.0}" y="{cell_y:.0}" width="{cell_size:.0}" height="{cell_size:.0}" fill="none" stroke="#444" stroke-width="1" rx="4"/>"##,
            )
            .unwrap();

            svg.push_str(&shape.svg_at(cx, row2_y, opt_size, opt_rotation, &fill));

            // Clickable overlay
            write!(
                svg,
                r#"<rect x="{cell_x:.0}" y="{cell_y:.0}" width="{cell_size:.0}" height="{cell_size:.0}" fill="transparent" data-index="{i}" style="cursor:pointer"/>"#,
            )
            .unwrap();
        }

        svg.push_str("</svg>");

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::SelectedIndices(vec![correct_option_idx as u32]),
            expected_solve_time_ms: expected_solve_time(difficulty.time_limit_ms),
            point_value: CaptchaType::SequenceCompletion.base_points(),
            captcha_type: CaptchaType::SequenceCompletion,
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
    fn test_sequence_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 6000,
            complexity: 0.0,
            noise: 0.0,
        };
        let gen = SequenceGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert_eq!(instance.captcha_type, CaptchaType::SequenceCompletion);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert_eq!(indices.len(), 1);
            assert!(indices[0] < 3); // min option count is 3
        } else {
            panic!("Expected SelectedIndices solution");
        }
        if let RenderPayload::Svg(ref s) = instance.render_data {
            assert!(s.contains("<svg"));
            assert!(s.contains("What comes next"));
        } else {
            panic!("Expected SVG render data");
        }
    }

    #[test]
    fn test_sequence_seed_determinism() {
        let difficulty = DifficultyParams {
            level: 10,
            round_number: 3,
            time_limit_ms: 7000,
            complexity: 0.5,
            noise: 0.3,
        };
        let gen = SequenceGenerator;

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
    fn test_sequence_correct_answer_validates() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 6000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = SequenceGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert!(gen.validate(
                &instance,
                &PlayerAnswer::SelectedIndices(indices.clone())
            ));
        }
    }

    #[test]
    fn test_sequence_wrong_answer_rejects() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 6000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = SequenceGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![99])));
        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![])));
        assert!(!gen.validate(&instance, &PlayerAnswer::Text("wrong".to_string())));
    }

    #[test]
    fn test_sequence_difficulty_scaling() {
        let gen = SequenceGenerator;
        let low = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 6000,
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

        // Higher complexity = more sequence items + more options = longer SVG
        if let (RenderPayload::Svg(svg_low), RenderPayload::Svg(svg_high)) =
            (&inst_low.render_data, &inst_high.render_data)
        {
            assert!(svg_high.len() > svg_low.len());
        }
    }
}
