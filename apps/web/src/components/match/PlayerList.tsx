import type { Standing } from '../../types/match';

interface PlayerListProps {
  standings: Standing[];
  currentPlayerId?: string;
}

export function PlayerList({ standings, currentPlayerId }: PlayerListProps) {
  return (
    <div style={styles.container}>
      <h3 style={styles.title}>Players</h3>
      <div style={styles.list}>
        {standings.map((s, i) => (
          <div
            key={s.player_id}
            style={{
              ...styles.row,
              opacity: s.alive ? 1 : 0.4,
              borderLeft: s.player_id === currentPlayerId ? '3px solid #ff6b6b' : '3px solid transparent',
            }}
          >
            <span style={styles.rank}>#{i + 1}</span>
            <span style={{
              ...styles.name,
              textDecoration: s.alive ? 'none' : 'line-through',
            }}>
              {s.display_name}
            </span>
            <span style={styles.score}>{s.score}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    background: '#16213e',
    borderRadius: '8px',
    padding: '16px',
    border: '1px solid #2a2a4e',
    minWidth: '200px',
  },
  title: {
    color: '#888',
    fontSize: '12px',
    textTransform: 'uppercase',
    letterSpacing: '1px',
    marginBottom: '12px',
  },
  list: {
    display: 'flex',
    flexDirection: 'column',
    gap: '4px',
  },
  row: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    padding: '4px 8px',
    borderRadius: '4px',
  },
  rank: {
    color: '#666',
    fontFamily: 'monospace',
    fontSize: '12px',
    width: '24px',
  },
  name: {
    color: '#e0e0e0',
    fontSize: '14px',
    flex: 1,
  },
  score: {
    color: '#ff6b6b',
    fontFamily: 'monospace',
    fontWeight: 'bold',
    fontSize: '14px',
  },
};
