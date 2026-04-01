import type { Env } from './index';

type GameMode = 'battle_royale' | 'sprint';

interface QueueEntry {
  playerId: string;
  displayName: string;
  elo: number;
  mode: GameMode;
  joinedAt: number;
}

const MODE_CONFIG: Record<GameMode, { minPlayers: number; targetPlayers: number; maxPlayers: number }> = {
  battle_royale: { minPlayers: 2, targetPlayers: 8, maxPlayers: 16 },
  sprint: { minPlayers: 2, targetPlayers: 4, maxPlayers: 8 },
};

const MIN_PLAYERS = 2;
const TARGET_PLAYERS = 8;
const MAX_PLAYERS = 16;
const BRACKET_EXPANSION_30S = 1;
const BRACKET_EXPANSION_60S = 2;

const BRACKETS = ['bronze', 'silver', 'gold', 'platinum', 'diamond'] as const;
type Bracket = (typeof BRACKETS)[number];

export class Matchmaker implements DurableObject {
  private state: DurableObjectState;
  private env: Env;
  private queues: Map<Bracket, QueueEntry[]> = new Map();
  private connections: Map<string, WebSocket> = new Map();
  private playerBrackets: Map<string, Bracket> = new Map();

  constructor(state: DurableObjectState, env: Env) {
    this.state = state;
    this.env = env;

    // Initialize empty queues
    for (const bracket of BRACKETS) {
      this.queues.set(bracket, []);
    }
  }

  async fetch(request: Request): Promise<Response> {
    if (request.headers.get('Upgrade') !== 'websocket') {
      return new Response('Expected WebSocket', { status: 400 });
    }

    const url = new URL(request.url);
    const playerId = url.searchParams.get('playerId');
    if (!playerId) {
      return new Response('Missing playerId', { status: 400 });
    }

    // Get player info from D1
    const player = await this.env.DB.prepare(
      'SELECT elo, display_name FROM players WHERE id = ?',
    )
      .bind(playerId)
      .first<{ elo: number; display_name: string }>();

    const elo = player?.elo ?? 1000;
    const displayName = player?.display_name ?? 'Player';
    const bracket = this.getBracket(elo);

    const pair = new WebSocketPair();
    const [client, server] = [pair[0], pair[1]];

    this.state.acceptWebSocket(server);
    this.connections.set(playerId, server);
    this.playerBrackets.set(playerId, bracket);

    // Read mode from query param
    const mode = (url.searchParams.get('mode') || 'battle_royale') as GameMode;

    // Remove any existing queue entry for this player (prevent double-queue)
    for (const [, q] of this.queues) {
      const idx = q.findIndex((e) => e.playerId === playerId);
      if (idx !== -1) q.splice(idx, 1);
    }

    // Close any existing connection for this player (second tab replaces first)
    const existingWs = this.connections.get(playerId);
    if (existingWs) {
      try { existingWs.send(JSON.stringify({ type: 'error', code: 'duplicate', message: 'Connected from another tab' })); } catch {}
      try { existingWs.close(); } catch {}
    }

    // Add to queue
    const queue = this.queues.get(bracket)!;
    queue.push({ playerId, displayName, elo, mode, joinedAt: Date.now() });

    // Broadcast updated queue status to everyone in this bracket
    this.broadcastQueueStatus(bracket);

    // Try to match immediately, then schedule recurring tick
    await this.runMatchmakingTick();
    this.scheduleAlarm();

    return new Response(null, { status: 101, webSocket: client });
  }

  async webSocketMessage(_ws: WebSocket, _msg: string | ArrayBuffer): Promise<void> {
    // Client doesn't send meaningful messages during matchmaking
  }

  async webSocketClose(ws: WebSocket): Promise<void> {
    // Remove player from queue
    const playerId = this.getPlayerIdForWs(ws);
    if (!playerId) return;

    this.connections.delete(playerId);
    const bracket = this.playerBrackets.get(playerId);
    this.playerBrackets.delete(playerId);

    if (bracket) {
      const queue = this.queues.get(bracket)!;
      const idx = queue.findIndex((e) => e.playerId === playerId);
      if (idx !== -1) queue.splice(idx, 1);
      // Notify remaining players
      this.broadcastQueueStatus(bracket);
    }
  }

  async webSocketError(ws: WebSocket): Promise<void> {
    await this.webSocketClose(ws);
  }

  async alarm(): Promise<void> {
    await this.runMatchmakingTick();

    // Reschedule if any players are queued
    if (this.totalQueued() > 0) {
      this.state.storage.setAlarm(Date.now() + 1000);
    }
  }

  private async runMatchmakingTick(): Promise<void> {
    this.expandBrackets();

    for (const [bracket, queue] of this.queues) {
      // Group by mode within each bracket
      for (const mode of ['battle_royale', 'sprint'] as GameMode[]) {
        const modePlayers = queue.filter((e) => e.mode === mode);
        const config = MODE_CONFIG[mode];

        if (modePlayers.length < config.minPlayers) continue;

        const shouldCreate =
          modePlayers.length >= config.targetPlayers ||
          (modePlayers.length >= config.minPlayers && this.oldestWait(modePlayers) > 15_000);

        if (shouldCreate) {
          const count = Math.min(modePlayers.length, config.maxPlayers);
          const matched = modePlayers.slice(0, count);
          // Remove matched players from the main queue
          for (const p of matched) {
            const idx = queue.findIndex((e) => e.playerId === p.playerId);
            if (idx !== -1) queue.splice(idx, 1);
          }
          await this.createMatch(matched, mode);
        }
      }
    }
  }

  private async createMatch(players: QueueEntry[], mode: GameMode): Promise<void> {
    const roomId = crypto.randomUUID();
    const roomObjId = this.env.MATCH_ROOM.idFromName(roomId);
    const room = this.env.MATCH_ROOM.get(roomObjId);

    // Initialize room
    await room.fetch(
      new Request('https://internal/init', {
        method: 'POST',
        body: JSON.stringify({
          players: players.map((p) => p.playerId),
          mode,
        }),
      }),
    );

    // Notify all matched players
    for (const player of players) {
      const ws = this.connections.get(player.playerId);
      if (ws) {
        try {
          ws.send(JSON.stringify({ type: 'match_found', roomId }));
        } catch {
          // Connection may be closed
        }
        this.connections.delete(player.playerId);
        this.playerBrackets.delete(player.playerId);
      }
    }
  }

  private expandBrackets(): void {
    const now = Date.now();

    for (let i = 0; i < BRACKETS.length; i++) {
      const queue = this.queues.get(BRACKETS[i])!;

      for (const entry of queue) {
        const waitTime = now - entry.joinedAt;

        if (waitTime > 60_000) {
          // Expand to ±2 brackets
          for (
            let j = Math.max(0, i - BRACKET_EXPANSION_60S);
            j <= Math.min(BRACKETS.length - 1, i + BRACKET_EXPANSION_60S);
            j++
          ) {
            if (j !== i) {
              const targetQueue = this.queues.get(BRACKETS[j])!;
              if (!targetQueue.find((e) => e.playerId === entry.playerId)) {
                targetQueue.push({ ...entry });
              }
            }
          }
        } else if (waitTime > 30_000) {
          // Expand to ±1 bracket
          for (
            let j = Math.max(0, i - BRACKET_EXPANSION_30S);
            j <= Math.min(BRACKETS.length - 1, i + BRACKET_EXPANSION_30S);
            j++
          ) {
            if (j !== i) {
              const targetQueue = this.queues.get(BRACKETS[j])!;
              if (!targetQueue.find((e) => e.playerId === entry.playerId)) {
                targetQueue.push({ ...entry });
              }
            }
          }
        }
      }
    }
  }

  private getBracket(elo: number): Bracket {
    if (elo < 800) return 'bronze';
    if (elo < 1000) return 'silver';
    if (elo < 1200) return 'gold';
    if (elo < 1500) return 'platinum';
    return 'diamond';
  }

  private oldestWait(queue: QueueEntry[]): number {
    if (queue.length === 0) return 0;
    return Date.now() - queue[0].joinedAt;
  }

  private totalQueued(): number {
    let total = 0;
    // Count unique players across all brackets
    const seen = new Set<string>();
    for (const queue of this.queues.values()) {
      for (const entry of queue) {
        seen.add(entry.playerId);
      }
    }
    return seen.size;
  }

  private getPlayerIdForWs(ws: WebSocket): string | null {
    for (const [id, conn] of this.connections) {
      if (conn === ws) return id;
    }
    return null;
  }

  private broadcastQueueStatus(bracket: Bracket): void {
    const queue = this.queues.get(bracket)!;
    const players = queue.map((e) => ({
      id: e.playerId,
      display_name: e.displayName,
      elo: e.elo,
    }));
    const msg = JSON.stringify({
      type: 'queue_status',
      bracket,
      playersInBracket: queue.length,
      players,
    });
    // Send to all connected players in this bracket
    for (const [playerId, ws] of this.connections) {
      if (this.playerBrackets.get(playerId) === bracket) {
        try { ws.send(msg); } catch { /* closing */ }
      }
    }
  }

  private scheduleAlarm(): void {
    this.state.storage.setAlarm(Date.now() + 1000);
  }
}
