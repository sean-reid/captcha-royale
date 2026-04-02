use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct SpatialGenerator;

/// Face identifiers for a cube: Top, Front, Right, Bottom, Back, Left
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Face {
    Top,
    Front,
    Right,
    Bottom,
    Back,
    Left,
}

#[allow(dead_code)]
const ALL_FACES: [Face; 6] = [
    Face::Top,
    Face::Front,
    Face::Right,
    Face::Bottom,
    Face::Back,
    Face::Left,
];

/// A color + pattern combo for a cube face
#[derive(Debug, Clone)]
struct FaceStyle {
    fill: String,
    pattern: FacePattern,
}

#[derive(Debug, Clone, Copy)]
enum FacePattern {
    Solid,
    Stripes,
    Dots,
    Cross,
    Diagonal,
}

const FACE_COLORS: [&str; 8] = [
    "#e74c3c", "#3498db", "#2ecc71", "#f39c12", "#9b59b6", "#1abc9c", "#e67e22", "#e91e63",
];

const PATTERNS: [FacePattern; 5] = [
    FacePattern::Solid,
    FacePattern::Stripes,
    FacePattern::Dots,
    FacePattern::Cross,
    FacePattern::Diagonal,
];

/// Generate unique face styles for all 6 faces of a cube.
fn generate_face_styles(rng: &mut ChaCha8Rng) -> [FaceStyle; 6] {
    // Pick 6 distinct colors
    let mut color_indices: Vec<usize> = (0..FACE_COLORS.len()).collect();
    // Fisher-Yates partial shuffle for first 6
    for i in 0..6 {
        let j = rng.gen_range(i..color_indices.len());
        color_indices.swap(i, j);
    }

    let mut styles = Vec::with_capacity(6);
    for i in 0..6 {
        styles.push(FaceStyle {
            fill: FACE_COLORS[color_indices[i]].to_string(),
            pattern: PATTERNS[rng.gen_range(0..PATTERNS.len())],
        });
    }
    styles.try_into().unwrap()
}

/// Render a pattern overlay inside a rectangular area, returning SVG elements.
fn render_pattern(pattern: FacePattern, x: f32, y: f32, w: f32, h: f32) -> String {
    let mut s = String::new();
    match pattern {
        FacePattern::Solid => {} // no overlay
        FacePattern::Stripes => {
            let count = 4;
            let spacing = w / (count as f32 + 1.0);
            for i in 1..=count {
                let lx = x + spacing * i as f32;
                write!(
                    s,
                    r#"<line x1="{lx:.1}" y1="{y:.1}" x2="{lx:.1}" y2="{:.1}" stroke="rgba(0,0,0,0.3)" stroke-width="2"/>"#,
                    y + h
                )
                .unwrap();
            }
        }
        FacePattern::Dots => {
            let cols = 3;
            let rows = 3;
            let dx = w / (cols as f32 + 1.0);
            let dy = h / (rows as f32 + 1.0);
            for r in 1..=rows {
                for c in 1..=cols {
                    let cx = x + dx * c as f32;
                    let cy = y + dy * r as f32;
                    write!(
                        s,
                        r#"<circle cx="{cx:.1}" cy="{cy:.1}" r="2.5" fill="rgba(0,0,0,0.35)"/>"#
                    )
                    .unwrap();
                }
            }
        }
        FacePattern::Cross => {
            let cx = x + w / 2.0;
            let cy = y + h / 2.0;
            let arm = w.min(h) * 0.35;
            write!(
                s,
                r#"<line x1="{:.1}" y1="{cy:.1}" x2="{:.1}" y2="{cy:.1}" stroke="rgba(0,0,0,0.35)" stroke-width="3"/>"#,
                cx - arm,
                cx + arm
            )
            .unwrap();
            write!(
                s,
                r#"<line x1="{cx:.1}" y1="{:.1}" x2="{cx:.1}" y2="{:.1}" stroke="rgba(0,0,0,0.35)" stroke-width="3"/>"#,
                cy - arm,
                cy + arm
            )
            .unwrap();
        }
        FacePattern::Diagonal => {
            write!(
                s,
                r#"<line x1="{x:.1}" y1="{y:.1}" x2="{:.1}" y2="{:.1}" stroke="rgba(0,0,0,0.3)" stroke-width="2"/>"#,
                x + w,
                y + h
            )
            .unwrap();
            write!(
                s,
                r#"<line x1="{:.1}" y1="{y:.1}" x2="{x:.1}" y2="{:.1}" stroke="rgba(0,0,0,0.3)" stroke-width="2"/>"#,
                x + w,
                y + h
            )
            .unwrap();
        }
    }
    s
}

/// Draw a flat square face in the unfolded net.
fn draw_net_face(style: &FaceStyle, x: f32, y: f32, size: f32) -> String {
    let mut s = String::new();
    write!(
        s,
        r##"<rect x="{x:.1}" y="{y:.1}" width="{size:.1}" height="{size:.1}" fill="{}" stroke="#333" stroke-width="1.5"/>"##,
        style.fill
    )
    .unwrap();
    s.push_str(&render_pattern(style.pattern, x, y, size, size));
    s
}

/// Draw one face of an isometric cube.
/// `face_type`: 0 = top, 1 = left-visible, 2 = right-visible
fn draw_iso_face(style: &FaceStyle, cx: f32, cy: f32, size: f32, face_type: u8) -> String {
    let half = size / 2.0;
    let h_quarter = size * 0.29; // isometric vertical scaling

    let points = match face_type {
        0 => {
            // Top face
            format!(
                "{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
                cx,
                cy - half,
                cx + half,
                cy - half + h_quarter,
                cx,
                cy - half + 2.0 * h_quarter,
                cx - half,
                cy - half + h_quarter
            )
        }
        1 => {
            // Left face
            format!(
                "{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
                cx - half,
                cy - half + h_quarter,
                cx,
                cy - half + 2.0 * h_quarter,
                cx,
                cy + half,
                cx - half,
                cy + half - h_quarter
            )
        }
        _ => {
            // Right face
            format!(
                "{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
                cx,
                cy - half + 2.0 * h_quarter,
                cx + half,
                cy - half + h_quarter,
                cx + half,
                cy + half - h_quarter,
                cx,
                cy + half
            )
        }
    };

    // Darken left/right faces slightly for depth
    let opacity = match face_type {
        0 => 1.0,
        1 => 0.85,
        _ => 0.7,
    };

    let mut s = String::new();
    write!(
        s,
        r##"<polygon points="{points}" fill="{}" stroke="#333" stroke-width="1" opacity="{opacity:.2}"/>"##,
        style.fill
    )
    .unwrap();
    s
}

/// Draw an isometric cube showing three visible faces.
/// `visible` is [top_style, left_style, right_style].
fn draw_iso_cube(visible: [&FaceStyle; 3], cx: f32, cy: f32, size: f32) -> String {
    let mut s = String::new();
    // Draw back-to-front: left, right, top
    s.push_str(&draw_iso_face(visible[1], cx, cy, size, 1));
    s.push_str(&draw_iso_face(visible[2], cx, cy, size, 2));
    s.push_str(&draw_iso_face(visible[0], cx, cy, size, 0));
    s
}

/// For the correct cube, the net cross maps as:
///   Net layout (cross):
///        [Top]
///   [Left][Front][Right]
///        [Bottom]
///        [Back]
///
/// Isometric view shows: Top, Front (left-visible), Right (right-visible)
fn correct_visible_faces(styles: &[FaceStyle; 6]) -> [&FaceStyle; 3] {
    [
        &styles[Face::Top as usize],
        &styles[Face::Front as usize],
        &styles[Face::Right as usize],
    ]
}

/// Generate wrong cube by swapping/rotating faces.
fn wrong_visible_faces<'a>(
    rng: &mut ChaCha8Rng,
    styles: &'a [FaceStyle; 6],
    _variant: usize,
) -> [&'a FaceStyle; 3] {
    // Produce a permutation of three faces that differs from the correct one
    let correct = [
        Face::Top as usize,
        Face::Front as usize,
        Face::Right as usize,
    ];

    // Possible wrong combos: swap two faces, or use wrong faces entirely
    let options: Vec<[usize; 3]> = vec![
        [Face::Top as usize, Face::Right as usize, Face::Front as usize],     // swap front/right
        [Face::Bottom as usize, Face::Front as usize, Face::Right as usize],  // wrong top
        [Face::Top as usize, Face::Left as usize, Face::Right as usize],      // wrong front
        [Face::Top as usize, Face::Front as usize, Face::Back as usize],      // wrong right
        [Face::Back as usize, Face::Left as usize, Face::Front as usize],     // all wrong
        [Face::Bottom as usize, Face::Right as usize, Face::Left as usize],   // all wrong v2
        [Face::Front as usize, Face::Top as usize, Face::Back as usize],      // rotated confusion
        [Face::Right as usize, Face::Back as usize, Face::Top as usize],      // another wrong combo
    ];

    // Filter out any that accidentally match correct
    let wrong: Vec<_> = options
        .into_iter()
        .filter(|o| o != &correct)
        .collect();

    let chosen = wrong[rng.gen_range(0..wrong.len())];
    [
        &styles[chosen[0]],
        &styles[chosen[1]],
        &styles[chosen[2]],
    ]
}

impl CaptchaGenerator for SpatialGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        let styles = generate_face_styles(rng);

        // Option count scales: 3 at low complexity, up to 5 at high
        let option_count = 3 + (difficulty.complexity * 2.0) as usize;
        let option_count = option_count.min(5);

        let correct_idx = rng.gen_range(0..option_count);

        // Layout constants
        let net_face_size: f32 = 50.0;
        let iso_cube_size: f32 = 70.0;
        let net_width: f32 = net_face_size * 4.0 + 20.0; // cross is 4 wide (with left)
        let net_height: f32 = net_face_size * 4.0 + 20.0; // cross is 4 tall
        let options_width: f32 = option_count as f32 * (iso_cube_size + 30.0) + 30.0;
        let total_width: f32 = net_width.max(options_width).max(400.0);
        let total_height: f32 = net_height + iso_cube_size + 100.0;

        let mut svg = String::with_capacity(8192);
        write!(
            svg,
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {total_width:.0} {total_height:.0}" width="{total_width:.0}" height="{total_height:.0}">"#
        )
        .unwrap();

        write!(
            svg,
            r##"<rect width="{total_width:.0}" height="{total_height:.0}" fill="#1a1a2e"/>"##
        )
        .unwrap();

        // Prompt
        write!(
            svg,
            r##"<text x="{:.0}" y="22" font-family="sans-serif" font-size="14" fill="#cccccc" text-anchor="middle">Which 3D cube does this net fold into?</text>"##,
            total_width / 2.0
        )
        .unwrap();

        // Add noise lines based on difficulty
        let noise_count = (2.0 + difficulty.noise * 8.0) as u32;
        for _ in 0..noise_count {
            let x1 = rng.gen::<f32>() * total_width;
            let y1 = rng.gen::<f32>() * total_height;
            let x2 = rng.gen::<f32>() * total_width;
            let y2 = rng.gen::<f32>() * total_height;
            let r = rng.gen_range(40..120);
            let g = rng.gen_range(40..120);
            let b = rng.gen_range(40..120);
            write!(
                svg,
                r#"<line x1="{x1:.0}" y1="{y1:.0}" x2="{x2:.0}" y2="{y2:.0}" stroke="rgb({r},{g},{b})" stroke-width="1" opacity="0.15"/>"#
            )
            .unwrap();
        }

        // Draw the unfolded net (cross shape) centered horizontally
        // Layout:
        //          [Top]        row 0, col 1
        //   [Left][Front][Right] row 1, col 0,1,2
        //          [Bottom]     row 2, col 1
        //          [Back]       row 3, col 1
        let net_origin_x = (total_width - net_face_size * 3.0) / 2.0;
        let net_origin_y = 35.0;

        let face_positions: [(Face, f32, f32); 6] = [
            (Face::Top, net_origin_x + net_face_size, net_origin_y),
            (Face::Left, net_origin_x, net_origin_y + net_face_size),
            (
                Face::Front,
                net_origin_x + net_face_size,
                net_origin_y + net_face_size,
            ),
            (
                Face::Right,
                net_origin_x + 2.0 * net_face_size,
                net_origin_y + net_face_size,
            ),
            (
                Face::Bottom,
                net_origin_x + net_face_size,
                net_origin_y + 2.0 * net_face_size,
            ),
            (
                Face::Back,
                net_origin_x + net_face_size,
                net_origin_y + 3.0 * net_face_size,
            ),
        ];

        // Label each face on the net for clarity
        for (face, fx, fy) in &face_positions {
            svg.push_str(&draw_net_face(
                &styles[*face as usize],
                *fx,
                *fy,
                net_face_size,
            ));
        }

        // Options section
        let options_label_y = net_origin_y + 4.0 * net_face_size + 20.0;
        write!(
            svg,
            r##"<text x="{:.0}" y="{options_label_y:.0}" font-family="sans-serif" font-size="12" fill="#999" text-anchor="middle">Select the matching cube:</text>"##,
            total_width / 2.0
        )
        .unwrap();

        let opt_start_x = (total_width - options_width) / 2.0 + 30.0;
        let opt_y = options_label_y + 20.0 + iso_cube_size / 2.0;

        for i in 0..option_count {
            let cx = opt_start_x + i as f32 * (iso_cube_size + 30.0) + iso_cube_size / 2.0;

            let visible = if i == correct_idx {
                correct_visible_faces(&styles)
            } else {
                wrong_visible_faces(rng, &styles, i)
            };

            svg.push_str(&draw_iso_cube(visible, cx, opt_y, iso_cube_size));

            // Option index label
            write!(
                svg,
                r##"<text x="{cx:.0}" y="{:.0}" font-family="sans-serif" font-size="11" fill="#888" text-anchor="middle">{}</text>"##,
                opt_y + iso_cube_size / 2.0 + 16.0,
                (b'A' + i as u8) as char
            )
            .unwrap();

            // Clickable overlay with data-index
            let overlay_x = cx - iso_cube_size / 2.0;
            let overlay_y = opt_y - iso_cube_size / 2.0;
            write!(
                svg,
                r#"<rect x="{overlay_x:.0}" y="{overlay_y:.0}" width="{iso_cube_size:.0}" height="{iso_cube_size:.0}" fill="transparent" data-index="{i}" style="cursor:pointer"/>"#
            )
            .unwrap();
        }

        svg.push_str("</svg>");

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::SelectedIndices(vec![correct_idx as u32]),
            expected_solve_time_ms: expected_solve_time(difficulty.time_limit_ms),
            point_value: CaptchaType::SpatialReasoning.base_points(),
            captcha_type: CaptchaType::SpatialReasoning,
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
    fn test_spatial_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 8000,
            complexity: 0.0,
            noise: 0.0,
        };
        let gen = SpatialGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert_eq!(instance.captcha_type, CaptchaType::SpatialReasoning);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert_eq!(indices.len(), 1);
            assert!(indices[0] < 5);
        } else {
            panic!("Expected SelectedIndices solution");
        }
        if let RenderPayload::Svg(ref s) = instance.render_data {
            assert!(s.contains("<svg"));
            assert!(s.contains("data-index"));
            assert!(s.contains("Which 3D cube"));
        } else {
            panic!("Expected SVG render data");
        }
    }

    #[test]
    fn test_spatial_seed_determinism() {
        let difficulty = DifficultyParams {
            level: 10,
            round_number: 3,
            time_limit_ms: 9000,
            complexity: 0.5,
            noise: 0.3,
        };
        let gen = SpatialGenerator;

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
    fn test_spatial_correct_answer_validates() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 8000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = SpatialGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::SelectedIndices(ref indices) = instance.solution {
            assert!(gen.validate(
                &instance,
                &PlayerAnswer::SelectedIndices(indices.clone())
            ));
        }
    }

    #[test]
    fn test_spatial_wrong_answer_rejects() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 8000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = SpatialGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![99])));
        assert!(!gen.validate(&instance, &PlayerAnswer::SelectedIndices(vec![])));
        assert!(!gen.validate(&instance, &PlayerAnswer::Text("wrong".to_string())));
    }

    #[test]
    fn test_spatial_difficulty_scaling() {
        let gen = SpatialGenerator;
        let low = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 8000,
            complexity: 0.0,
            noise: 0.0,
        };
        let high = DifficultyParams {
            level: 50,
            round_number: 10,
            time_limit_ms: 15000,
            complexity: 1.0,
            noise: 1.0,
        };

        let mut rng1 = rng_from_seed(42);
        let inst_low = gen.generate(&mut rng1, &low);
        let mut rng2 = rng_from_seed(42);
        let inst_high = gen.generate(&mut rng2, &high);

        // Higher difficulty = more options = longer SVG
        if let (RenderPayload::Svg(svg_low), RenderPayload::Svg(svg_high)) =
            (&inst_low.render_data, &inst_high.render_data)
        {
            assert!(svg_high.len() > svg_low.len());
        }
    }
}
