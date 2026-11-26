import { describe, expect, it } from 'vitest';
import { allowedOrigins, corsOriginFor } from './cors';

const NEW = 'https://captcha-royale.dwainosaur.com';
const OLD = 'https://sean-reid.github.io';

describe('corsOriginFor', () => {
  it('accepts the origin named by FRONTEND_URL', () => {
    expect(corsOriginFor(NEW, NEW)).toBe(NEW);
  });

  it('ignores a path on FRONTEND_URL', () => {
    expect(corsOriginFor(OLD, `${OLD}/captcha-royale`)).toBe(OLD);
  });

  it('keeps accepting the old origin after FRONTEND_URL moves', () => {
    expect(corsOriginFor(OLD, NEW)).toBe(OLD);
  });

  it('accepts the local dev origins', () => {
    expect(corsOriginFor('http://localhost:5173', NEW)).toBe('http://localhost:5173');
  });

  it('answers an unknown origin with the configured one', () => {
    expect(corsOriginFor('https://evil.example', NEW)).toBe(NEW);
  });

  it('falls back to the old origin when FRONTEND_URL is unset', () => {
    expect(corsOriginFor('https://evil.example', undefined)).toBe(OLD);
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
    expect(corsOriginFor('https://captcha-royale.dwainosaur.com', 'captcha-royale.dwainosaur.com')).toBe(OLD);
    expect(corsOriginFor(OLD, 'captcha-royale.dwainosaur.com')).toBe(OLD);
    expect(allowedOrigins('captcha-royale.dwainosaur.com')).not.toContain('captcha-royale.dwainosaur.com');
  });
});
