use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct CubeFoldingGenerator;

/// Face identifiers for the cube net.
/// Standard cross layout:
/// ```text
///       [Top]
/// [Left][Front][Right]
///       [Bottom]
///       [Back]
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Face {
    Top,
    Left,
    Front,
    Right,
    Bottom,
    Back,
}

const ALL_FACES: [Face; 6] = [
    Face::Top,
    Face::Left,
    Face::Front,
    Face::Right,
    Face::Bottom,
    Face::Back,
];

/// Symbols and colors for each face
const FACE_LABELS: [char; 6] = ['A', 'B', 'C', 'D', 'E', 'F'];
const FACE_COLORS: [&str; 6] = [
    "#e74c3c", // red
    "#3498db", // blue
    "#2ecc71", // green
    "#f39c12", // orange
    "#9b59b6", // purple
    "#1abc9c", // teal
];

/// Assignment of labels/colors to each face
#[derive(Debug, Clone)]
struct FaceAssignment {
    label: char,
    color: &'static str,
}

/// Which 3 faces are visible on an isometric cube view (front-right perspective)
/// In a standard isometric view you see: Top, Front (left-visible), Right (right-visible)
struct CubeView {
    top: usize,    // index into assignments
    left: usize,   // front face (shown on left side of cube)
    right: usize,  // right face (shown on right side of cube)
}

/// When folded into a cube (standard cross net), the spatial relationships are:
/// - Top is opposite Bottom
/// - Left is opposite Right
/// - Front is opposite Back
///
/// For an isometric view showing top + front + right:
///   top_face = Top (index 0)
///   left_visible_face = Front (index 2)
///   right_visible_face = Right (index 3)
fn correct_cube_view(_assignments: &[FaceAssignment]) -> CubeView {
    // Standard isometric: see Top, Front, Right
    CubeView {
        top: 0,   // Top
        left: 2,  // Front
        right: 3, // Right
    }
}

/// Generate a wrong cube view by swapping one face with its opposite or neighbor
fn wrong_cube_view(
    _assignments: &[FaceAssignment],
    rng: &mut ChaCha8Rng,
    difficulty_high: bool,
) -> CubeView {
    let correct = CubeView {
        top: 0,
        left: 2,
        right: 3,
    };

    if difficulty_high {
        // Subtle: swap two adjacent visible faces, or swap one visible with its opposite
        let swap_type = rng.gen_range(0..3);
        match swap_type {
            0 => CubeView {
                top: correct.top,
                left: correct.right,  // front and right swapped
                right: correct.left,
            },
            1 => CubeView {
                top: 4,              // Bottom instead of Top (opposite)
                left: correct.left,
                right: correct.right,
            },
            _ => CubeView {
                top: correct.top,
                left: 5,             // Back instead of Front (opposite)
                right: correct.right,
            },
        }
    } else {
        // Obvious: use clearly wrong faces
        let swap_type = rng.gen_range(0..3);
        match swap_type {
            0 => CubeView {
                top: 4,   // Bottom
                left: 5,  // Back
                right: 1, // Left
            },
            1 => CubeView {
                top: 1,   // Left
                left: 4,  // Bottom
                right: 5, // Back
            },
            _ => CubeView {
                top: 5,   // Back
                left: 1,  // Left
                right: 4, // Bottom
            },
        }
    }
}

/// Draw the cross-shaped unfolded net
fn draw_net(svg: &mut String, assignments: &[FaceAssignment], offset_x: f32, offset_y: f32) {
    let face_size: f32 = 50.0;
    let gap: f32 = 2.0;
    let step = face_size + gap;

    // Cross layout positions (col, row) for each face:
    // Top:    (1, 0)
    // Left:   (0, 1)
    // Front:  (1, 1)
    // Right:  (2, 1)
    // Bottom: (1, 2)
    // Back:   (1, 3)
    let positions: [(f32, f32); 6] = [
        (1.0, 0.0), // Top
        (0.0, 1.0), // Left
        (1.0, 1.0), // Front
        (2.0, 1.0), // Right
        (1.0, 2.0), // Bottom
        (1.0, 3.0), // Back
    ];

    for (i, (col, row)) in positions.iter().enumerate() {
        let x = offset_x + col * step;
        let y = offset_y + row * step;
        let a = &assignments[i];

        // Face rectangle
        write!(
            svg,
            r##"<rect x="{x:.1}" y="{y:.1}" width="{face_size:.0}" height="{face_size:.0}" fill="{}" stroke="#333" stroke-width="1.5"/>"##,
            a.color
        )
        .unwrap();

        // Face label
        let tx = x + face_size / 2.0;
        let ty = y + face_size / 2.0;
        write!(
            svg,
            r##"<text x="{tx:.1}" y="{ty:.1}" font-family="sans-serif" font-size="24" font-weight="bold" fill="white" text-anchor="middle" dominant-baseline="central">{}</text>"##,
            a.label
        )
        .unwrap();
    }
}

/// Draw an isometric cube showing 3 faces.
/// Uses parallelograms for top, left-front, right-front faces.
fn draw_isometric_cube(
    svg: &mut String,
    assignments: &[FaceAssignment],
    view: &CubeView,
    cx: f32,
    cy: f32,
    size: f32,
) {
    // Isometric projection constants (30-degree angles)
    let dx = size * 0.866; // cos(30)
    let dy = size * 0.5;   // sin(30)

    // Key points of the isometric cube:
    // center-top of cube
    let top_x = cx;
    let top_y = cy - size;

    // top-left
    let tl_x = cx - dx;
    let tl_y = cy - size + dy;

    // top-right
    let tr_x = cx + dx;
    let tr_y = cy - size + dy;

    // mid-left
    let ml_x = cx - dx;
    let ml_y = cy + dy;

    // mid-right
    let mr_x = cx + dx;
    let mr_y = cy + dy;

    // bottom-center
    let bot_x = cx;
    let bot_y = cy;

    // Draw top face (parallelogram: top -> tl -> center -> tr)
    let top_assign = &assignments[view.top];
    write!(
        svg,
        r##"<polygon points="{top_x:.1},{top_y:.1} {tl_x:.1},{tl_y:.1} {cx:.1},{cy:.1} {tr_x:.1},{tr_y:.1}" fill="{}" stroke="#222" stroke-width="1.5"/>"##,
        top_assign.color
    )
    .unwrap();

    // Top face label (center of the parallelogram)
    let top_label_x = (top_x + tl_x + cx + tr_x) / 4.0;
    let top_label_y = (top_y + tl_y + cy + tr_y) / 4.0;
    write!(
        svg,
        r##"<text x="{top_label_x:.1}" y="{top_label_y:.1}" font-family="sans-serif" font-size="16" font-weight="bold" fill="white" text-anchor="middle" dominant-baseline="central">{}</text>"##,
        top_assign.label
    )
    .unwrap();

    // Draw left face (parallelogram: tl -> ml -> bot -> center)
    let left_assign = &assignments[view.left];
    write!(
        svg,
        r##"<polygon points="{tl_x:.1},{tl_y:.1} {ml_x:.1},{ml_y:.1} {bot_x:.1},{bot_y:.1} {cx:.1},{cy:.1}" fill="{}" stroke="#222" stroke-width="1.5"/>"##,
        left_assign.color
    )
    .unwrap();

    let left_label_x = (tl_x + ml_x + bot_x + cx) / 4.0;
    let left_label_y = (tl_y + ml_y + bot_y + cy) / 4.0;
    write!(
        svg,
        r##"<text x="{left_label_x:.1}" y="{left_label_y:.1}" font-family="sans-serif" font-size="16" font-weight="bold" fill="white" text-anchor="middle" dominant-baseline="central">{}</text>"##,
        left_assign.label
    )
    .unwrap();

    // Draw right face (parallelogram: tr -> center -> bot -> mr)
    let right_assign = &assignments[view.right];
    write!(
        svg,
        r##"<polygon points="{tr_x:.1},{tr_y:.1} {cx:.1},{cy:.1} {bot_x:.1},{bot_y:.1} {mr_x:.1},{mr_y:.1}" fill="{}" stroke="#222" stroke-width="1.5"/>"##,
        right_assign.color
    )
    .unwrap();

    let right_label_x = (tr_x + cx + bot_x + mr_x) / 4.0;
    let right_label_y = (tr_y + cy + bot_y + mr_y) / 4.0;
    write!(
        svg,
        r##"<text x="{right_label_x:.1}" y="{right_label_y:.1}" font-family="sans-serif" font-size="16" font-weight="bold" fill="white" text-anchor="middle" dominant-baseline="central">{}</text>"##,
        right_assign.label
    )
    .unwrap();
}

impl CaptchaGenerator for CubeFoldingGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        let option_count = if difficulty.complexity < 0.5 { 3 } else { 4 };
        let high_difficulty = difficulty.complexity >= 0.5;

        // Create randomized face assignments by shuffling labels and colors together
        let mut indices: Vec<usize> = (0..6).collect();
        for i in (1..6).rev() {
            let j = rng.gen_range(0..=i);
            indices.swap(i, j);
        }

        let assignments: Vec<FaceAssignment> = indices
            .iter()
            .map(|&i| FaceAssignment {
                label: FACE_LABELS[i],
                color: FACE_COLORS[i],
            })
            .collect();

        // Correct answer position
        let correct_option_idx = rng.gen_range(0..option_count);

        // Generate views for each option
        let mut views: Vec<CubeView> = Vec::with_capacity(option_count);
        for i in 0..option_count {
            if i == correct_option_idx {
                views.push(correct_cube_view(&assignments));
            } else {
                views.push(wrong_cube_view(&assignments, rng, high_difficulty));
            }
        }

        // --- SVG Rendering ---
        let net_width: f32 = 3.0 * 52.0; // 3 columns * (50 + 2 gap)
        let net_height: f32 = 4.0 * 52.0; // 4 rows

        let cube_size: f32 = 35.0; // half-size for isometric projection
        let cube_spacing: f32 = 100.0;
        let cubes_total_width = option_count as f32 * cube_spacing;

        let total_width = (net_width + 40.0).max(cubes_total_width + 40.0).max(400.0);
        let total_height = 30.0 + net_height + 30.0 + cube_size * 2.0 + 60.0 + 20.0;

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
            r##"<text x="{:.0}" y="22" font-family="sans-serif" font-size="14" fill="#cccccc" text-anchor="middle">Which cube can be made from this net?</text>"##,
            total_width / 2.0
        )
        .unwrap();

        // Add noise lines
        let noise_count = (2.0 + difficulty.noise * 5.0) as u32;
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

        // Draw the unfolded net (centered)
        let net_offset_x = (total_width - net_width) / 2.0;
        let net_offset_y: f32 = 35.0;
        draw_net(&mut svg, &assignments, net_offset_x, net_offset_y);

        // Options label
        let options_label_y = net_offset_y + net_height + 20.0;
        write!(
            svg,
            r##"<text x="{:.0}" y="{options_label_y:.0}" font-family="sans-serif" font-size="12" fill="#999" text-anchor="middle">Select the correct cube:</text>"##,
            total_width / 2.0
        )
        .unwrap();

        // Draw isometric cube options
        let cubes_offset_x = (total_width - cubes_total_width) / 2.0 + cube_spacing / 2.0;
        let cubes_y = options_label_y + 15.0 + cube_size + 10.0;

        for (i, view) in views.iter().enumerate() {
            let cx = cubes_offset_x + i as f32 * cube_spacing;

            // Option background box
            let box_x = cx - cube_spacing / 2.0 + 5.0;
            let box_y = cubes_y - cube_size - 15.0;
            let box_w = cube_spacing - 10.0;
            let box_h = cube_size * 2.0 + 20.0;
            write!(
                svg,
                r##"<rect x="{box_x:.1}" y="{box_y:.1}" width="{box_w:.0}" height="{box_h:.0}" fill="#252545" stroke="#444" stroke-width="1" rx="4"/>"##
            )
            .unwrap();

            draw_isometric_cube(&mut svg, &assignments, view, cx, cubes_y, cube_size);

            // Clickable overlay
            write!(
                svg,
                r##"<rect x="{box_x:.1}" y="{box_y:.1}" width="{box_w:.0}" height="{box_h:.0}" fill="transparent" data-index="{i}" style="cursor:pointer"/>"##
            )
            .unwrap();
        }

        svg.push_str("</svg>");

        let time_limit = difficulty.time_limit_ms.clamp(10000, 25000);

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::SelectedIndices(vec![correct_option_idx as u32]),
            expected_solve_time_ms: expected_solve_time(time_limit),
            point_value: CaptchaType::ProceduralNovelType.base_points(),
            captcha_type: CaptchaType::ProceduralNovelType,
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
    fn test_cubefolding_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 15000,
            complexity: 0.0,
            noise: 0.0,
        };
        let gen = CubeFoldingGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert_eq!(instance.captcha_type, CaptchaType::ProceduralNovelType);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert_eq!(indices.len(), 1);
            assert!(indices[0] < 3); // low complexity = 3 options
        } else {
            panic!("Expected SelectedIndices solution");
        }
        if let RenderPayload::Svg(ref s) = instance.render_data {
            assert!(s.contains("<svg"));
            assert!(s.contains("Which cube"));
            assert!(s.contains("data-index"));
        } else {
            panic!("Expected SVG render data");
        }
    }

    #[test]
    fn test_cubefolding_seed_determinism() {
        let difficulty = DifficultyParams {
            level: 10,
            round_number: 3,
            time_limit_ms: 18000,
            complexity: 0.5,
            noise: 0.3,
        };
        let gen = CubeFoldingGenerator;

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
    fn test_cubefolding_correct_answer_validates() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 15000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = CubeFoldingGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert!(gen.validate(
                &instance,
                &PlayerAnswer::SelectedIndices(indices.clone())
            ));
        }
    }

    #[test]
    fn test_cubefolding_wrong_answer_rejects() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 15000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = CubeFoldingGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![99])));
        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![])));
        assert!(!gen.validate(&instance, &PlayerAnswer::Text("wrong".to_string())));
    }

    #[test]
    fn test_cubefolding_high_complexity_more_options() {
        let gen = CubeFoldingGenerator;
        let low = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 15000,
            complexity: 0.0,
            noise: 0.0,
        };
        let high = DifficultyParams {
            level: 50,
            round_number: 10,
            time_limit_ms: 22000,
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
