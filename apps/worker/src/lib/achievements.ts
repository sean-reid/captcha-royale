/**
 * Achievement checking logic.
 *
 * After each match we inspect the player's cumulative stats (from D1)
 * and the just-finished match result to see if any new achievements
 * should be unlocked.
 */

import type { Env } from '../index';

// ---------- Match result shape passed in by the caller ----------

export interface MatchResultForAchievements {
  matchId: string;
  placement: number;
  playersInMatch: number;
  roundsSurvived: number;
  mode: 'battle_royale' | 'sprint' | 'endless';
  /** The player's new ELO *after* this match. */
  eloAfter: number;
  /** Whether the player solved a tier-3 CAPTCHA this match. */
  solvedTier3: boolean;
  /** Whether the player solved a tier-4 CAPTCHA this match. */
  solvedTier4: boolean;
}

// ---------- D1 row shapes ----------

interface PlayerStats {
  wins: number;
  matches_played: number;
  elo: number;
}

interface SolveCountRow {
  total_rounds: number;
}

interface FastSolveCountRow {
  fast_solves: number;
}

interface RecentWinsRow {
  recent_wins: number;
}

// ---------- Public API ----------

/**
 * Check whether the player just unlocked any achievements.
 *
 * Returns an array of newly-unlocked achievement IDs (e.g. `['first_win', 'elo_1500']`).
 * The caller is responsible for broadcasting these to the client.
 */
export async function checkAchievements(
  env: Env,
  playerId: string,
  matchResult: MatchResultForAchievements,
): Promise<string[]> {
  // Load already-unlocked achievements so we skip duplicates.
  const existingRows = await env.DB.prepare(
    'SELECT achievement_id FROM player_achievements WHERE player_id = ?',
  )
    .bind(playerId)
    .all<{ achievement_id: string }>();

  const alreadyUnlocked = new Set(existingRows.results.map((r) => r.achievement_id));

  const newlyUnlocked: string[] = [];

  // Helper: only unlock if not already present
  const tryUnlock = (id: string) => {
    if (!alreadyUnlocked.has(id)) {
      newlyUnlocked.push(id);
    }
  };

  // ---- first_win: Win your first match ----
  if (matchResult.placement === 1) {
    tryUnlock('first_win');
  }

  // ---- win_16_player: Win a 16-player lobby ----
  if (matchResult.placement === 1 && matchResult.playersInMatch >= 16) {
    tryUnlock('win_16_player');
  }

  // ---- endless_50: Survive 50 rounds in Endless mode ----
  if (matchResult.mode === 'endless' && matchResult.roundsSurvived >= 50) {
    tryUnlock('endless_50');
  }

  // ---- tier3_solve / tier4_solve ----
  if (matchResult.solvedTier3) {
    tryUnlock('tier3_solve');
  }
  if (matchResult.solvedTier4) {
    tryUnlock('tier4_solve');
  }

  // ---- ELO-based achievements ----
  if (matchResult.eloAfter >= 1500) {
    tryUnlock('elo_1500');
  }
  if (matchResult.eloAfter >= 2000) {
    tryUnlock('elo_2000');
  }

  // ---- solve_100: Solve 100 CAPTCHAs (total rounds survived across all matches) ----
  if (!alreadyUnlocked.has('solve_100')) {
    const solveRow = await env.DB.prepare(
      'SELECT COALESCE(SUM(rounds_survived), 0) AS total_rounds FROM match_results WHERE player_id = ?',
    )
      .bind(playerId)
      .first<SolveCountRow>();

    if (solveRow && solveRow.total_rounds >= 100) {
      tryUnlock('solve_100');
    }
  }

  // ---- solve_100_under_2s: Solve 100 CAPTCHAs under 2 seconds ----
  if (!alreadyUnlocked.has('solve_100_under_2s')) {
    // We approximate fast solves as rounds_survived in matches where avg_solve_ms < 2000.
    // A more precise approach would need per-round solve times, but we use available data.
    const fastRow = await env.DB.prepare(
      'SELECT COALESCE(SUM(rounds_survived), 0) AS fast_solves FROM match_results WHERE player_id = ? AND avg_solve_ms IS NOT NULL AND avg_solve_ms < 2000',
    )
      .bind(playerId)
      .first<FastSolveCountRow>();

    if (fastRow && fastRow.fast_solves >= 100) {
      tryUnlock('solve_100_under_2s');
    }
  }

  // ---- win_streak_5: Win 5 matches in a row ----
  if (!alreadyUnlocked.has('win_streak_5')) {
    // Check the last 5 matches for this player; all must be wins.
    const recentRow = await env.DB.prepare(
      `SELECT COUNT(*) AS recent_wins
       FROM (
         SELECT mr.placement
         FROM match_results mr
         JOIN matches m ON mr.match_id = m.id
         WHERE mr.player_id = ?
         ORDER BY m.ended_at DESC
         LIMIT 5
       )
       WHERE placement = 1`,
    )
      .bind(playerId)
      .first<RecentWinsRow>();

    if (recentRow && recentRow.recent_wins >= 5) {
      tryUnlock('win_streak_5');
    }
  }

  // ---- Persist newly unlocked achievements ----
  for (const achievementId of newlyUnlocked) {
    await env.DB.prepare(
      "INSERT OR IGNORE INTO player_achievements (player_id, achievement_id, unlocked_at) VALUES (?, ?, datetime('now'))",
    )
      .bind(playerId, achievementId)
      .run();
  }

  return newlyUnlocked;
}
