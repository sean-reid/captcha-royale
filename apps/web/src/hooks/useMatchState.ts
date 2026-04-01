import { useState, useCallback } from 'react';
import type { MatchPhase, GameMode, ServerMessage } from '../types/match';
import type { Standing, FinalStanding, EloChange } from '../types/match';
import type { PlayerInfo } from '../types/player';
import type { CaptchaType, DifficultyParams } from '../types/captcha';

export interface MatchState {
  phase: MatchPhase;
  mode: GameMode;
  round: number;
  players: PlayerInfo[];
  standings: Standing[];
  finalStandings: FinalStanding[];
  eloChanges: EloChange[];
  currentSeed: bigint | null;
  currentCaptchaType: CaptchaType | null;
  currentDifficulty: DifficultyParams | null;
  timeLimit: number;
  eliminationFeed: Array<{ player_id: string; display_name: string; reason: string }>;
}

const initialState: MatchState = {
  phase: 'idle',
  mode: 'battle_royale',
  round: 0,
  players: [],
  standings: [],
  finalStandings: [],
  eloChanges: [],
  currentSeed: null,
  currentCaptchaType: null,
  currentDifficulty: null,
  timeLimit: 0,
  eliminationFeed: [],
};

export function useMatchState() {
  const [state, setState] = useState<MatchState>(initialState);

  const handleMessage = useCallback((msg: ServerMessage) => {
    switch (msg.type) {
      case 'lobby_update':
        setState((s) => ({
          ...s,
          phase: 'lobby',
          players: msg.players,
        }));
        break;

      case 'round_start':
        setState((s) => ({
          ...s,
          phase: 'playing',
          round: msg.round,
          currentSeed: BigInt(msg.seed),
          currentCaptchaType: msg.captcha_type,
          currentDifficulty: msg.difficulty,
          timeLimit: msg.time_limit_ms,
        }));
        break;

      case 'player_eliminated': {
        const player = state.players.find((p) => p.id === msg.player_id);
        setState((s) => ({
          ...s,
          eliminationFeed: [
            ...s.eliminationFeed,
            {
              player_id: msg.player_id,
              display_name: player?.display_name ?? 'Unknown',
              reason: msg.reason,
            },
          ],
        }));
        break;
      }

      case 'round_end':
        setState((s) => ({
          ...s,
          phase: 'between_rounds',
          standings: msg.standings,
        }));
        break;

      case 'match_end':
        setState((s) => ({
          ...s,
          phase: 'results',
          finalStandings: msg.final_standings,
          eloChanges: msg.elo_changes,
        }));
        break;

      default:
        break;
    }
  }, [state.players]);

  const reset = useCallback(() => {
    setState(initialState);
  }, []);

  const setPhase = useCallback((phase: MatchPhase) => {
    setState((s) => ({ ...s, phase }));
  }, []);

  const setMode = useCallback((mode: GameMode) => {
    setState((s) => ({ ...s, mode }));
  }, []);

  return { state, handleMessage, reset, setPhase, setMode };
}
