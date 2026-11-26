import { describe, expect, it } from 'vitest';
import { allowedOrigins, corsOriginFor } from './cors';

const NEW = 'https://captcha-royale.dwainosaur.com';
const OLD = 'https://sean-reid.github.io';
const DEFAULT = 'http://localhost:5173';

describe('corsOriginFor', () => {
  it('accepts the origin named by FRONTEND_URL', () => {
    expect(corsOriginFor(NEW, NEW)).toBe(NEW);
  });

  it('ignores a path on FRONTEND_URL', () => {
    expect(corsOriginFor(NEW, `${NEW}/app`)).toBe(NEW);
  });

  it('rejects the old github.io origin', () => {
    expect(corsOriginFor(OLD, NEW)).toBe(NEW);
    expect(allowedOrigins(NEW)).not.toContain(OLD);
  });

  it('accepts the local dev origins', () => {
    expect(corsOriginFor('http://localhost:5173', NEW)).toBe('http://localhost:5173');
  });

  it('answers an unknown origin with the configured one', () => {
    expect(corsOriginFor('https://evil.example', NEW)).toBe(NEW);
  });

  it('falls back to the first dev origin when FRONTEND_URL is unset', () => {
    expect(corsOriginFor('https://evil.example', undefined)).toBe(DEFAULT);
    expect(corsOriginFor(OLD, undefined)).toBe(DEFAULT);
    expect(allowedOrigins(undefined)).toEqual([DEFAULT, 'http://localhost:4173']);
  });

  it.each([
    'https://sean-reid.github.io.evil.example',
    'https://evil-sean-reid.github.io',
    'https://someone-else.github.io',
    'http://sean-reid.github.io',
    'https://captcha-royale.dwainosaur.com.evil.example',
    'http://captcha-royale.dwainosaur.com',
  ])('rejects the lookalike origin %s', (lookalike) => {
    expect(corsOriginFor(lookalike, NEW)).toBe(NEW);
  });

  it('treats an empty Origin as unknown', () => {
    expect(corsOriginFor('', NEW)).toBe(NEW);
  });

  it('accepts localhost 4173', () => {
    expect(corsOriginFor('http://localhost:4173', NEW)).toBe('http://localhost:4173');
  });

  it('skips a FRONTEND_URL that does not parse', () => {
    expect(corsOriginFor(NEW, 'captcha-royale.dwainosaur.com')).toBe(DEFAULT);
    expect(corsOriginFor(OLD, 'captcha-royale.dwainosaur.com')).toBe(DEFAULT);
    expect(allowedOrigins('captcha-royale.dwainosaur.com')).not.toContain('captcha-royale.dwainosaur.com');
  });
});
