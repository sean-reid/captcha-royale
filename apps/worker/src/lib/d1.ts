import type { Env } from '../index';
import { levelFromXp } from './progression';

export async function getPlayer(env: Env, playerId: string) {
  return env.DB.prepare(
    'SELECT id, display_name, avatar_url, elo, level, xp, matches_played, wins, created_at, updated_at FROM players WHERE id = ?',
  )
    .bind(playerId)
    .first();
}

export async function createPlayer(
  env: Env,
  data: { id: string; display_name: string; avatar_url: string | null },
) {
  await env.DB.prepare(
    'INSERT INTO players (id, display_name, avatar_url) VALUES (?, ?, ?)',
  )
    .bind(data.id, data.display_name, data.avatar_url)
    .run();
}

export async function upsertOAuthIdentity(
  env: Env,
  data: {
    provider: string;
    provider_id: string;
    player_id: string;
    email: string | null;
  },
) {
  await env.DB.prepare(
    'INSERT OR REPLACE INTO oauth_identities (provider, provider_id, player_id, email) VALUES (?, ?, ?, ?)',
  )
    .bind(data.provider, data.provider_id, data.player_id, data.email)
    .run();
}

export async function findPlayerByOAuth(
  env: Env,
  provider: string,
  providerId: string,
): Promise<string | null> {
  const row = await env.DB.prepare(
    'SELECT player_id FROM oauth_identities WHERE provider = ? AND provider_id = ?',
  )
    .bind(provider, providerId)
    .first<{ player_id: string }>();

  return row?.player_id ?? null;
}

export async function findPlayerByEmail(
  env: Env,
  email: string,
): Promise<string | null> {
  const row = await env.DB.prepare(
    'SELECT player_id FROM oauth_identities WHERE email = ? LIMIT 1',
  )
    .bind(email)
    .first<{ player_id: string }>();

  return row?.player_id ?? null;
}

export async function updatePlayer(
  env: Env,
  playerId: string,
  data: { display_name?: string },
) {
  if (data.display_name !== undefined) {
    await env.DB.prepare(
      "UPDATE players SET display_name = ?, updated_at = datetime('now') WHERE id = ?",
    )
      .bind(data.display_name, playerId)
      .run();
  }
}

export async function getMatchHistory(
  env: Env,
  playerId: string,
  page: number,
  limit: number,
) {
  const offset = (page - 1) * limit;
  const results = await env.DB.prepare(
    `SELECT m.id, m.mode, m.player_count, m.rounds_played, m.started_at, m.ended_at,
            mr.placement, mr.elo_before, mr.elo_after, mr.xp_earned, mr.rounds_survived, mr.avg_solve_ms
     FROM match_results mr
     JOIN matches m ON mr.match_id = m.id
     WHERE mr.player_id = ?
     ORDER BY m.ended_at DESC
     LIMIT ? OFFSET ?`,
  )
    .bind(playerId, limit, offset)
    .all();

  return results.results;
}

export async function getPlayerAchievements(env: Env, playerId: string) {
  const results = await env.DB.prepare(
    `SELECT a.id, a.name, a.description, a.icon, pa.unlocked_at
     FROM player_achievements pa
     JOIN achievements a ON pa.achievement_id = a.id
     WHERE pa.player_id = ?
     ORDER BY pa.unlocked_at DESC`,
  )
    .bind(playerId)
    .all();

  return results.results;
}

/**
 * Recalculate a player's level from their current XP and persist it if
 * the level changed.  Returns the (possibly updated) level.
 */
export async function updatePlayerLevel(
  env: Env,
  playerId: string,
): Promise<number> {
  const row = await env.DB.prepare('SELECT xp, level FROM players WHERE id = ?')
    .bind(playerId)
    .first<{ xp: number; level: number }>();

  if (!row) return 1;

  const newLevel = levelFromXp(row.xp);

  if (newLevel !== row.level) {
    await env.DB.prepare(
      "UPDATE players SET level = ?, updated_at = datetime('now') WHERE id = ?",
    )
      .bind(newLevel, playerId)
      .run();
  }

  return newLevel;
}

export async function getLeaderboard(env: Env, limit = 100) {
  const results = await env.DB.prepare(
    'SELECT id, display_name, avatar_url, elo, level, wins FROM players ORDER BY elo DESC LIMIT ?',
  )
    .bind(limit)
    .all();

  return results.results?.map((r, i) => ({ ...r, rank: i + 1 })) ?? [];
}

export async function getLeaderboardAround(env: Env, playerId: string) {
  const player = await env.DB.prepare('SELECT elo FROM players WHERE id = ?')
    .bind(playerId)
    .first<{ elo: number }>();

  if (!player) return [];

  const results = await env.DB.prepare(
    `SELECT id, display_name, avatar_url, elo, level, wins
     FROM players
     WHERE elo >= ? - 100 AND elo <= ? + 100
     ORDER BY elo DESC
     LIMIT 11`,
  )
    .bind(player.elo, player.elo)
    .all();

  return results.results ?? [];
}
