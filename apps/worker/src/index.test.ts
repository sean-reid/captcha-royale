import { describe, expect, it } from 'vitest';
import worker, { type Env } from './index';

const env = { FRONTEND_URL: 'https://captcha-royale.dwainosaur.com' } as Env;

describe('worker CORS headers', () => {
  it('answers a preflight with the matching origin and Vary: Origin', async () => {
    const res = await worker.fetch(
      new Request('https://w.example/api/leaderboard', {
        method: 'OPTIONS',
        headers: { Origin: 'http://localhost:4173' },
      }),
      env,
    );
    expect(res.headers.get('Access-Control-Allow-Origin')).toBe('http://localhost:4173');
    expect(res.headers.get('Vary')).toBe('Origin');
  });

  it('answers a preflight from the old github.io origin with the configured one', async () => {
    const res = await worker.fetch(
      new Request('https://w.example/api/leaderboard', {
        method: 'OPTIONS',
        headers: { Origin: 'https://sean-reid.github.io' },
      }),
      env,
    );
    expect(res.headers.get('Access-Control-Allow-Origin')).toBe('https://captcha-royale.dwainosaur.com');
    expect(res.headers.get('Vary')).toBe('Origin');
  });

  it('answers a preflight for a malformed FRONTEND_URL instead of throwing', async () => {
    const res = await worker.fetch(
      new Request('https://w.example/api/leaderboard', { method: 'OPTIONS' }),
      { FRONTEND_URL: 'captcha-royale.dwainosaur.com' } as Env,
    );
    expect(res.status).toBe(200);
    expect(res.headers.get('Access-Control-Allow-Origin')).toBe('http://localhost:5173');
  });

  it('adds the allow-origin and Vary headers to a handler response', async () => {
    const kv = { get: async () => [{ id: 'p1' }] } as unknown as KVNamespace;
    const res = await worker.fetch(
      new Request('https://w.example/api/leaderboard', {
        headers: { Origin: 'https://captcha-royale.dwainosaur.com' },
      }),
      { ...env, KV: kv } as Env,
    );
    expect(res.status).toBe(200);
    expect(res.headers.get('Access-Control-Allow-Origin')).toBe('https://captcha-royale.dwainosaur.com');
    expect(res.headers.get('Vary')).toBe('Origin');
  });

  it('adds Vary: Origin to a 500 from a throwing handler', async () => {
    const kv = {
      get: async () => {
        throw new Error('kv down');
      },
    } as unknown as KVNamespace;
    const res = await worker.fetch(
      new Request('https://w.example/api/leaderboard', {
        headers: { Origin: 'https://captcha-royale.dwainosaur.com' },
      }),
      { ...env, KV: kv } as Env,
    );
    expect(res.status).toBe(500);
    expect(res.headers.get('Access-Control-Allow-Origin')).toBe('https://captcha-royale.dwainosaur.com');
    expect(res.headers.get('Vary')).toBe('Origin');
  });
});
