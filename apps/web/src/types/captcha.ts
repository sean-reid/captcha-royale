export enum CaptchaType {
  // Tier 1 — Foundations
  DistortedText = 'DistortedText',
  SimpleMath = 'SimpleMath',
  ImageGrid = 'ImageGrid',
  SliderAlignment = 'SliderAlignment',
  DotCount = 'DotCount',
  ClockReading = 'ClockReading',
  FractionComparison = 'FractionComparison',
  GraphReading = 'GraphReading',
  // Tier 2 — Perceptual
  RotatedObject = 'RotatedObject',
  PartialOcclusion = 'PartialOcclusion',
  SemanticOddity = 'SemanticOddity',
  ToneRhythm = 'ToneRhythm',
  ColorPerception = 'ColorPerception',
  MirrorMatch = 'MirrorMatch',
  BalanceScale = 'BalanceScale',
  WordUnscramble = 'WordUnscramble',
  GradientOrder = 'GradientOrder',
  OverlapCounting = 'OverlapCounting',
  RotationPrediction = 'RotationPrediction',
  // Tier 3 — Cognitive
  AdversarialImage = 'AdversarialImage',
  SequenceCompletion = 'SequenceCompletion',
  MultiStepVerification = 'MultiStepVerification',
  SpatialReasoning = 'SpatialReasoning',
  ContextualReasoning = 'ContextualReasoning',
  PathTracing = 'PathTracing',
  BooleanLogic = 'BooleanLogic',
  // Tier 4 — Nightmare
  MetamorphicCaptcha = 'MetamorphicCaptcha',
  CombinedModality = 'CombinedModality',
  AdversarialTypography = 'AdversarialTypography',
  ProceduralNovelType = 'ProceduralNovelType',
  TimePressureCascade = 'TimePressureCascade',
}

export interface DifficultyParams {
  level: number;
  round_number: number;
  time_limit_ms: number;
  complexity: number;
  noise: number;
}

export interface GridCell {
  index: number;
  svg: string;
  label: string;
}

export type RenderPayload =
  | { Svg: string }
  | { Grid: { cols: number; rows: number; cells: GridCell[]; prompt: string } }
  | { Slider: { background_svg: string; piece_svg: string; correct_x: number; correct_y: number } };

export type Solution =
  | { Text: string }
  | { Number: number }
  | { SelectedIndices: number[] }
  | { Position: { x: number; y: number; tolerance: number } };

export type PlayerAnswer =
  | { Text: string }
  | { Number: number }
  | { SelectedIndices: number[] }
  | { Position: { x: number; y: number } };

export interface CaptchaInstance {
  render_data: RenderPayload;
  solution: Solution;
  expected_solve_time_ms: number;
  point_value: number;
  captcha_type: CaptchaType;
  time_limit_ms: number;
}

export interface ScoreResult {
  correct: boolean;
  base_points: number;
  speed_bonus: number;
  total_points: number;
}
