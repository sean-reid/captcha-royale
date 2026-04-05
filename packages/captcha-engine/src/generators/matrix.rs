use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct MatrixPatternGenerator;

/// Shape types used in the matrix grid
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    Circle,
    Square,
    Triangle,
}

const SHAPES: [Shape; 3] = [Shape::Circle, Shape::Square, Shape::Triangle];

/// Color palette with distinct hues
const COLORS: [&str; 3] = ["#e74c3c", "#3498db", "#2ecc71"]; // red, blue, green

/// Size steps (small, medium, large)
const SIZES: [f32; 3] = [14.0, 22.0, 30.0];

/// Which rules are active for pattern generation
#[derive(Debug, Clone, Copy)]
enum Rule {
    Shape,
    Color,
    Size,
}

const ALL_RULES: [Rule; 3] = [
    Rule::Shape,
    Rule::Color,
    Rule::Size,
];

/// Properties for a single cell in the 3x3 grid
#[derive(Debug, Clone)]
struct CellProps {
    shape: Shape,
    color_idx: usize,
    size_idx: usize,
}

impl CellProps {
    fn draw_svg(&self, cx: f32, cy: f32) -> String {
        let color = COLORS[self.color_idx % COLORS.len()];
        let size = SIZES[self.size_idx % SIZES.len()];
        match self.shape {
            Shape::Circle => {
                format!(
                    r##"<circle cx="{cx:.1}" cy="{cy:.1}" r="{size:.1}" fill="{color}"/>"##
                )
            }
            Shape::Square => {
                let half = size;
                format!(
                    r##"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="{color}"/>"##,
                    cx - half,
                    cy - half,
                    half * 2.0,
                    half * 2.0
                )
            }
            Shape::Triangle => {
                let r = size;
                let top_x = cx;
                let top_y = cy - r;
                let bl_x = cx - r * 0.866;
                let bl_y = cy + r * 0.5;
                let br_x = cx + r * 0.866;
                let br_y = cy + r * 0.5;
                format!(
                    r##"<polygon points="{top_x:.1},{top_y:.1} {bl_x:.1},{bl_y:.1} {br_x:.1},{br_y:.1}" fill="{color}"/>"##
                )
            }
        }
    }
}

/// Generate cell properties for the 3x3 grid based on active rules.
/// Row-major: cells[row * 3 + col] for row, col in 0..3.
/// Returns 9 cells where cells[8] (bottom-right) is the answer.
fn generate_grid(
    rng: &mut ChaCha8Rng,
    rules: &[Rule],
) -> Vec<CellProps> {
    // Base permutations for each dimension across rows/columns.
    // For shape: each row cycles through shapes in a permuted order.
    // For color: each row uses a different starting color, shifting across columns.
    // For size: each column has a different size level.

    // Create random permutations for each dimension
    let shape_perm = random_permutation(rng);
    let color_perm = random_permutation(rng);
    let size_perm = random_permutation(rng);

    // Row-wise offset for shapes (each row starts at a different shape)
    let shape_row_offsets: [usize; 3] = random_permutation(rng);
    let color_row_offsets: [usize; 3] = random_permutation(rng);

    let mut cells = Vec::with_capacity(9);

    for row in 0..3usize {
        for col in 0..3usize {
            let shape_idx = if rules.iter().any(|r| matches!(r, Rule::Shape)) {
                // Each row cycles through 3 shapes; the starting shape differs per row
                (shape_row_offsets[row] + shape_perm[col]) % 3
            } else {
                // Fixed shape across the whole grid
                shape_perm[0]
            };

            let color_idx = if rules.iter().any(|r| matches!(r, Rule::Color)) {
                // Each row has a shifted color sequence
                (color_row_offsets[row] + color_perm[col]) % 3
            } else {
                color_perm[0]
            };

            let size_idx = if rules.iter().any(|r| matches!(r, Rule::Size)) {
                // Size increases across columns
                size_perm[col]
            } else {
                size_perm[0]
            };

            cells.push(CellProps {
                shape: SHAPES[shape_idx],
                color_idx,
                size_idx,
            });
        }
    }

    cells
}

/// Generate a random permutation of [0, 1, 2]
fn random_permutation(rng: &mut ChaCha8Rng) -> [usize; 3] {
    let mut perm = [0usize, 1, 2];
    // Fisher-Yates shuffle
    for i in (1..3).rev() {
        let j = rng.gen_range(0..=i);
        perm.swap(i, j);
    }
    perm
}

/// Generate wrong options that break exactly one rule
fn generate_wrong_option(
    correct: &CellProps,
    rules: &[Rule],
    rng: &mut ChaCha8Rng,
) -> CellProps {
    let mut wrong = correct.clone();

    // Pick a random rule to break
    let rule_to_break = &rules[rng.gen_range(0..rules.len())];

    match rule_to_break {
        Rule::Shape => {
            // Use a different shape
            loop {
                let new_shape = SHAPES[rng.gen_range(0..3)];
                if new_shape != correct.shape {
                    wrong.shape = new_shape;
                    break;
                }
            }
        }
        Rule::Color => {
            loop {
                let new_color = rng.gen_range(0..3);
                if new_color != correct.color_idx {
                    wrong.color_idx = new_color;
                    break;
                }
            }
        }
        Rule::Size => {
            loop {
                let new_size = rng.gen_range(0..3);
                if new_size != correct.size_idx {
                    wrong.size_idx = new_size;
                    break;
                }
            }
        }
    }

    wrong
}

impl CaptchaGenerator for MatrixPatternGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        // Determine active rules based on complexity
        let num_rules = if difficulty.complexity < 0.5 { 1 } else { 2 };
        let option_count = if difficulty.complexity < 0.5 { 4 } else { 6 };

        // Shuffle and pick rules
        let mut rule_indices = [0usize, 1, 2];
        for i in (1..3).rev() {
            let j = rng.gen_range(0..=i);
            rule_indices.swap(i, j);
        }
        let active_rules: Vec<Rule> = rule_indices[..num_rules]
            .iter()
            .map(|&i| ALL_RULES[i])
            .collect();

        // Generate the 3x3 grid
        let cells = generate_grid(rng, &active_rules);

        // The correct answer is cells[8] (bottom-right)
        let correct_cell = cells[8].clone();

        // Place correct answer at a random option index
        let correct_option_idx = rng.gen_range(0..option_count);

        // Generate options
        let mut options: Vec<CellProps> = Vec::with_capacity(option_count);
        for i in 0..option_count {
            if i == correct_option_idx {
                options.push(correct_cell.clone());
            } else {
                options.push(generate_wrong_option(&correct_cell, &active_rules, rng));
            }
        }

        // --- SVG Rendering ---
        let cell_size: f32 = 80.0;
        let gap: f32 = 8.0;
        let grid_width = 3.0 * cell_size + 2.0 * gap;
        let options_width = option_count as f32 * cell_size + (option_count as f32 - 1.0) * gap;
        let total_width = grid_width.max(options_width) + 40.0; // 20px padding each side
        let total_height = 3.0 * cell_size + 2.0 * gap + 30.0 + cell_size + 60.0;
        // rows: 30 prompt + 3 grid rows + gap + label + option row + padding

        let mut svg = String::with_capacity(8192);
        write!(
            svg,
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {total_width:.0} {total_height:.0}" width="{total_width:.0}" height="{total_height:.0}">"##
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
            r##"<text x="{:.0}" y="22" font-family="sans-serif" font-size="14" fill="#cccccc" text-anchor="middle">Complete the pattern: select the missing piece</text>"##,
            total_width / 2.0
        )
        .unwrap();

        // Add noise lines
        let noise_count = (2.0 + difficulty.noise * 6.0) as u32;
        for _ in 0..noise_count {
            let x1: f32 = rng.gen::<f32>() * total_width;
            let y1: f32 = rng.gen::<f32>() * total_height;
            let x2: f32 = rng.gen::<f32>() * total_width;
            let y2: f32 = rng.gen::<f32>() * total_height;
            let r = rng.gen_range(40..140);
            let g = rng.gen_range(40..140);
            let b = rng.gen_range(40..140);
            write!(
                svg,
                r##"<line x1="{x1:.0}" y1="{y1:.0}" x2="{x2:.0}" y2="{y2:.0}" stroke="rgb({r},{g},{b})" stroke-width="1" opacity="0.12"/>"##
            )
            .unwrap();
        }

        // Draw 3x3 grid
        let grid_offset_x = (total_width - grid_width) / 2.0;
        let grid_offset_y: f32 = 35.0;

        for row in 0..3usize {
            for col in 0..3usize {
                let x = grid_offset_x + col as f32 * (cell_size + gap);
                let y = grid_offset_y + row as f32 * (cell_size + gap);
                let cx = x + cell_size / 2.0;
                let cy = y + cell_size / 2.0;

                // Cell background
                write!(
                    svg,
                    r##"<rect x="{x:.1}" y="{y:.1}" width="{cell_size:.0}" height="{cell_size:.0}" fill="#252545" stroke="#444" stroke-width="1" rx="4"/>"##
                )
                .unwrap();

                if row == 2 && col == 2 {
                    // Missing cell - draw "?"
                    write!(
                        svg,
                        r##"<rect x="{x:.1}" y="{y:.1}" width="{cell_size:.0}" height="{cell_size:.0}" fill="none" stroke="#666" stroke-width="2" rx="4" stroke-dasharray="6,3"/>"##
                    )
                    .unwrap();
                    write!(
                        svg,
                        r##"<text x="{cx:.1}" y="{cy:.1}" font-family="sans-serif" font-size="32" fill="#888" text-anchor="middle" dominant-baseline="central">?</text>"##
                    )
                    .unwrap();
                } else {
                    let cell = &cells[row * 3 + col];
                    svg.push_str(&cell.draw_svg(cx, cy));
                }
            }
        }

        // Options label
        let options_label_y = grid_offset_y + 3.0 * (cell_size + gap) + 15.0;
        write!(
            svg,
            r##"<text x="{:.0}" y="{options_label_y:.0}" font-family="sans-serif" font-size="12" fill="#999" text-anchor="middle">Select the correct piece:</text>"##,
            total_width / 2.0
        )
        .unwrap();

        // Draw options
        let options_offset_x = (total_width - options_width) / 2.0;
        let options_y = options_label_y + 12.0;

        for (i, opt) in options.iter().enumerate() {
            let x = options_offset_x + i as f32 * (cell_size + gap);
            let y = options_y;
            let cx = x + cell_size / 2.0;
            let cy = y + cell_size / 2.0;

            // Cell background
            write!(
                svg,
                r##"<rect x="{x:.1}" y="{y:.1}" width="{cell_size:.0}" height="{cell_size:.0}" fill="#252545" stroke="#444" stroke-width="1" rx="4"/>"##
            )
            .unwrap();

            svg.push_str(&opt.draw_svg(cx, cy));

            // Clickable overlay
            write!(
                svg,
                r##"<rect x="{x:.1}" y="{y:.1}" width="{cell_size:.0}" height="{cell_size:.0}" fill="transparent" data-index="{i}" style="cursor:pointer"/>"##
            )
            .unwrap();
        }

        svg.push_str("</svg>");

        let time_limit = difficulty.time_limit_ms.clamp(8000, 20000);

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::SelectedIndices(vec![correct_option_idx as u32]),
            expected_solve_time_ms: expected_solve_time(time_limit),
            point_value: CaptchaType::CombinedModality.base_points(),
            captcha_type: CaptchaType::CombinedModality,
            time_limit_ms: time_limit,
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
    fn test_matrix_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 10000,
            complexity: 0.0,
            noise: 0.0,
        };
        let gen = MatrixPatternGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert_eq!(instance.captcha_type, CaptchaType::CombinedModality);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert_eq!(indices.len(), 1);
            assert!(indices[0] < 4); // low complexity = 4 options
        } else {
            panic!("Expected SelectedIndices solution");
        }
        if let RenderPayload::Svg(ref s) = instance.render_data {
            assert!(s.contains("<svg"));
            assert!(s.contains("Complete the pattern"));
            assert!(s.contains("data-index"));
            assert!(s.contains("?"));
        } else {
            panic!("Expected SVG render data");
        }
    }

    #[test]
    fn test_matrix_seed_determinism() {
        let difficulty = DifficultyParams {
            level: 10,
            round_number: 3,
            time_limit_ms: 12000,
            complexity: 0.5,
            noise: 0.3,
        };
        let gen = MatrixPatternGenerator;

        let mut rng1 = rng_from_seed(12345);
        let inst1 = gen.generate(&mut rng1, &difficulty);

        let mut rng2 = rng_from_seed(12345);
        let inst2 = gen.generate(&mut rng2, &difficulty);

        if let (Solution::SelectedIndices(i1), Solution::SelectedIndices(i2)) =
            (&inst1.solution, &inst2.solution)
        {
            assert_eq!(i1, i2);
        }

        if let (RenderPayload::Svg(s1), RenderPayload::Svg(s2)) =
            (&inst1.render_data, &inst2.render_data)
        {
            assert_eq!(s1, s2);
        }
    }

    #[test]
    fn test_matrix_correct_answer_validates() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 10000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = MatrixPatternGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert!(gen.validate(
                &instance,
                &PlayerAnswer::SelectedIndices(indices.clone())
            ));
        }
    }

    #[test]
    fn test_matrix_wrong_answer_rejects() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 10000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = MatrixPatternGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![99])));
        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![])));
        assert!(!gen.validate(&instance, &PlayerAnswer::Text("wrong".to_string())));
    }

    #[test]
    fn test_matrix_high_complexity_more_options() {
        let gen = MatrixPatternGenerator;
        let low = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 10000,
            complexity: 0.0,
            noise: 0.0,
        };
        let high = DifficultyParams {
            level: 50,
            round_number: 10,
            time_limit_ms: 18000,
            complexity: 1.0,
            noise: 1.0,
        };

        let mut rng1 = rng_from_seed(42);
        let inst_low = gen.generate(&mut rng1, &low);
        let mut rng2 = rng_from_seed(42);
        let inst_high = gen.generate(&mut rng2, &high);

        // Higher complexity = more options = longer SVG
        if let (RenderPayload::Svg(svg_low), RenderPayload::Svg(svg_high)) =
            (&inst_low.render_data, &inst_high.render_data)
        {
            assert!(svg_high.len() > svg_low.len());
        }
    }
}
