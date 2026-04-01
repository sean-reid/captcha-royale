import { getEloBracket, getBracketColor, xpForLevel } from '../lib/elo';
import { useAuth } from '../hooks/useAuth';

export function Profile() {
  const { player, loading } = useAuth();

  if (loading) return <div style={styles.center}>Loading...</div>;
  if (!player) return <div style={styles.center}>Not logged in</div>;

  const nextLevelXp = xpForLevel(player.level + 1);
  const currentLevelXp = xpForLevel(player.level);
  const progress = (player.xp - currentLevelXp) / (nextLevelXp - currentLevelXp);

  return (
    <div style={styles.container}>
      <div style={styles.card}>
        <h1 style={styles.name}>{player.display_name}</h1>
        <div style={styles.bracket}>
          <span style={{ color: getBracketColor(player.elo), fontWeight: 'bold' }}>
            {getEloBracket(player.elo)}
          </span>
          <span style={styles.elo}>{player.elo} ELO</span>
        </div>
        <div style={styles.statsGrid}>
          <StatBox label="Level" value={player.level} />
          <StatBox label="Matches" value={player.matches_played} />
          <StatBox label="Wins" value={player.wins} />
          <StatBox label="XP" value={player.xp} />
        </div>
        <div style={styles.xpBar}>
          <div style={{ ...styles.xpFill, width: `${progress * 100}%` }} />
        </div>
        <p style={styles.xpLabel}>
          {player.xp} / {nextLevelXp} XP to Level {player.level + 1}
        </p>
      </div>
    </div>
  );
}

function StatBox({ label, value }: { label: string; value: number }) {
  return (
    <div style={styles.stat}>
      <span style={styles.statLabel}>{label}</span>
      <span style={styles.statValue}>{value}</span>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    justifyContent: 'center',
    padding: '48px 32px',
  },
  center: {
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
    minHeight: '60vh',
    color: '#888',
  },
  card: {
    background: '#16213e',
    borderRadius: '12px',
    padding: '32px',
    border: '1px solid #2a2a4e',
    maxWidth: '500px',
    width: '100%',
  },
  name: {
    color: '#e0e0e0',
    fontSize: '28px',
    marginBottom: '8px',
  },
  bracket: {
    display: 'flex',
    gap: '12px',
    alignItems: 'center',
    marginBottom: '24px',
  },
  elo: { color: '#888', fontSize: '14px' },
  statsGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(4, 1fr)',
    gap: '12px',
    marginBottom: '24px',
  },
  stat: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    gap: '4px',
  },
  statLabel: {
    color: '#888',
    fontSize: '11px',
    textTransform: 'uppercase',
  },
  statValue: {
    color: '#e0e0e0',
    fontSize: '24px',
    fontFamily: 'monospace',
    fontWeight: 'bold',
  },
  xpBar: {
    width: '100%',
    height: '8px',
    background: '#1a1a2e',
    borderRadius: '4px',
    overflow: 'hidden',
    marginBottom: '8px',
  },
  xpFill: {
    height: '100%',
    background: 'linear-gradient(90deg, #ff6b6b, #ee5a24)',
    borderRadius: '4px',
    transition: 'width 0.3s',
  },
  xpLabel: {
    color: '#888',
    fontSize: '12px',
    textAlign: 'center',
  },
};
