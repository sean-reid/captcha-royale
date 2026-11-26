import { describe, expect, it } from 'vitest';
import worker, { type Env } from './index';

const env = { FRONTEND_URL: 'https://captcha-royale.dwainosaur.com' } as Env;

describe('worker CORS headers', () => {
  it('answers a preflight with the matching origin and Vary: Origin', async () => {
    const res = await worker.fetch(
      new Request('https://w.example/api/leaderboard', {
        method: 'OPTIONS',
        headers: { Origin: 'https://sean-reid.github.io' },
      }),
      env,
    );
    expect(res.headers.get('Access-Control-Allow-Origin')).toBe('https://sean-reid.github.io');
    expect(res.headers.get('Vary')).toBe('Origin');
  });

  it('answers a preflight for a malformed FRONTEND_URL instead of throwing', async () => {
    const res = await worker.fetch(
      new Request('https://w.example/api/leaderboard', { method: 'OPTIONS' }),
      { FRONTEND_URL: 'captcha-royale.dwainosaur.com' } as Env,
    );
    expect(res.status).toBe(200);
    expect(res.headers.get('Access-Control-Allow-Origin')).toBe('https://sean-reid.github.io');
  });
});
