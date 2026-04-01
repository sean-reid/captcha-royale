import { useNavigate } from 'react-router-dom';
import { Button } from '../components/ui/Button';

export function Results() {
  const navigate = useNavigate();

  return (
    <div style={styles.container}>
      <h1 style={styles.title}>Match Results</h1>
      <p style={styles.subtitle}>No match data available.</p>
      <Button onClick={() => navigate('/')}>Back to Home</Button>
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
  },
};
