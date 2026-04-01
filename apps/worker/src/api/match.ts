import type { Env } from '../index';
import type { Session } from '../lib/session';

export async function handleMatchPrivate(
  _request: Request,
  env: Env,
  session: Session,
): Promise<Response> {
  const roomCode = generateRoomCode();
  const roomId = `private:${roomCode}`;

  const roomObjId = env.MATCH_ROOM.idFromName(roomId);
  const room = env.MATCH_ROOM.get(roomObjId);

  // Initialize the room
  await room.fetch(
    new Request('https://internal/init', {
      method: 'POST',
      body: JSON.stringify({
        players: [session.playerId],
        mode: 'battle_royale',
        isPrivate: true,
        roomCode,
      }),
    }),
  );

  return Response.json({ roomCode, roomId });
}

export async function handleMatchJoin(
  _request: Request,
  env: Env,
  url: URL,
  session: Session,
): Promise<Response> {
  const code = url.pathname.split('/').pop();
  if (!code) return new Response('Not Found', { status: 404 });

  const roomId = `private:${code}`;
  const roomObjId = env.MATCH_ROOM.idFromName(roomId);
  const room = env.MATCH_ROOM.get(roomObjId);

  // Check if room exists by trying to add the player
  const res = await room.fetch(
    new Request('https://internal/join', {
      method: 'POST',
      body: JSON.stringify({ playerId: session.playerId }),
    }),
  );

  if (!res.ok) {
    return new Response('Room not found or full', { status: 404 });
  }

  return Response.json({ roomId });
}

function generateRoomCode(): string {
  const chars = 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789';
  let code = '';
  const random = new Uint8Array(6);
  crypto.getRandomValues(random);
  for (const byte of random) {
    code += chars[byte % chars.length];
  }
  return code;
}
