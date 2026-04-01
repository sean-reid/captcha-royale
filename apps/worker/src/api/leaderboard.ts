import type { Env } from '../index';
import { getLeaderboard, getLeaderboardAround } from '../lib/d1';

export async function handleLeaderboard(
  _request: Request,
  env: Env,
  url: URL,
): Promise<Response> {
  const _season = url.searchParams.get('season') || 'current';

  // Try KV cache first
  const cached = await env.KV.get('leaderboard:top100', 'json');
  if (cached) return Response.json(cached);

  const leaderboard = await getLeaderboard(env);

  // Cache for 60 seconds
  await env.KV.put('leaderboard:top100', JSON.stringify(leaderboard), {
    expirationTtl: 60,
  });

  return Response.json(leaderboard);
}

export async function handleLeaderboardAround(
  _request: Request,
  env: Env,
  url: URL,
): Promise<Response> {
  const playerId = url.pathname.split('/').pop();
  if (!playerId) return new Response('Not Found', { status: 404 });

  const leaderboard = await getLeaderboardAround(env, playerId);
  return Response.json(leaderboard);
}
