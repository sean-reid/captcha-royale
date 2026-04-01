use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct GridGenerator;

/// Available shape types for the grid
#[derive(Debug, Clone, Copy, PartialEq)]
enum Shape {
    Circle,
    Square,
    Triangle,
    Star,
    Diamond,
    Hexagon,
}

impl Shape {
    fn name(&self) -> &'static str {
        match self {
            Shape::Circle => "circles",
            Shape::Square => "squares",
            Shape::Triangle => "triangles",
            Shape::Star => "stars",
            Shape::Diamond => "diamonds",
            Shape::Hexagon => "hexagons",
        }
    }

    fn svg_at(&self, cx: f32, cy: f32, size: f32, fill: &str) -> String {
        match self {
            Shape::Circle => {
                format!(
                    r#"<circle cx="{cx:.0}" cy="{cy:.0}" r="{:.0}" fill="{fill}"/>"#,
                    size * 0.4
                )
            }
            Shape::Square => {
                let half = size * 0.35;
                format!(
                    r#"<rect x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}" fill="{fill}"/>"#,
                    cx - half,
                    cy - half,
                    half * 2.0,
                    half * 2.0
                )
            }
            Shape::Triangle => {
                let h = size * 0.4;
                format!(
                    r#"<polygon points="{cx:.0},{:.0} {:.0},{:.0} {:.0},{:.0}" fill="{fill}"/>"#,
                    cy - h,
                    cx - h,
                    cy + h,
                    cx + h,
                    cy + h
                )
            }
            Shape::Star => {
                let r_outer = size * 0.4;
                let r_inner = size * 0.18;
                let mut points = String::new();
                for i in 0..10 {
                    let angle =
                        std::f32::consts::PI / 2.0 + std::f32::consts::PI * 2.0 * i as f32 / 10.0;
                    let r = if i % 2 == 0 { r_outer } else { r_inner };
                    let px = cx + angle.cos() * r;
                    let py = cy - angle.sin() * r;
                    if !points.is_empty() {
                        points.push(' ');
                    }
                    write!(points, "{px:.0},{py:.0}").unwrap();
                }
                format!(r#"<polygon points="{points}" fill="{fill}"/>"#)
            }
            Shape::Diamond => {
                let h = size * 0.4;
                format!(
                    r#"<polygon points="{cx:.0},{:.0} {:.0},{cy:.0} {cx:.0},{:.0} {:.0},{cy:.0}" fill="{fill}"/>"#,
                    cy - h,
                    cx + h,
                    cy + h,
                    cx - h
                )
            }
            Shape::Hexagon => {
                let r = size * 0.38;
                let mut points = String::new();
                for i in 0..6 {
                    let angle = std::f32::consts::PI * 2.0 * i as f32 / 6.0
                        - std::f32::consts::PI / 6.0;
                    let px = cx + angle.cos() * r;
                    let py = cy + angle.sin() * r;
                    if !points.is_empty() {
                        points.push(' ');
                    }
                    write!(points, "{px:.0},{py:.0}").unwrap();
                }
                format!(r#"<polygon points="{points}" fill="{fill}"/>"#)
            }
        }
    }
}

const ALL_SHAPES: [Shape; 6] = [
    Shape::Circle,
    Shape::Square,
    Shape::Triangle,
    Shape::Star,
    Shape::Diamond,
    Shape::Hexagon,
];

impl CaptchaGenerator for GridGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        // Grid size scales from 2x2 to 4x4
        let grid_size = if difficulty.complexity < 0.33 {
            2
        } else if difficulty.complexity < 0.66 {
            3
        } else {
            4
        };
        let total_cells = grid_size * grid_size;

        // Pick target shape and distractor shapes
        let target_shape = ALL_SHAPES[rng.gen_range(0..ALL_SHAPES.len())];
        let distractor_shapes: Vec<Shape> = ALL_SHAPES
            .iter()
            .filter(|s| **s != target_shape)
            .copied()
            .collect();

        // Decide which cells contain the target (at least 1, at most half+1)
        let target_count = rng.gen_range(1..=(total_cells / 2 + 1).min(total_cells));
        let mut is_target = vec![false; total_cells as usize];
        let mut indices: Vec<u32> = (0..total_cells).collect();

        // Fisher-Yates shuffle
        for i in (1..indices.len()).rev() {
            let j = rng.gen_range(0..=i);
            indices.swap(i, j);
        }

        let correct_indices: Vec<u32> = indices[..target_count as usize].to_vec();
        for &idx in &correct_indices {
            is_target[idx as usize] = true;
        }

        // Generate cell SVGs
        let cell_size = 100.0;
        let mut cells = Vec::with_capacity(total_cells as usize);

        for i in 0..total_cells {
            let shape = if is_target[i as usize] {
                target_shape
            } else {
                distractor_shapes[rng.gen_range(0..distractor_shapes.len())]
            };

            let r = rng.gen_range(80..220);
            let g = rng.gen_range(80..220);
            let b = rng.gen_range(80..220);
            let fill = format!("rgb({r},{g},{b})");

            let cx = cell_size / 2.0;
            let cy = cell_size / 2.0;

            let mut cell_svg = format!(
                r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {cell_size} {cell_size}" width="{cell_size}" height="{cell_size}"><rect width="{cell_size}" height="{cell_size}" fill="#16213e"/>"##
            );

            // Add noise based on difficulty
            let noise_count = (difficulty.noise * 8.0) as u32;
            for _ in 0..noise_count {
                let nx = rng.gen::<f32>() * cell_size;
                let ny = rng.gen::<f32>() * cell_size;
                let nr = rng.gen_range(1..4);
                write!(
                    cell_svg,
                    r#"<circle cx="{nx:.0}" cy="{ny:.0}" r="{nr}" fill="rgba(255,255,255,0.1)"/>"#
                )
                .unwrap();
            }

            cell_svg.push_str(&shape.svg_at(cx, cy, cell_size, &fill));
            cell_svg.push_str("</svg>");

            cells.push(GridCell {
                index: i,
                svg: cell_svg,
                label: shape.name().to_string(),
            });
        }

        let prompt = format!("Select all {} with {}", grid_size_label(grid_size), target_shape.name());

        CaptchaInstance {
            render_data: RenderPayload::Grid {
                cols: grid_size,
                rows: grid_size,
                cells,
                prompt,
            },
            solution: Solution::SelectedIndices({
                let mut sorted = correct_indices;
                sorted.sort();
                sorted
            }),
            expected_solve_time_ms: expected_solve_time(difficulty.time_limit_ms),
            point_value: CaptchaType::ImageGrid.base_points(),
            captcha_type: CaptchaType::ImageGrid,
            time_limit_ms: difficulty.time_limit_ms,
        }
    }

    fn validate(&self, instance: &CaptchaInstance, answer: &PlayerAnswer) -> bool {
        instance.validate(answer)
    }
}

fn grid_size_label(size: u32) -> &'static str {
    match size {
        2 => "squares",
        3 => "squares",
        4 => "squares",
        _ => "cells",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::rng_from_seed;

    #[test]
    fn test_grid_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 4000,
            complexity: 0.0,
            noise: 0.0,
        };
        let gen = GridGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert_eq!(instance.captcha_type, CaptchaType::ImageGrid);
        if let RenderPayload::Grid {
            cols,
            rows,
            cells,
            prompt,
        } = &instance.render_data
        {
            assert_eq!(*cols, 2);
            assert_eq!(*rows, 2);
            assert_eq!(cells.len(), 4);
            assert!(!prompt.is_empty());
        } else {
            panic!("Expected Grid render data");
        }
    }

    #[test]
    fn test_grid_correct_answer_validates() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.3,
            noise: 0.2,
        };
        let gen = GridGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert!(gen.validate(
                &instance,
                &PlayerAnswer::SelectedIndices(indices.clone())
            ));
        }
    }

    #[test]
    fn test_grid_wrong_answer_rejects() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.3,
            noise: 0.2,
        };
        let gen = GridGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        // Empty selection should fail
        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![])));
        // Wrong type should fail
        assert!(!gen.validate(&instance, &PlayerAnswer::Text("wrong".to_string())));
    }

    #[test]
    fn test_grid_scales_with_difficulty() {
        let gen = GridGenerator;
        let low = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 4000,
            complexity: 0.0,
            noise: 0.0,
        };
        let high = DifficultyParams {
            level: 50,
            round_number: 10,
            time_limit_ms: 8000,
            complexity: 1.0,
            noise: 1.0,
        };

        let mut rng1 = rng_from_seed(42);
        let inst_low = gen.generate(&mut rng1, &low);
        let mut rng2 = rng_from_seed(42);
        let inst_high = gen.generate(&mut rng2, &high);

        if let (
            RenderPayload::Grid { cols: c1, .. },
            RenderPayload::Grid { cols: c2, .. },
        ) = (&inst_low.render_data, &inst_high.render_data)
        {
            assert!(*c2 >= *c1);
        }
    }

    #[test]
    fn test_grid_seed_determinism() {
        let difficulty = DifficultyParams {
            level: 10,
            round_number: 3,
            time_limit_ms: 6000,
            complexity: 0.5,
            noise: 0.3,
        };
        let gen = GridGenerator;

        let mut rng1 = rng_from_seed(12345);
        let inst1 = gen.generate(&mut rng1, &difficulty);

        let mut rng2 = rng_from_seed(12345);
        let inst2 = gen.generate(&mut rng2, &difficulty);

        if let (
            Solution::SelectedIndices(i1),
            Solution::SelectedIndices(i2),
        ) = (&inst1.solution, &inst2.solution)
        {
            assert_eq!(i1, i2);
        }
    }
}
