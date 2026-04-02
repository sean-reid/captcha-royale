import { useState, useCallback, useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../hooks/useAuth';
import { Button } from '../components/ui/Button';
import { getBracketColorByName } from '../lib/elo';
import { wsUrl } from '../lib/config';

export function Queue() {
  const navigate = useNavigate();
  const { player, loading, refresh } = useAuth();

  // Refresh profile on mount so ELO is current
  useEffect(() => { refresh(); }, []);
  const [selectedMode, setSelectedMode] = useState<'battle_royale' | 'sprint' | null>(null);
  const [bracket, setBracket] = useState('');
  const [queuePlayers, setQueuePlayers] = useState<Array<{ id: string; display_name: string; elo: number }>>([]);
  const [waitTime, setWaitTime] = useState(0);
  const wsRef = useRef<WebSocket | null>(null);

  const handleWsMessage = useCallback(
    (event: MessageEvent) => {
      try {
        const msg = JSON.parse(event.data);
        if (msg.type === 'queue_status') {
          setBracket(msg.bracket as string);
          setQueuePlayers(msg.players || []);
        } else if (msg.type === 'match_found') {
          navigate(`/match/${msg.roomId}`);
        }
      } catch { /* ignore */ }
    },
    [navigate],
  );

  const joinQueue = useCallback((mode: 'battle_royale' | 'sprint') => {
    if (wsRef.current) return; // already queued
    setSelectedMode(mode);

    const url = wsUrl(`/match/queue?mode=${mode}`);
    const ws = new WebSocket(url);
    wsRef.current = ws;

    ws.onmessage = (e) => handleWsMessage(e);
    ws.onclose = () => { wsRef.current = null; };
    ws.onerror = () => { ws.close(); };
  }, [handleWsMessage]);

  const leaveQueue = useCallback(() => {
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }
    setSelectedMode(null);
    setQueuePlayers([]);
    setWaitTime(0);
  }, []);

  // Cleanup on unmount
  useEffect(() => {
    return () => { wsRef.current?.close(); };
  }, []);

  // Track wait time
  useEffect(() => {
    if (!selectedMode) return;
    const interval = setInterval(() => setWaitTime((t) => t + 1), 1000);
    return () => clearInterval(interval);
  }, [selectedMode]);

  if (!selectedMode) {
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
                onSelect={() => joinQueue('battle_royale')}
              />
              <ModeCard
                name="Sprint"
                description="2-8 players. Solve 10 CAPTCHAs as fast as possible. Pure speed."
                players="2-8"
                onSelect={() => joinQueue('sprint')}
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
      <h1 style={styles.title}>
        Finding {selectedMode === 'sprint' ? 'Sprint' : 'Battle Royale'} Match...
      </h1>
      <div style={styles.queueInfo}>
        <div style={styles.spinner} />
        {bracket && (
          <p style={{ color: getBracketColorByName(bracket), fontSize: '16px' }}>
            Bracket: <strong>{bracket.charAt(0).toUpperCase() + bracket.slice(1)}</strong>
          </p>
        )}
        <p style={styles.detail}>{queuePlayers.length} player{queuePlayers.length !== 1 ? 's' : ''} in queue</p>
        {queuePlayers.length > 0 && (
          <div style={styles.playerList}>
            {queuePlayers.map((p) => (
              <div key={p.id} style={styles.playerRow}>
                <span style={styles.playerName}>{p.display_name}</span>
                <span style={styles.playerElo}>{p.elo}</span>
              </div>
            ))}
          </div>
        )}
        <p style={styles.detail}>Waiting: {waitTime}s</p>
        {queuePlayers.length < (selectedMode === 'sprint' ? 2 : 4) && (
          <p style={styles.detail}>
            Need {(selectedMode === 'sprint' ? 2 : 4) - queuePlayers.length} more to start
          </p>
        )}
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
}: {
  name: string;
  description: string;
  players: string;
  onSelect: () => void;
}) {
  return (
    <div style={styles.modeCard}>
      <h3 style={styles.modeName}>{name}</h3>
      <p style={styles.modeDesc}>{description}</p>
      <p style={styles.modePlayers}>{players} players</p>
      <Button onClick={onSelect} size="sm">
        Queue Up
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
  playerList: {
    display: 'flex',
    flexDirection: 'column',
    gap: '4px',
    width: '100%',
  },
  playerRow: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '6px 12px',
    background: '#1a1a2e',
    borderRadius: '6px',
    fontSize: '13px',
  },
  playerName: {
    color: '#e0e0e0',
    fontWeight: 'bold',
  },
  playerElo: {
    color: '#888',
    fontFamily: 'monospace',
    fontSize: '12px',
  },
};
