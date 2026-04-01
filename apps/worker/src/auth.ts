import type { Env } from './index';
import { createSession, deleteSession, sessionCookie, clearSessionCookie, validateSession } from './lib/session';
import { createPlayer, findPlayerByOAuth, findPlayerByEmail, upsertOAuthIdentity, getPlayer } from './lib/d1';

interface ProviderConfig {
  authUrl: string;
  tokenUrl: string;
  profileUrl: string;
  scopes: string[];
}

const PROVIDERS: Record<string, ProviderConfig> = {
  google: {
    authUrl: 'https://accounts.google.com/o/oauth2/v2/auth',
    tokenUrl: 'https://oauth2.googleapis.com/token',
    profileUrl: 'https://www.googleapis.com/oauth2/v2/userinfo',
    scopes: ['openid', 'email', 'profile'],
  },
  discord: {
    authUrl: 'https://discord.com/api/oauth2/authorize',
    tokenUrl: 'https://discord.com/api/oauth2/token',
    profileUrl: 'https://discord.com/api/users/@me',
    scopes: ['identify', 'email'],
  },
  github: {
    authUrl: 'https://github.com/login/oauth/authorize',
    tokenUrl: 'https://github.com/login/oauth/access_token',
    profileUrl: 'https://api.github.com/user',
    scopes: ['read:user', 'user:email'],
  },
};

function getCredentials(provider: string, env: Env): { clientId: string; clientSecret: string } {
  switch (provider) {
    case 'google':
      return { clientId: env.GOOGLE_CLIENT_ID, clientSecret: env.GOOGLE_CLIENT_SECRET };
    case 'discord':
      return { clientId: env.DISCORD_CLIENT_ID, clientSecret: env.DISCORD_CLIENT_SECRET };
    case 'github':
      return { clientId: env.GITHUB_CLIENT_ID, clientSecret: env.GITHUB_CLIENT_SECRET };
    default:
      throw new Error(`Unknown provider: ${provider}`);
  }
}

export async function handleAuth(request: Request, env: Env, path: string): Promise<Response> {
  // GET /api/auth/me — return current player
  if (path === '/api/auth/me') {
    const session = await validateSession(request, env);
    if (!session) return new Response('Unauthorized', { status: 401 });
    const player = await getPlayer(env, session.playerId);
    if (!player) return new Response('Not Found', { status: 404 });
    return Response.json(player);
  }

  // POST /api/auth/logout
  if (path === '/api/auth/logout' && request.method === 'POST') {
    await deleteSession(env, request);
    return new Response(JSON.stringify({ ok: true }), {
      headers: {
        'Content-Type': 'application/json',
        'Set-Cookie': clearSessionCookie(),
      },
    });
  }

  // GET /api/auth/callback — handle OAuth callback
  if (path === '/api/auth/callback') {
    return handleCallback(request, env);
  }

  // GET /api/auth/:provider — initiate OAuth
  const providerMatch = path.match(/^\/api\/auth\/(google|discord|github)$/);
  if (providerMatch) {
    return initiateOAuth(providerMatch[1], env, request);
  }

  return new Response('Not Found', { status: 404 });
}

async function initiateOAuth(provider: string, env: Env, request: Request): Promise<Response> {
  const config = PROVIDERS[provider];
  const creds = getCredentials(provider, env);

  const state = crypto.randomUUID();
  // Store state in KV with 5-minute TTL
  await env.KV.put(`oauth_state:${state}`, provider, { expirationTtl: 300 });

  const baseUrl = env.BASE_URL || new URL(request.url).origin;
  const redirectUri = `${baseUrl}/api/auth/callback`;

  const params = new URLSearchParams({
    client_id: creds.clientId,
    redirect_uri: redirectUri,
    response_type: 'code',
    scope: config.scopes.join(' '),
    state,
  });

  return Response.redirect(`${config.authUrl}?${params.toString()}`, 302);
}

async function handleCallback(request: Request, env: Env): Promise<Response> {
  const url = new URL(request.url);
  const code = url.searchParams.get('code');
  const state = url.searchParams.get('state');

  if (!code || !state) {
    return new Response('Missing code or state', { status: 400 });
  }

  // Verify state
  const provider = await env.KV.get(`oauth_state:${state}`);
  if (!provider) {
    return new Response('Invalid or expired state', { status: 403 });
  }
  // Delete state (single-use)
  await env.KV.delete(`oauth_state:${state}`);

  const config = PROVIDERS[provider];
  const creds = getCredentials(provider, env);
  const baseUrl = env.BASE_URL || url.origin;
  const redirectUri = `${baseUrl}/api/auth/callback`;

  // Exchange code for token
  const tokenRes = await fetch(config.tokenUrl, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded',
      Accept: 'application/json',
    },
    body: new URLSearchParams({
      client_id: creds.clientId,
      client_secret: creds.clientSecret,
      code,
      redirect_uri: redirectUri,
      grant_type: 'authorization_code',
    }).toString(),
  });

  const tokenData = (await tokenRes.json()) as { access_token?: string; error?: string };
  if (!tokenData.access_token) {
    return new Response('Token exchange failed', { status: 400 });
  }

  // Fetch user profile
  const profileRes = await fetch(config.profileUrl, {
    headers: { Authorization: `Bearer ${tokenData.access_token}` },
  });
  const profile = (await profileRes.json()) as Record<string, unknown>;

  // Extract provider ID, name, email, avatar
  const { providerId, displayName, email, avatarUrl } = extractProfile(provider, profile);

  // Check if this OAuth identity already exists
  let playerId = await findPlayerByOAuth(env, provider, providerId);

  if (!playerId) {
    // Try to link by email
    if (email) {
      playerId = await findPlayerByEmail(env, email);
    }

    if (!playerId) {
      // Create new player
      playerId = crypto.randomUUID();
      await createPlayer(env, { id: playerId, display_name: displayName, avatar_url: avatarUrl });
    }

    // Link OAuth identity
    await upsertOAuthIdentity(env, {
      provider,
      provider_id: providerId,
      player_id: playerId,
      email,
    });
  }

  // Create session
  const sessionToken = await createSession(env, playerId);

  // Redirect to frontend app
  const frontendUrl = env.FRONTEND_URL || '/';
  return new Response(null, {
    status: 302,
    headers: {
      Location: frontendUrl,
      'Set-Cookie': sessionCookie(sessionToken),
    },
  });
}

function extractProfile(
  provider: string,
  profile: Record<string, unknown>,
): { providerId: string; displayName: string; email: string | null; avatarUrl: string | null } {
  switch (provider) {
    case 'google':
      return {
        providerId: profile.id as string,
        displayName: (profile.name as string) || 'Player',
        email: (profile.email as string) || null,
        avatarUrl: (profile.picture as string) || null,
      };
    case 'discord':
      return {
        providerId: profile.id as string,
        displayName: (profile.username as string) || 'Player',
        email: (profile.email as string) || null,
        avatarUrl: profile.avatar
          ? `https://cdn.discordapp.com/avatars/${profile.id}/${profile.avatar}.png`
          : null,
      };
    case 'github':
      return {
        providerId: String(profile.id),
        displayName: (profile.login as string) || 'Player',
        email: (profile.email as string) || null,
        avatarUrl: (profile.avatar_url as string) || null,
      };
    default:
      return { providerId: '', displayName: 'Player', email: null, avatarUrl: null };
  }
}
