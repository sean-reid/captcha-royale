import { useState, useEffect, type ReactNode } from 'react';

function isMobileDevice(): boolean {
  // Check for touch-only device (no fine pointer = no mouse)
  if (window.matchMedia('(pointer: coarse)').matches &&
      !window.matchMedia('(pointer: fine)').matches) {
    return true;
  }
  // Fallback: check user agent for common mobile strings
  return /Android|iPhone|iPad|iPod|webOS|BlackBerry|IEMobile|Opera Mini/i.test(
    navigator.userAgent,
  );
}

export function DesktopGate({ children }: { children: ReactNode }) {
  const [isMobile, setIsMobile] = useState(false);

  useEffect(() => {
    setIsMobile(isMobileDevice());
  }, []);

  if (isMobile) {
    return (
      <div style={styles.gate}>
        <h1 style={styles.title}>CAPTCHA Royale</h1>
        <p style={styles.message}>
          This game requires a desktop browser with a keyboard and mouse.
        </p>
        <p style={styles.sub}>
          Please visit on a desktop computer to play.
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
