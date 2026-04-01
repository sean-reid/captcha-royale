import { useState, useEffect, type ReactNode } from 'react';

const MIN_WIDTH = 1024;

export function DesktopGate({ children }: { children: ReactNode }) {
  const [isDesktop, setIsDesktop] = useState(window.innerWidth >= MIN_WIDTH);

  useEffect(() => {
    const handleResize = () => setIsDesktop(window.innerWidth >= MIN_WIDTH);
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  if (!isDesktop) {
    return (
      <div style={styles.gate}>
        <h1 style={styles.title}>CAPTCHA Royale</h1>
        <p style={styles.message}>
          This game requires a desktop browser with a keyboard and mouse.
        </p>
        <p style={styles.sub}>
          Please visit on a desktop with a window at least 1024px wide.
        </p>
      </div>
    );
  }

  return <>{children}</>;
}

const styles: Record<string, React.CSSProperties> = {
  gate: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    minHeight: '100vh',
    padding: '32px',
    textAlign: 'center',
    background: '#0a0a1a',
  },
  title: {
    fontSize: '32px',
    color: '#ff6b6b',
    fontFamily: 'monospace',
    marginBottom: '16px',
  },
  message: {
    color: '#e0e0e0',
    fontSize: '18px',
    marginBottom: '8px',
  },
  sub: {
    color: '#888',
    fontSize: '14px',
  },
};
