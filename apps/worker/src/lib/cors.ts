export const LEGACY_FRONTEND_ORIGIN = 'https://sean-reid.github.io';

const DEV_ORIGINS = ['http://localhost:5173', 'http://localhost:4173'];

export function allowedOrigins(frontendUrl: string | undefined): string[] {
  const frontend = frontendUrl ? new URL(frontendUrl).origin : '';
  return [frontend, LEGACY_FRONTEND_ORIGIN, ...DEV_ORIGINS].filter(Boolean);
}

export function corsOriginFor(origin: string, frontendUrl: string | undefined): string {
  const allowed = allowedOrigins(frontendUrl);
  return allowed.includes(origin) ? origin : allowed[0];
}
