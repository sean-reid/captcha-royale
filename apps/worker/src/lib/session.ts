import type { Env } from '../index';

export interface Session {
  playerId: string;
  expiresAt: number;
}

export async function validateSession(
  request: Request,
  env: Env,
): Promise<Session | null> {
  // For WebSocket upgrades, prefer query param token (client sets it explicitly)
  // over cookies which may be stale cross-origin cookies
  const isWebSocket = request.headers.get('Upgrade') === 'websocket';

  // Try query param first for WebSocket requests
  if (isWebSocket) {
    const url = new URL(request.url);
    const tokenParam = url.searchParams.get('token');
    if (tokenParam) {
      return validateToken(tokenParam, env);
    }
  }

  // Try Authorization: Bearer header (cross-origin API calls)
  const authHeader = request.headers.get('Authorization');
  if (authHeader?.startsWith('Bearer ')) {
    return validateToken(authHeader.slice(7), env);
  }

  // Fall back to cookie (same-origin / local dev)
  const cookie = request.headers.get('Cookie');
  if (cookie) {
    const match = cookie.match(/session=([^;]+)/);
    if (match) {
      return validateToken(match[1], env);
    }
  }

  // Fall back to query param for non-WebSocket requests
  if (!isWebSocket) {
    const url = new URL(request.url);
    const tokenParam = url.searchParams.get('token');
    if (tokenParam) {
      return validateToken(tokenParam, env);
    }
  }

  return null;
}

async function validateToken(token: string, env: Env): Promise<Session | null> {
  const data = await env.KV.get(`session:${token}`, 'json');
  if (!data) return null;

  const session = data as Session;
  if (Date.now() > session.expiresAt) {
    await env.KV.delete(`session:${token}`);
    return null;
  }

  return session;
}

export async function createSession(
  env: Env,
  playerId: string,
): Promise<string> {
  const token = crypto.randomUUID();
  const ttl = 7 * 24 * 60 * 60; // 7 days in seconds
  const session: Session = {
    playerId,
    expiresAt: Date.now() + ttl * 1000,
  };

  await env.KV.put(`session:${token}`, JSON.stringify(session), {
    expirationTtl: ttl,
  });

  return token;
}

export async function deleteSession(
  env: Env,
  request: Request,
): Promise<void> {
  const authHeader = request.headers.get('Authorization');
  if (authHeader?.startsWith('Bearer ')) {
    await env.KV.delete(`session:${authHeader.slice(7)}`);
    return;
  }

  const cookie = request.headers.get('Cookie');
  if (!cookie) return;
  const match = cookie.match(/session=([^;]+)/);
  if (match) {
    await env.KV.delete(`session:${match[1]}`);
  }
}
