import { useNavigate } from 'react-router-dom';
import { Button } from '../components/ui/Button';

export function Home() {
  const navigate = useNavigate();

  return (
    <div style={styles.container}>
      <div style={styles.hero}>
        <h1 style={styles.title}>
          <span style={styles.titleMain}>CAPTCHA</span>
          <span style={styles.titleAccent}>Royale</span>
        </h1>
        <p style={styles.subtitle}>
          Race to solve CAPTCHAs. Outsmart your opponents. Last player standing wins.
        </p>
      </div>

      <div style={styles.actions}>
        <Button size="lg" onClick={() => navigate('/play')}>
          Play Endless Mode
        </Button>
        <Button size="lg" variant="secondary" onClick={() => navigate('/queue')}>
          Multiplayer (Coming Soon)
        </Button>
      </div>

      <div style={styles.features}>
        <FeatureCard
          title="Procedural CAPTCHAs"
          description="Every challenge is unique, generated from seeds — no two games are the same."
        />
        <FeatureCard
          title="4 Difficulty Tiers"
          description="From basic text to nightmare-level metamorphic puzzles that push human perception."
        />
        <FeatureCard
          title="Competitive ELO"
          description="Ranked matchmaking puts you against equally skilled opponents."
        />
      </div>
    </div>
  );
}

function FeatureCard({ title, description }: { title: string; description: string }) {
  return (
    <div style={styles.card}>
      <h3 style={styles.cardTitle}>{title}</h3>
      <p style={styles.cardDesc}>{description}</p>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    padding: '60px 32px',
    gap: '48px',
  },
  hero: {
    textAlign: 'center',
  },
  title: {
    display: 'flex',
    gap: '16px',
    fontSize: '56px',
    fontFamily: 'monospace',
    marginBottom: '16px',
    justifyContent: 'center',
  },
  titleMain: { color: '#e0e0e0' },
  titleAccent: { color: '#ff6b6b' },
  subtitle: {
    color: '#888',
    fontSize: '18px',
    maxWidth: '500px',
  },
  actions: {
    display: 'flex',
    gap: '16px',
  },
  features: {
    display: 'grid',
    gridTemplateColumns: 'repeat(3, 1fr)',
    gap: '24px',
    maxWidth: '900px',
    width: '100%',
  },
  card: {
    background: '#16213e',
    borderRadius: '12px',
    padding: '24px',
    border: '1px solid #2a2a4e',
  },
  cardTitle: {
    color: '#ff6b6b',
    fontSize: '16px',
    marginBottom: '8px',
  },
  cardDesc: {
    color: '#888',
    fontSize: '14px',
    lineHeight: '1.5',
  },
};
