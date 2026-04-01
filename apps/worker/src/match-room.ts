import type { Env } from './index';
import { calculateMultiplayerElo } from './lib/elo';

interface PlayerState {
  playerId: string;
  displayName: string;
  elo: number;
  level: number;
  alive: boolean;
  score: number;
  roundsSurvived: number;
  solveTimes: number[];
}

interface RoomState {
  matchId: string;
  secret: string;
  round: number;
  mode: 'battle_royale' | 'sprint' | 'endless';
  isPrivate: boolean;
  roomCode: string | null;
  started: boolean;
  ended: boolean;
  startedAt: string | null;
  roundStartTime: number;
  roundAnswers: Map<string, { correct: boolean; timeMs: number }>;
  maxRounds: number;
}

const MAX_PLAYERS = 16;
const MIN_PLAYERS = 2;
const COUNTDOWN_SECONDS = 3;
const RECONNECT_GRACE_MS = 10_000;

export class MatchRoom implements DurableObject {
  private state: DurableObjectState;
  private env: Env;
  private players: Map<string, PlayerState> = new Map();
  private connections: Map<string, WebSocket> = new Map();
  private disconnectTimers: Map<string, ReturnType<typeof setTimeout>> = new Map();
  private roomState: RoomState = {
    matchId: '',
    secret: '',
    round: 0,
    mode: 'battle_royale',
    isPrivate: false,
    roomCode: null,
    started: false,
    ended: false,
    startedAt: null,
    roundStartTime: 0,
    roundAnswers: new Map(),
    maxRounds: 20,
  };

  constructor(state: DurableObjectState, env: Env) {
    this.state = state;
    this.env = env;
  }

  async fetch(request: Request): Promise<Response> {
    const url = new URL(request.url);

    // Internal init endpoint
    if (url.pathname === '/init' && request.method === 'POST') {
      return this.handleInit(request);
    }

    // Internal join endpoint (for private rooms)
    if (url.pathname === '/join' && request.method === 'POST') {
      return this.handleJoin(request);
    }

    // WebSocket upgrade
    if (request.headers.get('Upgrade') === 'websocket') {
      return this.handleWebSocket(request, url);
    }

    return new Response('Not Found', { status: 404 });
  }

  private async handleInit(request: Request): Promise<Response> {
    const body = (await request.json()) as {
      players: string[];
      mode: string;
      isPrivate?: boolean;
      roomCode?: string;
    };

    this.roomState.matchId = crypto.randomUUID();
    this.roomState.secret = crypto.randomUUID();
    this.roomState.mode = (body.mode || 'battle_royale') as RoomState['mode'];
    this.roomState.isPrivate = body.isPrivate || false;
    this.roomState.roomCode = body.roomCode || null;
    this.roomState.maxRounds = body.mode === 'sprint' ? 10 : 20;

    return new Response('OK');
  }

  private async handleJoin(request: Request): Promise<Response> {
    const body = (await request.json()) as { playerId: string };
    if (this.players.size >= MAX_PLAYERS) {
      return new Response('Room full', { status: 400 });
    }
    if (this.roomState.started) {
      return new Response('Match already started', { status: 400 });
    }
    return new Response('OK');
  }

  private handleWebSocket(request: Request, url: URL): Response {
    const playerId = url.searchParams.get('playerId');
    if (!playerId) {
      return new Response('Missing playerId', { status: 400 });
    }

    if (this.players.size >= MAX_PLAYERS && !this.players.has(playerId)) {
      return new Response('Room full', { status: 400 });
    }

    const pair = new WebSocketPair();
    const [client, server] = [pair[0], pair[1]];

    this.state.acceptWebSocket(server);

    // Clear any disconnect timer for reconnection
    const timer = this.disconnectTimers.get(playerId);
    if (timer) {
      clearTimeout(timer);
      this.disconnectTimers.delete(playerId);
    }

    // Register connection
    this.connections.set(playerId, server);

    // Initialize player if new
    if (!this.players.has(playerId)) {
      this.players.set(playerId, {
        playerId,
        displayName: `Player ${this.players.size + 1}`,
        elo: 1000,
        level: 1,
        alive: true,
        score: 0,
        roundsSurvived: 0,
        solveTimes: [],
      });
    } else {
      // Reconnecting — mark alive again if within grace period
      const player = this.players.get(playerId)!;
      if (!player.alive) {
        // Can't resurrect after elimination
      }
    }

    this.broadcastLobbyUpdate();

    // Auto-start if enough players and not private room waiting
    if (!this.roomState.started && !this.roomState.isPrivate && this.players.size >= MIN_PLAYERS) {
      this.startCountdown();
    }

    return new Response(null, { status: 101, webSocket: client });
  }

  async webSocketMessage(ws: WebSocket, msg: string | ArrayBuffer): Promise<void> {
    if (typeof msg !== 'string') return;

    const playerId = this.getPlayerIdForWs(ws);
    if (!playerId) return;

    try {
      const data = JSON.parse(msg);
      switch (data.type) {
        case 'submit_answer':
          this.handleAnswer(playerId, data);
          break;
        case 'forfeit':
          this.eliminatePlayer(playerId, 'forfeit');
          break;
        case 'start':
          // For private rooms, host can start
          if (this.roomState.isPrivate && !this.roomState.started) {
            this.startCountdown();
          }
          break;
        case 'heartbeat':
          break;
      }
    } catch {
      // Ignore malformed messages
    }
  }

  async webSocketClose(ws: WebSocket): Promise<void> {
    const playerId = this.getPlayerIdForWs(ws);
    if (!playerId) return;

    this.connections.delete(playerId);

    // Grace period for reconnection
    const timer = setTimeout(() => {
      this.eliminatePlayer(playerId, 'disconnect');
      this.disconnectTimers.delete(playerId);
    }, RECONNECT_GRACE_MS);

    this.disconnectTimers.set(playerId, timer);

    // If no one is connected, end the match
    if (this.connections.size === 0 && this.roomState.started) {
      setTimeout(() => this.endMatch(), RECONNECT_GRACE_MS + 1000);
    }
  }

  async webSocketError(ws: WebSocket): Promise<void> {
    await this.webSocketClose(ws);
  }

  private getPlayerIdForWs(ws: WebSocket): string | null {
    for (const [id, conn] of this.connections) {
      if (conn === ws) return id;
    }
    return null;
  }

  private startCountdown(): void {
    this.roomState.started = true;
    this.roomState.startedAt = new Date().toISOString();

    // Broadcast countdown
    this.broadcast({
      type: 'lobby_update',
      players: this.getPlayerInfos(),
      countdown: COUNTDOWN_SECONDS,
    });

    // Start first round after countdown
    this.state.storage.setAlarm(Date.now() + COUNTDOWN_SECONDS * 1000);
  }

  async alarm(): Promise<void> {
    if (this.roomState.ended) return;

    // Check if it's a timeout alarm for current round
    if (this.roomState.round > 0) {
      this.handleRoundTimeout();
      return;
    }

    // Start next round
    this.startNextRound();
  }

  private startNextRound(): void {
    this.roomState.round++;
    this.roomState.roundAnswers = new Map();
    this.roomState.roundStartTime = Date.now();

    const isSprint = this.roomState.mode === 'sprint';

    if (!isSprint) {
      // Battle Royale: end when 1 or fewer alive
      const alivePlayers = this.getAlivePlayers();
      if (alivePlayers.length <= 1) {
        this.endMatch();
        return;
      }
    }

    if (this.roomState.round > this.roomState.maxRounds) {
      this.endMatch();
      return;
    }

    // Determine CAPTCHA type based on round
    const captchaTypes = ['DistortedText', 'SimpleMath', 'ImageGrid'];
    const captchaType = captchaTypes[this.roomState.round % captchaTypes.length];

    // Generate seed
    const seed = this.deriveSeed(this.roomState.round);

    // Compute difficulty based on median level
    const medianLevel = this.getMedianLevel();
    const complexity = Math.min(
      (medianLevel / 100) * 0.7 + (this.roomState.round / 20) * 0.3,
      1.0,
    );
    const timeLimit = 5000 + Math.floor(complexity * 5000);

    this.broadcast({
      type: 'round_start',
      round: this.roomState.round,
      seed: seed.toString(),
      captcha_type: captchaType,
      difficulty: {
        level: medianLevel,
        round_number: this.roomState.round,
        time_limit_ms: timeLimit,
        complexity,
        noise: complexity * 0.8,
      },
      time_limit_ms: timeLimit,
    });

    // Set timeout alarm
    this.state.storage.setAlarm(Date.now() + timeLimit + 1000);
  }

  private handleAnswer(playerId: string, data: {
    round: number;
    answer: unknown;
    client_time_ms: number;
  }): void {
    const player = this.players.get(playerId);
    if (!player || !player.alive) return;

    // Prevent duplicate answers for the same round
    if (this.roomState.roundAnswers.has(playerId)) return;

    // Validate round number
    if (data.round !== this.roomState.round) return;

    const serverTimeMs = Date.now() - this.roomState.roundStartTime;

    // Server-side validation would use the embedded WASM module.
    // For now, we trust the round logic and mark the answer.
    // In production: regenerate CAPTCHA from seed, validate answer.
    const correct = true; // TODO: Replace with actual WASM validation

    this.roomState.roundAnswers.set(playerId, { correct, timeMs: serverTimeMs });

    const isSprint = this.roomState.mode === 'sprint';

    if (correct) {
      const tier = 1; // TODO: derive from captcha type
      const basePoints = tier * 10;
      const timeLimit = 10000; // TODO: from round params
      const speedBonus = Math.max(0, Math.floor(((timeLimit - serverTimeMs) / timeLimit) * basePoints));
      player.score += basePoints + speedBonus;
      player.roundsSurvived++;
      player.solveTimes.push(serverTimeMs);

      this.broadcast({
        type: 'player_solved',
        player_id: playerId,
        time_ms: serverTimeMs,
      });
    } else if (isSprint) {
      // Sprint: wrong answer = no points, but player stays alive
      player.roundsSurvived++;
      this.broadcast({
        type: 'player_solved',
        player_id: playerId,
        time_ms: serverTimeMs,
      });
    } else {
      // Battle Royale: wrong answer = eliminated
      this.eliminatePlayer(playerId, 'wrong');
    }

    // Check if all players have answered (in Sprint, all players are always alive)
    const activeCount = isSprint ? this.players.size : this.getAlivePlayers().length;
    const answeredCount = this.roomState.roundAnswers.size;
    if (answeredCount >= activeCount) {
      this.endRound();
    }
  }

  private handleRoundTimeout(): void {
    const isSprint = this.roomState.mode === 'sprint';
    for (const [, player] of this.players) {
      if (player.alive && !this.roomState.roundAnswers.has(player.playerId)) {
        if (isSprint) {
          // Sprint: timeout = 0 points, but stay in the game
          this.roomState.roundAnswers.set(player.playerId, { correct: false, timeMs: 0 });
          player.roundsSurvived++;
        } else {
          this.eliminatePlayer(player.playerId, 'timeout');
        }
      }
    }
    this.endRound();
  }

  private endRound(): void {
    const standings = this.getStandings();
    this.broadcast({ type: 'round_end', standings });

    const isSprint = this.roomState.mode === 'sprint';
    const alivePlayers = this.getAlivePlayers();
    const shouldEnd = isSprint
      ? this.roomState.round >= this.roomState.maxRounds
      : alivePlayers.length <= 1 || this.roomState.round >= this.roomState.maxRounds;

    if (shouldEnd) {
      setTimeout(() => this.endMatch(), 2000);
    } else {
      // Reset round counter for alarm logic
      const currentRound = this.roomState.round;
      this.roomState.round = 0;
      // Start next round after brief pause
      setTimeout(() => {
        this.roomState.round = currentRound;
        this.startNextRound();
      }, 3000);
    }
  }

  private eliminatePlayer(playerId: string, reason: string): void {
    const player = this.players.get(playerId);
    if (!player || !player.alive) return;

    player.alive = false;

    this.broadcast({
      type: 'player_eliminated',
      player_id: playerId,
      reason: reason as 'wrong' | 'timeout',
    });

    // Check if match should end
    const alivePlayers = this.getAlivePlayers();
    if (alivePlayers.length <= 1 && this.roomState.started) {
      setTimeout(() => this.endMatch(), 1000);
    }
  }

  private async endMatch(): Promise<void> {
    if (this.roomState.ended) return;
    this.roomState.ended = true;

    const placements = this.computePlacements();
    const matchesPlayed = new Map<string, number>();

    // Compute ELO changes
    const eloInput = placements.map((p) => ({
      playerId: p.player_id,
      elo: this.players.get(p.player_id)!.elo,
      placement: p.placement,
    }));
    const eloResults = calculateMultiplayerElo(eloInput, matchesPlayed);

    const finalStandings = placements.map((p) => {
      const player = this.players.get(p.player_id)!;
      return {
        player_id: p.player_id,
        display_name: player.displayName,
        score: player.score,
        alive: player.alive,
        placement: p.placement,
        rounds_survived: player.roundsSurvived,
        avg_solve_ms: player.solveTimes.length > 0
          ? Math.round(player.solveTimes.reduce((a, b) => a + b, 0) / player.solveTimes.length)
          : null,
      };
    });

    const eloChanges = eloResults.map((r) => ({
      player_id: r.playerId,
      elo_before: r.eloBefore,
      elo_after: r.eloAfter,
      delta: r.delta,
    }));

    this.broadcast({
      type: 'match_end',
      final_standings: finalStandings,
      elo_changes: eloChanges,
    });

    // Flush results to D1
    await this.flushResults(finalStandings, eloChanges);
  }

  private async flushResults(
    standings: Array<{ player_id: string; placement: number; rounds_survived: number; avg_solve_ms: number | null }>,
    eloChanges: Array<{ player_id: string; elo_before: number; elo_after: number; delta: number }>,
  ): Promise<void> {
    try {
      const medianLevel = this.getMedianLevel();

      await this.env.DB.prepare(
        'INSERT INTO matches (id, mode, player_count, rounds_played, median_level, started_at, ended_at) VALUES (?, ?, ?, ?, ?, ?, ?)',
      )
        .bind(
          this.roomState.matchId,
          this.roomState.mode,
          this.players.size,
          this.roomState.round,
          medianLevel,
          this.roomState.startedAt || new Date().toISOString(),
          new Date().toISOString(),
        )
        .run();

      for (const standing of standings) {
        const eloChange = eloChanges.find((e) => e.player_id === standing.player_id);
        const player = this.players.get(standing.player_id)!;
        const xpEarned = this.calculateXp(standing.placement, this.players.size, player.score);

        await this.env.DB.prepare(
          'INSERT INTO match_results (match_id, player_id, placement, elo_before, elo_after, xp_earned, rounds_survived, avg_solve_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?)',
        )
          .bind(
            this.roomState.matchId,
            standing.player_id,
            standing.placement,
            eloChange?.elo_before ?? player.elo,
            eloChange?.elo_after ?? player.elo,
            xpEarned,
            standing.rounds_survived,
            standing.avg_solve_ms,
          )
          .run();

        // Update player stats
        await this.env.DB.prepare(
          `UPDATE players SET
            elo = ?,
            xp = xp + ?,
            matches_played = matches_played + 1,
            wins = wins + ?,
            updated_at = datetime('now')
          WHERE id = ?`,
        )
          .bind(
            eloChange?.elo_after ?? player.elo,
            xpEarned,
            standing.placement === 1 ? 1 : 0,
            standing.player_id,
          )
          .run();
      }
    } catch (err) {
      console.error('Failed to flush match results:', err);
    }
  }

  private calculateXp(placement: number, playerCount: number, score: number): number {
    let xp = 20; // participation
    xp += score; // points earned
    if (placement === 1) xp += Math.floor(100 * (playerCount / 4));
    else if (placement <= 3) xp += Math.floor(50 * (playerCount / 4));
    return xp;
  }

  private computePlacements(): Array<{ player_id: string; placement: number }> {
    const sorted = Array.from(this.players.values()).sort((a, b) => {
      // Alive players ranked above eliminated
      if (a.alive !== b.alive) return a.alive ? -1 : 1;
      // Higher score first
      return b.score - a.score;
    });

    return sorted.map((p, i) => ({ player_id: p.playerId, placement: i + 1 }));
  }

  private deriveSeed(round: number): bigint {
    // Simple deterministic seed — in production, use HMAC-SHA256
    const combined = `${this.roomState.secret}:${round}:${this.roomState.roundStartTime}`;
    let hash = 0n;
    for (let i = 0; i < combined.length; i++) {
      hash = (hash * 31n + BigInt(combined.charCodeAt(i))) & 0xFFFFFFFFFFFFFFFFn;
    }
    return hash;
  }

  private getAlivePlayers(): PlayerState[] {
    return Array.from(this.players.values()).filter((p) => p.alive);
  }

  private getMedianLevel(): number {
    const levels = Array.from(this.players.values())
      .map((p) => p.level)
      .sort((a, b) => a - b);
    return levels[Math.floor(levels.length / 2)] || 1;
  }

  private getStandings() {
    return Array.from(this.players.values())
      .sort((a, b) => b.score - a.score)
      .map((p) => ({
        player_id: p.playerId,
        display_name: p.displayName,
        score: p.score,
        alive: p.alive,
      }));
  }

  private getPlayerInfos() {
    return Array.from(this.players.values()).map((p) => ({
      id: p.playerId,
      display_name: p.displayName,
      avatar_url: null,
      elo: p.elo,
      level: p.level,
    }));
  }

  private broadcast(message: unknown): void {
    const json = JSON.stringify(message);
    for (const ws of this.connections.values()) {
      try {
        ws.send(json);
      } catch {
        // Connection may be closing
      }
    }
  }

  private broadcastLobbyUpdate(): void {
    this.broadcast({
      type: 'lobby_update',
      players: this.getPlayerInfos(),
    });
  }
}
