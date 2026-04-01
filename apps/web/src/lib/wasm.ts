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
  DistortedText: 0,
  SimpleMath: 1,
  ImageGrid: 2,
  SliderAlignment: 3,
  RotatedObject: 4,
  PartialOcclusion: 5,
  SemanticOddity: 6,
  ToneRhythm: 7,
  ColorPerception: 8,
  AdversarialImage: 9,
  SequenceCompletion: 10,
  MultiStepVerification: 11,
  SpatialReasoning: 12,
  ContextualReasoning: 13,
  MetamorphicCaptcha: 14,
  CombinedModality: 15,
  AdversarialTypography: 16,
  ProceduralNovelType: 17,
  TimePressureCascade: 18,
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
