import { useCallback, useEffect, useRef, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useWebSocket } from '../hooks/useWebSocket';
import { useMatchState } from '../hooks/useMatchState';
import { useCaptchaEngine } from '../hooks/useCaptchaEngine';
import { wsUrl } from '../lib/config';
import { CaptchaRenderer } from '../components/captcha/CaptchaRenderer';
import { Timer } from '../components/match/Timer';
import { PlayerList } from '../components/match/PlayerList';
import { RoundIndicator } from '../components/match/RoundIndicator';
import { EliminationFeed } from '../components/match/EliminationFeed';
import { Button } from '../components/ui/Button';
import type { CaptchaInstance, CaptchaType, PlayerAnswer } from '../types/captcha';
import type { ServerMessage } from '../types/match';

export function Match() {
  const { id: roomId } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { ready, generate } = useCaptchaEngine();
  const { state, handleMessage, reset } = useMatchState();
  const [currentCaptcha, setCurrentCaptcha] = useState<CaptchaInstance | null>(null);
  const [answered, setAnswered] = useState(false);
  const roundStartRef = useRef(0);

  // Use refs for the message handler so the WebSocket always calls the latest version
  const readyRef = useRef(ready);
  const generateRef = useRef(generate);
  const handleMessageRef = useRef(handleMessage);
  readyRef.current = ready;
  generateRef.current = generate;
  handleMessageRef.current = handleMessage;

  const onServerMessage = useCallback(
    (data: unknown) => {
      const msg = data as ServerMessage;
      handleMessageRef.current(msg);

      if (msg.type === 'round_start' && readyRef.current) {
        try {
          const captcha = generateRef.current(
            BigInt(msg.seed),
            msg.captcha_type as CaptchaType,
            msg.difficulty.level,
            msg.round,
          );
          setCurrentCaptcha(captcha);
          setAnswered(false);
          roundStartRef.current = Date.now();
        } catch (err) {
          console.error('CAPTCHA generation failed:', err);
        }
      }

      if (msg.type === 'round_end' || msg.type === 'match_end') {
        setCurrentCaptcha(null);
      }
    },
    [], // stable — uses refs internally
  );

  const roomWsUrl = roomId ? wsUrl(`/match/room/${roomId}`) : '';

  const { connected, connect, send, disconnect } = useWebSocket({
    url: roomWsUrl,
    onMessage: onServerMessage,
  });

  // Connect once when roomId is available
  const connectedRef = useRef(false);
  useEffect(() => {
    if (roomId && !connectedRef.current) {
      connectedRef.current = true;
      connect();
    }
    return () => {
      if (connectedRef.current) {
        disconnect();
        connectedRef.current = false;
      }
    };
  }, [roomId]); // eslint-disable-line react-hooks/exhaustive-deps

  const handleSubmit = useCallback(
    (answer: PlayerAnswer) => {
      if (answered) return;
      setAnswered(true);
      const clientTimeMs = Date.now() - roundStartRef.current;
      send({
        type: 'submit_answer',
        round: state.round,
        answer,
        client_time_ms: clientTimeMs,
      });
    },
    [answered, state.round, send],
  );

  const handleTimeout = useCallback(() => {
    if (!answered) {
      setAnswered(true);
    }
  }, [answered]);

  const handleForfeit = useCallback(() => {
    send({ type: 'forfeit' });
  }, [send]);

  // Lobby phase
  if (state.phase === 'lobby') {
    return (
      <div style={styles.container}>
        <h1 style={styles.title}>Match Lobby</h1>
        <p style={styles.subtitle}>
          {state.players.length} player{state.players.length !== 1 ? 's' : ''} connected
        </p>
        <div style={styles.playerGrid}>
          {state.players.map((p) => (
            <div key={p.id} style={styles.playerCard}>
              <span style={styles.playerName}>{p.display_name}</span>
              <span style={styles.playerElo}>{p.elo} ELO</span>
            </div>
          ))}
        </div>
        <p style={styles.waiting}>Waiting for more players...</p>
      </div>
    );
  }

  // Results phase
  if (state.phase === 'results') {
    return (
      <div style={styles.container}>
        <h1 style={styles.title}>Match Results</h1>
        <div style={styles.resultsList}>
          {state.finalStandings.map((s) => {
            const eloChange = state.eloChanges.find((e) => e.player_id === s.player_id);
            return (
              <div key={s.player_id} style={styles.resultRow}>
                <span style={styles.placement}>#{s.placement}</span>
                <span style={styles.playerName}>{s.display_name}</span>
                <span style={styles.score}>{s.score} pts</span>
                <span style={styles.rounds}>{s.rounds_survived} rounds</span>
                {eloChange && (
                  <span
                    style={{
                      color: eloChange.delta >= 0 ? '#2ecc71' : '#e74c3c',
                      fontFamily: 'monospace',
                      fontWeight: 'bold',
                    }}
                  >
                    {eloChange.delta >= 0 ? '+' : ''}
                    {eloChange.delta}
                  </span>
                )}
              </div>
            );
          })}
        </div>
        <div style={{ display: 'flex', gap: '12px', marginTop: '24px' }}>
          <Button onClick={() => navigate('/queue')}>Play Again</Button>
          <Button variant="secondary" onClick={() => navigate('/')}>
            Home
          </Button>
        </div>
      </div>
    );
  }

  // Playing phase
  return (
    <div style={styles.gameLayout}>
      <div style={styles.sidebar}>
        <PlayerList standings={state.standings} />
        <EliminationFeed entries={state.eliminationFeed} />
      </div>
      <div style={styles.mainArea}>
        <div style={styles.hud}>
          <RoundIndicator round={state.round} captchaType={state.currentCaptchaType ?? undefined} />
          <Button variant="danger" size="sm" onClick={handleForfeit}>
            Forfeit
          </Button>
        </div>

        {currentCaptcha && (
          <>
            <Timer
              durationMs={currentCaptcha.time_limit_ms}
              onTimeout={handleTimeout}
              running={!answered}
            />
            <CaptchaRenderer
              instance={currentCaptcha}
              onSubmit={handleSubmit}
              disabled={answered}
            />
          </>
        )}

        {answered && state.phase === 'playing' && (
          <p style={{ color: '#888', textAlign: 'center', marginTop: '16px' }}>
            Waiting for other players...
          </p>
        )}

        {state.phase === 'between_rounds' && (
          <p style={{ color: '#e0e0e0', textAlign: 'center', fontSize: '20px', marginTop: '32px' }}>
            Next round starting...
          </p>
        )}
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    padding: '32px',
    gap: '16px',
  },
  title: { color: '#e0e0e0', fontSize: '32px', fontFamily: 'monospace' },
  subtitle: { color: '#888', fontSize: '16px' },
  waiting: { color: '#666', fontSize: '14px', fontStyle: 'italic' },
  playerGrid: {
    display: 'flex',
    flexWrap: 'wrap',
    gap: '12px',
    justifyContent: 'center',
    maxWidth: '600px',
  },
  playerCard: {
    background: '#16213e',
    border: '1px solid #2a2a4e',
    borderRadius: '8px',
    padding: '12px 20px',
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    gap: '4px',
  },
  playerName: { color: '#e0e0e0', fontWeight: 'bold', fontSize: '14px' },
  playerElo: { color: '#888', fontSize: '12px', fontFamily: 'monospace' },
  gameLayout: {
    display: 'flex',
    gap: '24px',
    padding: '24px',
    maxWidth: '1100px',
    margin: '0 auto',
  },
  sidebar: {
    display: 'flex',
    flexDirection: 'column',
    gap: '16px',
    width: '220px',
    flexShrink: 0,
  },
  mainArea: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    gap: '16px',
    maxWidth: '700px',
  },
  hud: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  resultsList: {
    display: 'flex',
    flexDirection: 'column',
    gap: '8px',
    width: '100%',
    maxWidth: '600px',
  },
  resultRow: {
    display: 'flex',
    alignItems: 'center',
    gap: '16px',
    padding: '12px 16px',
    background: '#16213e',
    borderRadius: '8px',
    border: '1px solid #2a2a4e',
  },
  placement: {
    color: '#ff6b6b',
    fontWeight: 'bold',
    fontFamily: 'monospace',
    fontSize: '18px',
    width: '40px',
  },
  score: { color: '#ff6b6b', fontFamily: 'monospace', fontWeight: 'bold' },
  rounds: { color: '#888', fontSize: '13px' },
};
