use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::fmt::Write;

use crate::difficulty::expected_solve_time;
use crate::types::*;

use super::CaptchaGenerator;

pub struct UnscrambleGenerator;

// Word lists curated to avoid common anagram alternatives.
// Each word should have only ONE common English unscrambling.
const WORDS_EASY: &[&str] = &[
    "CUP", "DOG", "SUN", "HAT", "PEN", "BOX", "JAR", "FAN", "BUG", "HUG",
    "FISH", "BOOK", "TREE", "BIRD", "CAKE", "FROG", "LAMP", "BELL", "MOON", "MILK",
    "COIN", "DUCK", "RAIN", "DRUM", "KING", "WOLF", "GOLD", "FERN", "CRAB", "SHIP",
];

const WORDS_MID: &[&str] = &[
    "APPLE", "HOUSE", "CHAIR", "WATER", "PLANT", "LIGHT", "BREAD", "BLAZE", "GRAIN", "BLOOM",
    "SPARK", "GLOBE", "TIGER", "WHEEL", "BRUSH", "PRISM", "TOWER", "CABIN", "TRUCK", "BRAIN",
    "CLIFF", "DWARF", "FOGGY", "GHOST", "HIPPO", "JEWEL", "KAYAK", "LUNCH", "MOOSE", "OLIVE",
    "PLUMB", "VINYL", "YACHT", "WALTZ", "THUMB", "PIANO", "IGLOO", "MANGO", "PUPPY", "SQUID",
];

const WORDS_HARD: &[&str] = &[
    "BLANKET", "CAPTAIN", "CHIMNEY", "COMPASS", "DOLPHIN", "ELEMENT", "FEATHER",
    "GATEWAY", "KITCHEN", "LANTERN", "MACHINE", "NETWORK", "ORCHARD", "PANTHER",
    "RAINBOW", "VOLCANO", "WEATHER", "WHISTLE", "CABINET", "CRYSTAL",
    "FURNACE", "JOURNEY", "LIBRARY", "MYSTERY", "BUFFALO", "GIRAFFE", "PENGUIN",
    "PUMPKIN", "MAMMOTH", "JUKEBOX", "BISCUIT", "KETCHUP", "GONDOLA", "HAMMOCK",
];

fn scramble_word(word: &str, rng: &mut ChaCha8Rng) -> String {
    let mut chars: Vec<char> = word.chars().collect();
    let original = chars.clone();

    // Fisher-Yates shuffle, retry if result equals original
    for attempt in 0..20 {
        for i in (1..chars.len()).rev() {
            let j = rng.gen_range(0..=i);
            chars.swap(i, j);
        }
        if chars != original || attempt == 19 {
            break;
        }
    }
    chars.iter().collect()
}

impl CaptchaGenerator for UnscrambleGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> CaptchaInstance {
        // Pick word based on complexity: 3-4 letters → 5 letters → 7 letters
        let word = if difficulty.complexity < 0.35 {
            WORDS_EASY[rng.gen_range(0..WORDS_EASY.len())]
        } else if difficulty.complexity < 0.7 {
            WORDS_MID[rng.gen_range(0..WORDS_MID.len())]
        } else {
            WORDS_HARD[rng.gen_range(0..WORDS_HARD.len())]
        };

        let scrambled = scramble_word(word, rng);
        let letter_count = scrambled.len();

        // Layout
        let letter_w = 40.0_f32;
        let letter_gap = if difficulty.complexity > 0.7 {
            // Partially overlapping at high complexity
            -5.0
        } else {
            8.0
        };
        let total_letters_w =
            letter_count as f32 * letter_w + (letter_count as f32 - 1.0) * letter_gap;
        let width = total_letters_w.max(300.0) + 40.0;
        let height = 140.0_f32;

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
            r##"<text x="{:.0}" y="24" font-family="sans-serif" font-size="14" fill="#cccccc" text-anchor="middle">Unscramble this word:</text>"##,
            width / 2.0
        )
        .unwrap();

        // Draw each letter
        let start_x = (width - total_letters_w) / 2.0;
        let base_y = 80.0;
        let font_size = 28.0;

        for (i, ch) in scrambled.chars().enumerate() {
            let x = start_x + i as f32 * (letter_w + letter_gap);
            let cx = x + letter_w / 2.0;

            // Per-letter distortion based on difficulty
            let rotation = if difficulty.complexity > 0.3 {
                rng.gen_range(-15.0..15.0_f32) * difficulty.complexity
            } else {
                0.0
            };
            let dy = if difficulty.complexity > 0.5 {
                rng.gen_range(-6.0..6.0_f32) * difficulty.complexity
            } else {
                0.0
            };

            // Letter background tile
            write!(
                svg,
                r##"<rect x="{x:.0}" y="{:.0}" width="{letter_w:.0}" height="{letter_w:.0}" rx="4" fill="#2a2a4a" stroke="#555" stroke-width="1"/>"##,
                base_y - letter_w / 2.0 - 5.0
            )
            .unwrap();

            // Letter
            let hue = rng.gen_range(30..60);
            write!(
                svg,
                r##"<text x="{cx:.0}" y="{:.0}" font-family="monospace" font-size="{font_size:.0}" font-weight="bold" fill="hsl({hue},70%,65%)" text-anchor="middle" dominant-baseline="central" transform="rotate({rotation:.1},{cx:.0},{:.0})">{ch}</text>"##,
                base_y + dy,
                base_y + dy
            )
            .unwrap();
        }

        // Noise overlay at high difficulty
        let noise_count = (difficulty.noise * 15.0) as u32;
        for _ in 0..noise_count {
            let x1 = rng.gen_range(0.0..width);
            let y1 = rng.gen_range(30.0..height);
            let x2 = rng.gen_range(0.0..width);
            let y2 = rng.gen_range(30.0..height);
            let r = rng.gen_range(80..160);
            let g = rng.gen_range(80..160);
            let b = rng.gen_range(80..160);
            write!(
                svg,
                r##"<line x1="{x1:.0}" y1="{y1:.0}" x2="{x2:.0}" y2="{y2:.0}" stroke="rgb({r},{g},{b})" stroke-width="1" opacity="0.2"/>"##
            )
            .unwrap();
        }

        // Noise dots
        let dot_count = (difficulty.noise * 20.0) as u32;
        for _ in 0..dot_count {
            let cx = rng.gen_range(start_x..(start_x + total_letters_w));
            let cy = rng.gen_range((base_y - 25.0)..(base_y + 25.0));
            let r = rng.gen_range(1.0..3.0_f32);
            let gray = rng.gen_range(100..200);
            write!(
                svg,
                r##"<circle cx="{cx:.0}" cy="{cy:.0}" r="{r:.1}" fill="rgb({gray},{gray},{gray})" opacity="0.3"/>"##
            )
            .unwrap();
        }

        svg.push_str("</svg>");

        CaptchaInstance {
            render_data: RenderPayload::Svg(svg),
            solution: Solution::Text(word.to_string()),
            expected_solve_time_ms: expected_solve_time(difficulty.time_limit_ms),
            point_value: CaptchaType::WordUnscramble.base_points(),
            captcha_type: CaptchaType::WordUnscramble,
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
    fn test_unscramble_generates_valid_instance() {
        let mut rng = rng_from_seed(42);
        let difficulty = DifficultyParams {
            level: 1,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.0,
            noise: 0.0,
        };
        let gen = UnscrambleGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert_eq!(instance.captcha_type, CaptchaType::WordUnscramble);
        if let Solution::Text(ref word) = instance.solution {
            assert!(word.len() >= 4 && word.len() <= 7);
            assert!(word.chars().all(|c| c.is_ascii_uppercase()));
        } else {
            panic!("Expected Text solution");
        }
        if let RenderPayload::Svg(ref s) = instance.render_data {
            assert!(s.contains("<svg"));
            assert!(s.contains("Unscramble"));
        } else {
            panic!("Expected SVG render data");
        }
    }

    #[test]
    fn test_unscramble_seed_determinism() {
        let difficulty = DifficultyParams {
            level: 10,
            round_number: 3,
            time_limit_ms: 7000,
            complexity: 0.5,
            noise: 0.3,
        };
        let gen = UnscrambleGenerator;

        let mut rng1 = rng_from_seed(12345);
        let inst1 = gen.generate(&mut rng1, &difficulty);

        let mut rng2 = rng_from_seed(12345);
        let inst2 = gen.generate(&mut rng2, &difficulty);

        if let (Solution::Text(w1), Solution::Text(w2)) = (&inst1.solution, &inst2.solution) {
            assert_eq!(w1, w2);
        }
    }

    #[test]
    fn test_unscramble_correct_answer_validates() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = UnscrambleGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        if let Solution::Text(ref word) = instance.solution {
            // Exact case
            assert!(gen.validate(&instance, &PlayerAnswer::Text(word.clone())));
            // Lowercase should also work (case-insensitive)
            assert!(gen.validate(
                &instance,
                &PlayerAnswer::Text(word.to_lowercase())
            ));
        }
    }

    #[test]
    fn test_unscramble_wrong_answer_rejects() {
        let mut rng = rng_from_seed(99);
        let difficulty = DifficultyParams {
            level: 5,
            round_number: 1,
            time_limit_ms: 5000,
            complexity: 0.2,
            noise: 0.1,
        };
        let gen = UnscrambleGenerator;
        let instance = gen.generate(&mut rng, &difficulty);
        assert!(!gen.validate(
            &instance,
            &PlayerAnswer::Text("ZZZZZ".to_string())
        ));
        assert!(!gen.validate(&instance, &PlayerAnswer::Number(0.0)));
        assert!(!gen.validate(
            &instance,
            &PlayerAnswer::SelectedIndices(vec![0])
        ));
    }
}
