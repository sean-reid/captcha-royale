use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct BooleanLogicGenerator;

#[derive(Debug, Clone, Copy)]
enum Gate {
    And,
    Or,
    Not,
}

impl Gate {
    fn label(&self) -> &'static str {
        match self {
            Gate::And => "AND",
            Gate::Or => "OR",
            Gate::Not => "NOT",
        }
    }

    fn evaluate(&self, inputs: &[u8]) -> u8 {
        match self {
            Gate::And => {
                if inputs.contains(&1) && !inputs.contains(&0) { 1 } else { 0 }
            }
            Gate::Or => {
                if inputs.contains(&1) { 1 } else { 0 }
            }
            Gate::Not => {
                if inputs[0] == 0 { 1 } else { 0 }
            }
        }
    }

    fn input_count(&self) -> usize {
        match self {
            Gate::Not => 1,
            _ => 2,
        }
    }
}

/// A simple circuit: a chain of gates.
/// For low complexity: 1-2 gates with 2-3 inputs total.
/// For high complexity: 3 gates chained.
struct Circuit {
    inputs: Vec<u8>,
    gates: Vec<(Gate, Vec<usize>)>, // gate + indices into wires
    wires: Vec<u8>,                 // all values: inputs first, then gate outputs
}

impl Circuit {
    fn build(rng: &mut ChaCha8Rng, complexity: f32) -> Self {
        let gate_count = if complexity < 0.35 {
            1
        } else if complexity < 0.7 {
            2
        } else {
            3
        };

        // Pick input count: 2 for simple, 3 for complex
        let input_count = if gate_count <= 1 { 2 } else { 3 };
        let inputs: Vec<u8> = (0..input_count).map(|_| rng.gen_range(0..=1)).collect();
        let mut wires: Vec<u8> = inputs.clone();
        let mut gates: Vec<(Gate, Vec<usize>)> = Vec::new();

        // Build the gate chain ensuring ALL inputs are used
        let two_input_gates = [Gate::And, Gate::Or];
        let all_gates = [Gate::And, Gate::Or, Gate::Not];

        // Track which inputs have been used
        let mut used_inputs = vec![false; input_count];

        for g in 0..gate_count {
            let wire_count = wires.len();

            // Find unused raw inputs
            let unused: Vec<usize> = (0..input_count).filter(|&i| !used_inputs[i]).collect();

            let (gate, gate_inputs) = if g == 0 && input_count >= 2 {
                // First gate: always use a 2-input gate with inputs 0 and 1
                let gate = two_input_gates[rng.gen_range(0..two_input_gates.len())];
                used_inputs[0] = true;
                used_inputs[1] = true;
                (gate, vec![0_usize, 1])
            } else if unused.len() >= 2 {
                // Still have 2+ unused inputs — use a 2-input gate
                let gate = two_input_gates[rng.gen_range(0..two_input_gates.len())];
                let a = unused[0];
                let b = unused[1];
                used_inputs[a] = true;
                used_inputs[b] = true;
                (gate, vec![a, b])
            } else if unused.len() == 1 {
                // One unused input — combine it with previous gate output
                let gate = two_input_gates[rng.gen_range(0..two_input_gates.len())];
                let a = unused[0];
                used_inputs[a] = true;
                (gate, vec![a, wire_count - 1])
            } else {
                // All inputs used — can use NOT on previous output, or combine two wires
                let gate = all_gates[rng.gen_range(0..all_gates.len())];
                if gate.input_count() == 1 {
                    (gate, vec![wire_count - 1])
                } else {
                    let other = rng.gen_range(0..input_count);
                    (gate, vec![other, wire_count - 1])
                }
            };

            let input_vals: Vec<u8> = gate_inputs.iter().map(|&i| wires[i]).collect();
            let output = gate.evaluate(&input_vals);
            wires.push(output);
            gates.push((gate, gate_inputs));
        }

        Circuit {
            inputs,
            gates,
            wires,
        }
    }

    fn output(&self) -> u8 {
        *self.wires.last().unwrap()
    }
}

fn draw_gate_box(
    svg: &mut String,
    label: &str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) {
    write!(
        svg,
        r##"<rect x="{x:.1}" y="{y:.1}" width="{w:.0}" height="{h:.0}" rx="4" fill="#2c3e50" stroke="#7f8c8d" stroke-width="1.5"/>"##
    ).unwrap();
    write!(
        svg,
        r##"<text x="{:.1}" y="{:.1}" font-family="monospace" font-size="13" font-weight="bold" fill="#ecf0f1" text-anchor="middle" dominant-baseline="central">{label}</text>"##,
        x + w / 2.0,
        y + h / 2.0
    ).unwrap();
}

impl CaptchaGenerator for BooleanLogicGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        let circuit = Circuit::build(rng, difficulty.complexity);
        let correct_output = circuit.output() as f64;

        let input_count = circuit.inputs.len();
        let gate_count = circuit.gates.len();

        let gate_w = 60.0_f32;
        let gate_h = 36.0_f32;

        // Use fixed vertical lanes for inputs and gates
        // Each input gets its own horizontal row; gates get their own rows too
        // Total rows = max(input_count, gate_count + 1 for output)
        let row_height = 55.0_f32;
        let total_rows = input_count.max(gate_count + 1);
        let content_h = total_rows as f32 * row_height;
        let col_width = 130.0_f32;
        let width = 100.0 + (gate_count as f32 + 1.0) * col_width + 80.0;
        let width = width.max(400.0_f32);
        let height = content_h + 60.0;

        let mut svg = String::with_capacity(4096);
        write!(svg,
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width:.0} {height:.0}" width="{width:.0}" height="{height:.0}">"##
        ).unwrap();
        write!(svg, r##"<rect width="{width:.0}" height="{height:.0}" fill="#1a1a2e"/>"##).unwrap();

        write!(svg,
            r##"<text x="{:.0}" y="22" font-family="sans-serif" font-size="14" fill="#cccccc" text-anchor="middle">What is the output? (0 or 1)</text>"##,
            width / 2.0
        ).unwrap();

        // Noise
        let noise_count = (difficulty.noise * 6.0) as u32;
        for _ in 0..noise_count {
            let x1 = rng.gen_range(0.0..width);
            let y1 = rng.gen_range(30.0..height);
            let x2 = rng.gen_range(0.0..width);
            let y2 = rng.gen_range(30.0..height);
            let gray = rng.gen_range(30..80);
            write!(svg,
                r##"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="rgb({gray},{gray},{gray})" stroke-width="1" opacity="0.25"/>"##
            ).unwrap();
        }

        // Draw inputs — evenly spaced vertically in column 0
        let input_x = 30.0_f32;
        let content_top = 40.0_f32;

        let mut wire_positions: Vec<(f32, f32)> = Vec::new(); // (right_x, center_y)

        for i in 0..input_count {
            let iy = content_top + (i as f32 + 0.5) * (content_h / input_count as f32);
            let label = format!("{}={}", (b'A' + i as u8) as char, circuit.inputs[i]);
            write!(svg,
                r##"<rect x="{:.1}" y="{:.1}" width="50" height="28" rx="4" fill="#1e6b3a" stroke="#27ae60" stroke-width="1.5"/>"##,
                input_x, iy - 14.0
            ).unwrap();
            write!(svg,
                r##"<text x="{:.1}" y="{iy:.1}" font-family="monospace" font-size="14" font-weight="bold" fill="#ffffff" text-anchor="middle" dominant-baseline="central">{label}</text>"##,
                input_x + 25.0
            ).unwrap();
            // Extend a horizontal wire from the input to the right
            let wire_end_x = input_x + 50.0 + 20.0;
            write!(svg,
                r##"<line x1="{:.1}" y1="{iy:.1}" x2="{wire_end_x:.1}" y2="{iy:.1}" stroke="#7f8c8d" stroke-width="2"/>"##,
                input_x + 50.0
            ).unwrap();
            wire_positions.push((wire_end_x, iy));
        }

        // Two-pass: first compute all positions, then draw wires, then draw gates on top
        let gate_col_start = input_x + 120.0;

        // Pass 1: compute gate positions
        struct GateLayout { gx: f32, gy: f32, gate_idx: usize }
        let mut gate_layouts: Vec<GateLayout> = Vec::new();

        for g in 0..gate_count {
            let gx = gate_col_start + g as f32 * col_width;
            let (_, ref inputs_idx) = circuit.gates[g];

            let min_y = inputs_idx.iter().map(|&i| wire_positions[i].1).fold(f32::MAX, f32::min);
            let max_y = inputs_idx.iter().map(|&i| wire_positions[i].1).fold(f32::MIN, f32::max);
            let gy = (min_y + max_y) / 2.0;

            gate_layouts.push(GateLayout { gx, gy, gate_idx: g });
            let out_x = gx + gate_w + 20.0;
            wire_positions.push((out_x, gy));
        }

        // Pass 2: draw all wires (behind everything)
        for gl in &gate_layouts {
            let (_, ref inputs_idx) = circuit.gates[gl.gate_idx];
            for &idx in inputs_idx.iter() {
                let (from_x, from_y) = wire_positions[idx];
                let junc_x = gl.gx - 15.0;
                write!(svg,
                    r##"<path d="M {from_x:.1} {from_y:.1} H {junc_x:.1} V {:.1} H {:.1}" fill="none" stroke="#7f8c8d" stroke-width="2"/>"##,
                    gl.gy, gl.gx
                ).unwrap();
                write!(svg,
                    r##"<circle cx="{junc_x:.1}" cy="{from_y:.1}" r="2" fill="#7f8c8d"/>"##
                ).unwrap();
            }
            // Output wire
            write!(svg,
                r##"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="#7f8c8d" stroke-width="2"/>"##,
                gl.gx + gate_w, gl.gy, gl.gx + gate_w + 20.0, gl.gy
            ).unwrap();
        }

        // Pass 3: draw all gate boxes ON TOP of wires (solid fill covers wires)
        for gl in &gate_layouts {
            let (gate, _) = circuit.gates[gl.gate_idx];
            draw_gate_box(&mut svg, gate.label(), gl.gx, gl.gy - gate_h / 2.0, gate_w, gate_h);
        }

        // Output label
        let last_wire = wire_positions.last().unwrap();
        write!(svg,
            r##"<text x="{:.1}" y="{:.1}" font-family="monospace" font-size="16" font-weight="bold" fill="#f39c12" dominant-baseline="central">= ?</text>"##,
            last_wire.0 + 5.0, last_wire.1
        ).unwrap();

        // Show intermediate values at low difficulty
        if difficulty.complexity < 0.5 && gate_count > 1 {
            for g in 0..gate_count - 1 {
                let val = circuit.wires[input_count + g];
                let (wx, wy) = wire_positions[input_count + g];
                write!(svg,
                    r##"<text x="{:.1}" y="{:.1}" font-family="monospace" font-size="10" fill="#95a5a6" text-anchor="middle">{val}</text>"##,
                    wx - 10.0, wy - 14.0
                ).unwrap();
            }
        }

        // Show the truth of the circuit for debug rendering
        // (label showing expected answer is NOT rendered -- player must compute it)

        svg.push_str("</svg>");

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::Number(correct_output),
            expected_solve_time_ms: expected_solve_time(difficulty.time_limit_ms),
            point_value: CaptchaType::BooleanLogic.base_points(),
            captcha_type: CaptchaType::BooleanLogic,
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
    fn test_booleanlogic_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 7000,
            complexity: 0.0,
            noise: 0.0,
        };
        let gen = BooleanLogicGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert_eq!(instance.captcha_type, CaptchaType::BooleanLogic);
        if let Solution::Number(n) = instance.solution {
            assert!(n == 0.0 || n == 1.0, "Output must be 0 or 1, got {n}");
        } else {
            panic!("Expected Number solution");
        }
        if let RenderPayload::Svg(ref s) = instance.render_data {
            assert!(s.contains("<svg"));
            assert!(s.contains("= ?"));
            assert!(s.contains("What is the output"));
        } else {
            panic!("Expected SVG render data");
        }
    }

    #[test]
    fn test_booleanlogic_seed_determinism() {
        let difficulty = DifficultyParams {
            level: 10,
            round_number: 3,
            time_limit_ms: 9000,
            complexity: 0.5,
            noise: 0.3,
        };
        let gen = BooleanLogicGenerator;

        let mut rng1 = rng_from_seed(12345);
        let inst1 = gen.generate(&mut rng1, &difficulty);

        let mut rng2 = rng_from_seed(12345);
        let inst2 = gen.generate(&mut rng2, &difficulty);

        if let (Solution::Number(n1), Solution::Number(n2)) =
            (&inst1.solution, &inst2.solution)
        {
            assert_eq!(n1, n2);
        }
    }

    #[test]
    fn test_booleanlogic_correct_answer_validates() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 7000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = BooleanLogicGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::Number(n) = instance.solution {
            assert!(gen.validate(&instance, &PlayerAnswer::Number(n)));
        }
    }

    #[test]
    fn test_booleanlogic_wrong_answer_rejects() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 7000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = BooleanLogicGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::Number(n) = instance.solution {
            // Submit the opposite answer
            let wrong = if n == 0.0 { 1.0 } else { 0.0 };
            assert!(!gen.validate(&instance, &PlayerAnswer::Number(wrong)));
        }
        assert!(!gen.validate(&instance, &PlayerAnswer::Text("wrong".to_string())));
    }

    #[test]
    fn test_booleanlogic_gate_evaluation() {
        // AND gate
        assert_eq!(Gate::And.evaluate(&[1, 1]), 1);
        assert_eq!(Gate::And.evaluate(&[1, 0]), 0);
        assert_eq!(Gate::And.evaluate(&[0, 0]), 0);
        // OR gate
        assert_eq!(Gate::Or.evaluate(&[0, 0]), 0);
        assert_eq!(Gate::Or.evaluate(&[1, 0]), 1);
        assert_eq!(Gate::Or.evaluate(&[0, 1]), 1);
        // NOT gate
        assert_eq!(Gate::Not.evaluate(&[0]), 1);
        assert_eq!(Gate::Not.evaluate(&[1]), 0);
    }
}
