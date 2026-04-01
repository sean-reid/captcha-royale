import type { CaptchaType, DifficultyParams } from './captcha';
import type { PlayerInfo } from './player';

export type ServerMessage =
  | { type: 'lobby_update'; players: PlayerInfo[]; countdown?: number }
  | {
      type: 'round_start';
      round: number;
      seed: string; // bigint as string
      captcha_type: CaptchaType;
      difficulty: DifficultyParams;
      time_limit_ms: number;
    }
  | { type: 'player_solved'; player_id: string; time_ms: number }
  | { type: 'player_eliminated'; player_id: string; reason: 'wrong' | 'timeout' }
  | { type: 'round_end'; standings: Standing[] }
  | { type: 'match_end'; final_standings: FinalStanding[]; elo_changes: EloChange[] }
  | { type: 'match_found'; roomId: string }
  | { type: 'error'; code: string; message: string };

export type ClientMessage =
  | { type: 'submit_answer'; round: number; answer: unknown; client_time_ms: number }
  | { type: 'heartbeat' }
  | { type: 'forfeit' };

export interface Standing {
  player_id: string;
  display_name: string;
  score: number;
  alive: boolean;
}

export interface FinalStanding extends Standing {
  placement: number;
  rounds_survived: number;
  avg_solve_ms: number | null;
}

export interface EloChange {
  player_id: string;
  elo_before: number;
  elo_after: number;
  delta: number;
}

export type MatchPhase = 'idle' | 'queuing' | 'lobby' | 'playing' | 'between_rounds' | 'results';

export type GameMode = 'battle_royale' | 'sprint' | 'endless';
