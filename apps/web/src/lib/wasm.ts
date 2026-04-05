// @ts-expect-error — WASM module resolved at runtime by vite-plugin-wasm
import initWasmModule, * as wasmBindings from 'captcha-engine';
import type { CaptchaInstance, CaptchaType, DifficultyParams, PlayerAnswer, ScoreResult } from '../types/captcha';

let initialized = false;

export async function initWasm(): Promise<void> {
  if (initialized) return;
  await initWasmModule();
  initialized = true;
}

// Map our TS enum to the WASM enum value
const CAPTCHA_TYPE_MAP: Record<CaptchaType, number> = {
  // Tier 1
  DistortedText: 0,
  SimpleMath: 1,
  ImageGrid: 2,
  SliderAlignment: 3,
  // Tier 2
  RotatedObject: 4,
  PartialOcclusion: 5,
  SemanticOddity: 6,
  ToneRhythm: 7,
  ColorPerception: 8,
  // Tier 3
  AdversarialImage: 9,
  SequenceCompletion: 10,
  MultiStepVerification: 11,
  SpatialReasoning: 12,
  ContextualReasoning: 13,
  PathTracing: 14,
  BooleanLogic: 15,
  // Tier 1 extended
  DotCount: 16,
  ClockReading: 17,
  FractionComparison: 18,
  GraphReading: 19,
  // Tier 2 extended
  MirrorMatch: 20,
  BalanceScale: 21,
  WordUnscramble: 22,
  GradientOrder: 23,
  OverlapCounting: 24,
  RotationPrediction: 25,
  // Tier 4
  MetamorphicCaptcha: 26,
  CombinedModality: 27,
  AdversarialTypography: 28,
  ProceduralNovelType: 29,
  TimePressureCascade: 30,
};

export function generateCaptcha(
  seed: bigint,
  captchaType: CaptchaType,
  difficulty: DifficultyParams,
): CaptchaInstance {
  const wasmType = CAPTCHA_TYPE_MAP[captchaType];
  const result = wasmBindings.generate_captcha(BigInt(seed), wasmType, JSON.stringify(difficulty));
  return JSON.parse(result);
}

export function validateAnswer(instance: CaptchaInstance, answer: PlayerAnswer): boolean {
  return wasmBindings.validate_answer(JSON.stringify(instance), JSON.stringify(answer));
}

export function scoreAnswer(
  instance: CaptchaInstance,
  answer: PlayerAnswer,
  solveTimeMs: number,
): ScoreResult {
  const result = wasmBindings.score_answer(JSON.stringify(instance), JSON.stringify(answer), solveTimeMs);
  return JSON.parse(result);
}

export function computeDifficulty(
  captchaType: CaptchaType,
  level: number,
  roundNumber: number,
): DifficultyParams {
  const wasmType = CAPTCHA_TYPE_MAP[captchaType];
  const result = wasmBindings.compute_difficulty_params(wasmType, level, roundNumber);
  return JSON.parse(result);
}
