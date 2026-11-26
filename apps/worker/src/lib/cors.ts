const DEV_ORIGINS = ['http://localhost:5173', 'http://localhost:4173'];

function originOf(url: string | undefined): string {
  if (!url) return '';
  try {
    return new URL(url).origin;
  } catch {
    return '';
  }
}

export function allowedOrigins(frontendUrl: string | undefined): string[] {
  return [originOf(frontendUrl), ...DEV_ORIGINS].filter(Boolean);
}

export function corsOriginFor(origin: string, frontendUrl: string | undefined): string {
  const allowed = allowedOrigins(frontendUrl);
  return allowed.includes(origin) ? origin : allowed[0];
}
