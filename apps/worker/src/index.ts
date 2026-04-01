import { handleAuth } from './auth';
import { handleProfile, handleProfileHistory, handleProfileAchievements } from './api/profile';
import { handleLeaderboard, handleLeaderboardAround } from './api/leaderboard';
import { handleMatchPrivate, handleMatchJoin } from './api/match';
import { validateSession } from './lib/session';

export { MatchRoom } from './match-room';
export { Matchmaker } from './matchmaker';

export interface Env {
  DB: D1Database;
  KV: KVNamespace;
  MATCH_ROOM: DurableObjectNamespace;
  MATCHMAKER: DurableObjectNamespace;
  GOOGLE_CLIENT_ID: string;
  GOOGLE_CLIENT_SECRET: string;
  DISCORD_CLIENT_ID: string;
  DISCORD_CLIENT_SECRET: string;
  GITHUB_CLIENT_ID: string;
  GITHUB_CLIENT_SECRET: string;
  BASE_URL: string;
  FRONTEND_URL: string;
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    const path = url.pathname;

    // CORS — allow the frontend origin with credentials
    const origin = request.headers.get('Origin') || '';
    // FRONTEND_URL may include a path (e.g. https://x.github.io/captcha-royale)
    // but the Origin header is just the scheme+host, so extract the origin from FRONTEND_URL
    const frontendOrigin = env.FRONTEND_URL
      ? new URL(env.FRONTEND_URL).origin
      : '';
    const allowedOrigins = [
      frontendOrigin,
      'http://localhost:5173',
      'http://localhost:4173',
    ].filter(Boolean);
    const corsOrigin = allowedOrigins.includes(origin) ? origin : allowedOrigins[0] || '*';

    const corsHeaders: Record<string, string> = {
      'Access-Control-Allow-Origin': corsOrigin,
      'Access-Control-Allow-Methods': 'GET, POST, PATCH, DELETE, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type',
      'Access-Control-Allow-Credentials': 'true',
    };

    if (request.method === 'OPTIONS') {
      return new Response(null, { headers: corsHeaders });
    }

    try {
      let response: Response;

      // Auth routes (no session required)
      if (path.startsWith('/api/auth/')) {
        response = await handleAuth(request, env, path);
      }
      // WebSocket upgrade for matchmaking
      else if (path === '/api/match/queue') {
        const session = await validateSession(request, env);
        if (!session) return new Response('Unauthorized', { status: 401 });

        const matchmakerId = env.MATCHMAKER.idFromName('global');
        const matchmaker = env.MATCHMAKER.get(matchmakerId);
        const newUrl = new URL(request.url);
        newUrl.searchParams.set('playerId', session.playerId);
        return matchmaker.fetch(new Request(newUrl.toString(), request));
      }
      // WebSocket upgrade for match room
      else if (path.startsWith('/api/match/room/')) {
        const session = await validateSession(request, env);
        if (!session) return new Response('Unauthorized', { status: 401 });

        const roomId = path.split('/').pop()!;
        const roomObjId = env.MATCH_ROOM.idFromName(roomId);
        const room = env.MATCH_ROOM.get(roomObjId);
        const newUrl = new URL(request.url);
        newUrl.searchParams.set('playerId', session.playerId);
        return room.fetch(new Request(newUrl.toString(), request));
      }
      // Protected API routes
      else {
        const session = await validateSession(request, env);
        if (!session && !path.startsWith('/api/profile/') && request.method !== 'GET') {
          return new Response('Unauthorized', { status: 401 });
        }

        if (path.startsWith('/api/profile') && path.includes('/history')) {
          response = await handleProfileHistory(request, env, url);
        } else if (path.startsWith('/api/profile') && path.includes('/achievements')) {
          response = await handleProfileAchievements(request, env, url);
        } else if (path.startsWith('/api/profile')) {
          response = await handleProfile(request, env, url, session);
        } else if (path === '/api/leaderboard') {
          response = await handleLeaderboard(request, env, url);
        } else if (path.startsWith('/api/leaderboard/around/')) {
          response = await handleLeaderboardAround(request, env, url);
        } else if (path === '/api/match/private') {
          response = await handleMatchPrivate(request, env, session!);
        } else if (path.startsWith('/api/match/join/')) {
          response = await handleMatchJoin(request, env, url, session!);
        } else {
          response = new Response('Not Found', { status: 404 });
        }
      }

      // Add CORS headers to response
      const newHeaders = new Headers(response.headers);
      Object.entries(corsHeaders).forEach(([k, v]) => newHeaders.set(k, v));
      return new Response(response.body, {
        status: response.status,
        statusText: response.statusText,
        headers: newHeaders,
      });
    } catch (err) {
      console.error('Worker error:', err);
      return new Response(JSON.stringify({ error: 'Internal Server Error' }), {
        status: 500,
        headers: { 'Content-Type': 'application/json', ...corsHeaders },
      });
    }
  },
};
