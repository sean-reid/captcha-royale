import { useAuth } from '../hooks/useAuth';
import { Button } from '../components/ui/Button';

export function Login() {
  const { login } = useAuth();

  return (
    <div style={styles.container}>
      <h1 style={styles.title}>Sign In</h1>
      <p style={styles.subtitle}>Choose a provider to get started</p>
      <div style={styles.providers}>
        <Button onClick={() => login('google')} variant="secondary" size="lg" style={styles.provider}>
          Sign in with Google
        </Button>
        <Button onClick={() => login('discord')} variant="secondary" size="lg" style={styles.provider}>
          Sign in with Discord
        </Button>
        <Button onClick={() => login('github')} variant="secondary" size="lg" style={styles.provider}>
          Sign in with GitHub
        </Button>
      </div>
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
    gap: '16px',
  },
  title: {
    color: '#e0e0e0',
    fontSize: '32px',
    fontFamily: 'monospace',
  },
  subtitle: {
    color: '#888',
    fontSize: '14px',
    marginBottom: '16px',
  },
  providers: {
    display: 'flex',
    flexDirection: 'column',
    gap: '12px',
    width: '300px',
  },
  provider: {
    width: '100%',
    justifyContent: 'center',
  },
};
