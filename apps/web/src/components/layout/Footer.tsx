export function Footer() {
  return (
    <footer style={styles.footer}>
      <span style={styles.text}>CAPTCHA Royale — Desktop Only</span>
    </footer>
  );
}

const styles: Record<string, React.CSSProperties> = {
  footer: {
    padding: '12px 32px',
    textAlign: 'center',
    borderTop: '1px solid #1a1a3e',
    color: '#555',
    fontSize: '12px',
  },
  text: {
    fontFamily: 'monospace',
  },
};
