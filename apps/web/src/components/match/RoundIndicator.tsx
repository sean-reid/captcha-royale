interface RoundIndicatorProps {
  round: number;
  captchaType?: string;
}

export function RoundIndicator({ round, captchaType }: RoundIndicatorProps) {
  return (
    <div style={styles.container}>
      <span style={styles.round}>Round {round}</span>
      {captchaType && <span style={styles.type}>{captchaType}</span>}
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
  },
  round: {
    color: '#e0e0e0',
    fontSize: '20px',
    fontWeight: 'bold',
    fontFamily: 'monospace',
  },
  type: {
    color: '#888',
    fontSize: '14px',
    background: '#1a1a2e',
    padding: '4px 12px',
    borderRadius: '12px',
    border: '1px solid #2a2a4e',
  },
};
