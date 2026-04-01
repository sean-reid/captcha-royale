/**
 * XP / Level progression system and CAPTCHA tier selection.
 */

// ---------- CaptchaType enum (mirrors the client-side types) ----------

export type CaptchaType =
  | 'DistortedText'
  | 'SimpleMath'
  | 'ImageGrid'
  | 'SliderPuzzle'
  | 'AudioChallenge'
  | 'RotateObject'
  | 'SemanticImage'
  | 'LogicPuzzle';

/**
 * Each captcha type belongs to a tier (1-4).
 * Difficulty multiplier scales XP rewards.
 */
interface CaptchaTierInfo {
  type: CaptchaType;
  tier: number;
  difficultyMultiplier: number;
}

const CAPTCHA_TIERS: CaptchaTierInfo[] = [
  // Tier 1
  { type: 'DistortedText', tier: 1, difficultyMultiplier: 1.0 },
  { type: 'SimpleMath', tier: 1, difficultyMultiplier: 1.0 },
  // Tier 2
  { type: 'ImageGrid', tier: 2, difficultyMultiplier: 1.5 },
  { type: 'SliderPuzzle', tier: 2, difficultyMultiplier: 1.5 },
  // Tier 3
  { type: 'AudioChallenge', tier: 3, difficultyMultiplier: 2.0 },
  { type: 'RotateObject', tier: 3, difficultyMultiplier: 2.0 },
  // Tier 4
  { type: 'SemanticImage', tier: 4, difficultyMultiplier: 3.0 },
  { type: 'LogicPuzzle', tier: 4, difficultyMultiplier: 3.0 },
];

// ---------- XP / Level helpers ----------

/**
 * Total XP required to reach a given level (cumulative from level 1).
 * Formula: 100 * level^1.5
 */
export function xpForLevel(level: number): number {
  return Math.floor(100 * Math.pow(level, 1.5));
}

/**
 * Derive the current level from a cumulative XP total.
 * Inverse of `xpForLevel`: level = (xp / 100)^(2/3), floored, minimum 1.
 */
export function levelFromXp(xp: number): number {
  if (xp <= 0) return 1;
  const level = Math.floor(Math.pow(xp / 100, 2 / 3));
  return Math.max(level, 1);
}

// ---------- XP reward computation ----------

export interface XpRewardParams {
  /** Number of CAPTCHAs the player solved this match. */
  captchasSolved: number;
  /** The difficulty multiplier of each solved CAPTCHA (parallel array). */
  difficultyMultipliers: number[];
  /** Whether the player had the fastest solve in at least one round. */
  fastestSolveRounds: number;
  /** Final placement (1 = winner). */
  placement: number;
  /** Total number of players in the match. */
  playersInMatch: number;
}

/**
 * Compute the total XP earned from a single match.
 *
 * Breakdown:
 *  - Each CAPTCHA solved:   10 * difficulty_multiplier
 *  - Fastest solve in round: +25 bonus (per round)
 *  - Match win (placement 1): 100 * (players_in_match / 4)
 *  - Top 3 (placement 2-3):   50 * (players_in_match / 4)
 *  - Participation:            20
 */
export function computeXpReward(params: XpRewardParams): number {
  const { captchasSolved, difficultyMultipliers, fastestSolveRounds, placement, playersInMatch } =
    params;

  let xp = 0;

  // Per-solve XP
  for (let i = 0; i < captchasSolved; i++) {
    const multiplier = difficultyMultipliers[i] ?? 1;
    xp += Math.floor(10 * multiplier);
  }

  // Fastest-solve bonus
  xp += fastestSolveRounds * 25;

  // Win bonus
  if (placement === 1) {
    xp += Math.floor(100 * (playersInMatch / 4));
  } else if (placement <= 3) {
    xp += Math.floor(50 * (playersInMatch / 4));
  }

  // Participation
  xp += 20;

  return xp;
}

// ---------- Tier gating by level ----------

/**
 * Returns the maximum CAPTCHA tier a player at `level` may face.
 *
 *  Level  1-10  -> Tier 1 only
 *  Level 11-25  -> Tiers 1-2
 *  Level 26-50  -> Tiers 1-3
 *  Level 50+    -> Tiers 1-4 (all)
 */
export function getTierForLevel(level: number): number {
  if (level <= 10) return 1;
  if (level <= 25) return 2;
  if (level <= 50) return 3;
  return 4;
}

// ---------- CAPTCHA type selection ----------

/**
 * Simple seeded pseudo-random number generator (mulberry32).
 * Accepts a numeric seed and returns a function that produces [0, 1).
 */
export type Rng = () => number;

/**
 * Select a random CaptchaType weighted by the tiers available for the
 * given player level.  Higher tiers have lower weight so they appear less
 * often, keeping the challenge curve smooth.
 *
 * @param level  The player (or median lobby) level.
 * @param round  The current round number (unused for weighting but reserved
 *               for future round-scaling).
 * @param rng    A () => number function producing values in [0, 1).
 */
export function selectCaptchaType(level: number, _round: number, rng: Rng): CaptchaType {
  const maxTier = getTierForLevel(level);

  // Filter to available tiers
  const available = CAPTCHA_TIERS.filter((c) => c.tier <= maxTier);

  // Weight: lower tiers appear more often.
  // Weight for tier t = maxTier - t + 1  (e.g. tier 1 in a maxTier=3 lobby gets weight 3)
  const weighted: { type: CaptchaType; weight: number }[] = available.map((c) => ({
    type: c.type,
    weight: maxTier - c.tier + 1,
  }));

  const totalWeight = weighted.reduce((sum, w) => sum + w.weight, 0);
  let roll = rng() * totalWeight;

  for (const entry of weighted) {
    roll -= entry.weight;
    if (roll <= 0) return entry.type;
  }

  // Fallback (should not be reached)
  return available[0].type;
}

/**
 * Retrieve the difficulty multiplier for a given CaptchaType.
 */
export function getDifficultyMultiplier(captchaType: CaptchaType): number {
  return CAPTCHA_TIERS.find((c) => c.type === captchaType)?.difficultyMultiplier ?? 1;
}
