import { useState, useCallback, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useWebSocket } from '../hooks/useWebSocket';
import { useAuth } from '../hooks/useAuth';
import { Button } from '../components/ui/Button';
import { getEloBracket, getBracketColor } from '../lib/elo';
import { wsUrl } from '../lib/config';
import type { ServerMessage } from '../types/match';

export function Queue() {
  const navigate = useNavigate();
  const { player, loading } = useAuth();
  const [inQueue, setInQueue] = useState(false);
  const [bracket, setBracket] = useState('');
  const [playersInBracket, setPlayersInBracket] = useState(0);
  const [waitTime, setWaitTime] = useState(0);

  const handleMessage = useCallback(
    (data: unknown) => {
      const msg = data as ServerMessage | { type: string; bracket?: string; playersInBracket?: number };
      if (msg.type === 'queue_status') {
        const qs = msg as { bracket: string; playersInBracket: number };
        setBracket(qs.bracket);
        setPlayersInBracket(qs.playersInBracket);
      } else if (msg.type === 'match_found') {
        const mf = msg as { roomId: string };
        navigate(`/match/${mf.roomId}`);
      }
    },
    [navigate],
  );

  const queueWsUrl = wsUrl('/match/queue');

  const { connected, connect, disconnect } = useWebSocket({
    url: queueWsUrl,
    onMessage: handleMessage,
  });

  const joinQueue = () => {
    if (inQueue) return; // prevent double-queue
    setInQueue(true);
    connect();
  };

  const leaveQueue = () => {
    setInQueue(false);
    disconnect();
    setWaitTime(0);
  };

  // Track wait time
  // Track wait time
  useEffect(() => {
    if (!inQueue) return;
    const interval = setInterval(() => setWaitTime((t) => t + 1), 1000);
    return () => clearInterval(interval);
  }, [inQueue]);

  // Disconnect on unmount
  useEffect(() => {
    return () => { disconnect(); };
  }, [disconnect]);

  if (!inQueue) {
    return (
      <div style={styles.container}>
        <h1 style={styles.title}>Matchmaking</h1>
        {!loading && !player ? (
          <>
            <p style={styles.subtitle}>Sign in to play multiplayer</p>
            <Button onClick={() => navigate('/login')}>Sign In</Button>
            <Button variant="secondary" onClick={() => navigate('/play')} style={{ marginTop: '8px' }}>
              Or Play Endless Mode (Solo)
            </Button>
          </>
        ) : (
          <>
            <p style={styles.subtitle}>Find a match against other players</p>
            <div style={styles.modes}>
              <ModeCard
                name="Battle Royale"
                description="4-16 players. Wrong answer = eliminated. Last one standing wins."
                players="4-16"
                onSelect={joinQueue}
              />
              <ModeCard
                name="Sprint"
                description="2-8 players. Solve 10 CAPTCHAs as fast as possible. Pure speed."
                players="2-8"
                onSelect={joinQueue}
                disabled
              />
            </div>
            <Button variant="secondary" onClick={() => navigate('/play')}>
              Or Play Endless Mode (Solo)
            </Button>
          </>
        )}
      </div>
    );
  }

  return (
    <div style={styles.container}>
      <h1 style={styles.title}>Finding Match...</h1>
      <div style={styles.queueInfo}>
        <div style={styles.spinner} />
        {bracket && (
          <p style={{ color: getBracketColor(1000), fontSize: '16px' }}>
            Bracket: <strong>{bracket.charAt(0).toUpperCase() + bracket.slice(1)}</strong>
          </p>
        )}
        <p style={styles.detail}>{playersInBracket} players in queue</p>
        <p style={styles.detail}>Waiting: {waitTime}s</p>
        {waitTime > 15 && (
          <p style={{ color: '#f39c12', fontSize: '13px' }}>
            Expanding search to nearby brackets...
          </p>
        )}
      </div>
      <Button variant="danger" onClick={leaveQueue}>
        Leave Queue
      </Button>
    </div>
  );
}

function ModeCard({
  name,
  description,
  players,
  onSelect,
  disabled,
}: {
  name: string;
  description: string;
  players: string;
  onSelect: () => void;
  disabled?: boolean;
}) {
  return (
    <div style={{ ...styles.modeCard, opacity: disabled ? 0.5 : 1 }}>
      <h3 style={styles.modeName}>{name}</h3>
      <p style={styles.modeDesc}>{description}</p>
      <p style={styles.modePlayers}>{players} players</p>
      <Button onClick={onSelect} disabled={disabled} size="sm">
        {disabled ? 'Coming Soon' : 'Queue Up'}
      </Button>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    minHeight: '60vh',
    gap: '24px',
    padding: '32px',
  },
  title: {
    color: '#e0e0e0',
    fontSize: '32px',
    fontFamily: 'monospace',
  },
  subtitle: { color: '#888', fontSize: '16px' },
  modes: {
    display: 'flex',
    gap: '24px',
    marginBottom: '16px',
  },
  modeCard: {
    background: '#16213e',
    borderRadius: '12px',
    padding: '24px',
    border: '1px solid #2a2a4e',
    width: '280px',
    display: 'flex',
    flexDirection: 'column',
    gap: '12px',
  },
  modeName: { color: '#ff6b6b', fontSize: '18px', margin: 0 },
  modeDesc: { color: '#888', fontSize: '14px', lineHeight: '1.5', margin: 0 },
  modePlayers: { color: '#666', fontSize: '12px', fontFamily: 'monospace', margin: 0 },
  queueInfo: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    gap: '12px',
    padding: '32px',
    background: '#16213e',
    borderRadius: '12px',
    border: '1px solid #2a2a4e',
    minWidth: '300px',
  },
  spinner: {
    width: '40px',
    height: '40px',
    border: '3px solid #2a2a4e',
    borderTopColor: '#ff6b6b',
    borderRadius: '50%',
    animation: 'spin 1s linear infinite',
  },
  detail: { color: '#888', fontSize: '14px' },
};
