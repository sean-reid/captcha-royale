import { describe, expect, it } from 'vitest';
import { corsOriginFor } from './cors';

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
});
