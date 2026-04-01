import { useNavigate } from 'react-router-dom';
import { useAuth } from '../../hooks/useAuth';
import { getEloBracket, getBracketColor } from '../../lib/elo';

export function Header() {
  const { player, logout } = useAuth();
  const navigate = useNavigate();

  return (
    <header style={styles.header}>
      <div style={styles.logo} onClick={() => navigate('/')} role="button" tabIndex={0}>
        <span style={styles.logoText}>CAPTCHA</span>
        <span style={styles.logoAccent}>Royale</span>
      </div>
      {player ? (
        <div style={styles.userInfo}>
          <span style={{ color: getBracketColor(player.elo) }}>
            {getEloBracket(player.elo)}
          </span>
          <span style={styles.elo}>{player.elo} ELO</span>
          <span style={styles.name}>{player.display_name}</span>
          <button onClick={logout} style={styles.logoutBtn}>
            Log Out
          </button>
        </div>
      ) : (
        <button onClick={() => navigate('/login')} style={styles.signInBtn}>
          Sign In
        </button>
      )}
    </header>
  );
}

const styles: Record<string, React.CSSProperties> = {
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '16px 32px',
    background: 'rgba(10, 10, 26, 0.9)',
    borderBottom: '1px solid #1a1a3e',
  },
  logo: {
    display: 'flex',
    gap: '8px',
    fontSize: '24px',
    fontWeight: 'bold',
    cursor: 'pointer',
  },
  logoText: {
    color: '#e0e0e0',
    fontFamily: 'monospace',
  },
  logoAccent: {
    color: '#ff6b6b',
    fontFamily: 'monospace',
  },
  userInfo: {
    display: 'flex',
    alignItems: 'center',
    gap: '16px',
    fontSize: '14px',
  },
  elo: { color: '#888' },
  name: { color: '#e0e0e0', fontWeight: 'bold' },
  logoutBtn: {
    background: 'none',
    border: '1px solid #444',
    color: '#888',
    padding: '4px 12px',
    borderRadius: '4px',
    cursor: 'pointer',
    fontSize: '12px',
  },
  signInBtn: {
    background: 'linear-gradient(135deg, #ff6b6b, #ee5a24)',
    border: 'none',
    borderRadius: '6px',
    color: '#fff',
    padding: '8px 20px',
    cursor: 'pointer',
    fontSize: '14px',
    fontWeight: 'bold',
    fontFamily: 'inherit',
  },
};
