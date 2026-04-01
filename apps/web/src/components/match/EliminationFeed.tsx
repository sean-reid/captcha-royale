interface EliminationEntry {
  player_id: string;
  display_name: string;
  reason: string;
}

interface EliminationFeedProps {
  entries: EliminationEntry[];
}

export function EliminationFeed({ entries }: EliminationFeedProps) {
  if (entries.length === 0) return null;

  // Show last 5
  const recent = entries.slice(-5);

  return (
    <div style={styles.container}>
      {recent.map((entry, i) => (
        <div key={`${entry.player_id}-${i}`} style={styles.entry}>
          <span style={styles.icon}>&#x2620;</span>
          <span style={styles.name}>{entry.display_name}</span>
          <span style={styles.reason}>
            {entry.reason === 'timeout' ? 'timed out' : 'wrong answer'}
          </span>
        </div>
      ))}
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    gap: '4px',
    maxHeight: '150px',
    overflow: 'hidden',
  },
  entry: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    padding: '4px 8px',
    background: 'rgba(231, 76, 60, 0.1)',
    borderRadius: '4px',
    borderLeft: '2px solid #e74c3c',
    fontSize: '13px',
    animation: 'fadeIn 0.3s ease-in',
  },
  icon: { fontSize: '14px' },
  name: { color: '#e0e0e0', fontWeight: 'bold' },
  reason: { color: '#888' },
};
