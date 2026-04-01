import { getBracketColor, getEloBracket } from '../lib/elo';

// Placeholder with sample data until backend is connected
const SAMPLE_LEADERBOARD = Array.from({ length: 20 }, (_, i) => ({
  rank: i + 1,
  display_name: `Player${i + 1}`,
  elo: 1500 - i * 25,
  level: 50 - i,
  wins: 100 - i * 4,
}));

export function Leaderboard() {
  return (
    <div style={styles.container}>
      <h1 style={styles.title}>Leaderboard</h1>
      <div style={styles.table}>
        <div style={styles.headerRow}>
          <span style={{ ...styles.cell, width: '60px' }}>Rank</span>
          <span style={{ ...styles.cell, flex: 1 }}>Player</span>
          <span style={{ ...styles.cell, width: '80px' }}>Bracket</span>
          <span style={{ ...styles.cell, width: '80px' }}>ELO</span>
          <span style={{ ...styles.cell, width: '60px' }}>Level</span>
          <span style={{ ...styles.cell, width: '60px' }}>Wins</span>
        </div>
        {SAMPLE_LEADERBOARD.map((entry) => (
          <div key={entry.rank} style={styles.row}>
            <span style={{ ...styles.cell, width: '60px', color: '#888', fontFamily: 'monospace' }}>
              #{entry.rank}
            </span>
            <span style={{ ...styles.cell, flex: 1, color: '#e0e0e0', fontWeight: 'bold' }}>
              {entry.display_name}
            </span>
            <span style={{ ...styles.cell, width: '80px', color: getBracketColor(entry.elo), fontSize: '12px' }}>
              {getEloBracket(entry.elo)}
            </span>
            <span style={{ ...styles.cell, width: '80px', color: '#ff6b6b', fontFamily: 'monospace' }}>
              {entry.elo}
            </span>
            <span style={{ ...styles.cell, width: '60px', color: '#888', fontFamily: 'monospace' }}>
              {entry.level}
            </span>
            <span style={{ ...styles.cell, width: '60px', color: '#888', fontFamily: 'monospace' }}>
              {entry.wins}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    padding: '32px',
    maxWidth: '800px',
    margin: '0 auto',
  },
  title: {
    color: '#e0e0e0',
    fontSize: '28px',
    fontFamily: 'monospace',
    marginBottom: '24px',
  },
  table: {
    background: '#16213e',
    borderRadius: '12px',
    border: '1px solid #2a2a4e',
    overflow: 'hidden',
  },
  headerRow: {
    display: 'flex',
    padding: '12px 16px',
    borderBottom: '1px solid #2a2a4e',
    color: '#888',
    fontSize: '11px',
    textTransform: 'uppercase',
    letterSpacing: '1px',
  },
  row: {
    display: 'flex',
    padding: '10px 16px',
    borderBottom: '1px solid rgba(42, 42, 78, 0.5)',
    fontSize: '14px',
  },
  cell: {
    display: 'flex',
    alignItems: 'center',
  },
};
