import type { Env } from '../index';
import type { Session } from '../lib/session';
import { getPlayer, updatePlayer, getMatchHistory, getPlayerAchievements } from '../lib/d1';

export async function handleProfile(
  request: Request,
  env: Env,
  url: URL,
  session: Session | null,
): Promise<Response> {
  // PATCH /api/profile — update own profile
  if (request.method === 'PATCH' && url.pathname === '/api/profile') {
    if (!session) return new Response('Unauthorized', { status: 401 });
    const body = (await request.json()) as { display_name?: string };

    // Sanitize display name
    if (body.display_name) {
      body.display_name = body.display_name.replace(/[<>&"']/g, '').trim().slice(0, 30);
    }

    await updatePlayer(env, session.playerId, body);
    const player = await getPlayer(env, session.playerId);
    return Response.json(player);
  }

  // GET /api/profile/:id
  if (request.method === 'GET') {
    const id = url.pathname.split('/').pop();
    if (!id) return new Response('Not Found', { status: 404 });
    const player = await getPlayer(env, id);
    if (!player) return new Response('Not Found', { status: 404 });
    return Response.json(player);
  }

  return new Response('Method Not Allowed', { status: 405 });
}

export async function handleProfileHistory(
  request: Request,
  env: Env,
  url: URL,
): Promise<Response> {
  const segments = url.pathname.split('/');
  const playerIdIdx = segments.indexOf('profile') + 1;
  const playerId = segments[playerIdIdx];
  if (!playerId) return new Response('Not Found', { status: 404 });

  const page = parseInt(url.searchParams.get('page') || '1', 10);
  const limit = Math.min(parseInt(url.searchParams.get('limit') || '10', 10), 50);

  const history = await getMatchHistory(env, playerId, page, limit);
  return Response.json({ results: history, page, limit });
}

export async function handleProfileAchievements(
  request: Request,
  env: Env,
  url: URL,
): Promise<Response> {
  const segments = url.pathname.split('/');
  const playerIdIdx = segments.indexOf('profile') + 1;
  const playerId = segments[playerIdIdx];
  if (!playerId) return new Response('Not Found', { status: 404 });

  const achievements = await getPlayerAchievements(env, playerId);
  return Response.json({ achievements });
}
