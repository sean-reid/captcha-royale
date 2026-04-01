import type { Env } from '../index';

export interface Session {
  playerId: string;
  expiresAt: number;
}

export async function validateSession(
  request: Request,
  env: Env,
): Promise<Session | null> {
  const cookie = request.headers.get('Cookie');
  if (!cookie) return null;

  const match = cookie.match(/session=([^;]+)/);
  if (!match) return null;

  const token = match[1];
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
  const cookie = request.headers.get('Cookie');
  if (!cookie) return;

  const match = cookie.match(/session=([^;]+)/);
  if (!match) return;

  await env.KV.delete(`session:${match[1]}`);
}

export function sessionCookie(token: string, maxAge = 7 * 24 * 60 * 60): string {
  return `session=${token}; HttpOnly; Secure; SameSite=None; Path=/; Max-Age=${maxAge}`;
}

export function clearSessionCookie(): string {
  return 'session=; HttpOnly; Secure; SameSite=None; Path=/; Max-Age=0';
}
