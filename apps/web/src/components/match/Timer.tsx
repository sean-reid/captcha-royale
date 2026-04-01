import { useState, useEffect, useRef } from 'react';

interface TimerProps {
  durationMs: number;
  onTimeout: () => void;
  running: boolean;
}

export function Timer({ durationMs, onTimeout, running }: TimerProps) {
  const [remaining, setRemaining] = useState(durationMs);
  const startRef = useRef(Date.now());
  const calledRef = useRef(false);

  useEffect(() => {
    setRemaining(durationMs);
    startRef.current = Date.now();
    calledRef.current = false;
  }, [durationMs]);

  useEffect(() => {
    if (!running) return;

    const interval = setInterval(() => {
      const elapsed = Date.now() - startRef.current;
      const left = Math.max(0, durationMs - elapsed);
      setRemaining(left);

      if (left === 0 && !calledRef.current) {
        calledRef.current = true;
        onTimeout();
      }
    }, 50);

    return () => clearInterval(interval);
  }, [running, durationMs, onTimeout]);

  const seconds = (remaining / 1000).toFixed(1);
  const fraction = remaining / durationMs;
  const isLow = fraction < 0.25;
  const isCritical = fraction < 0.1;

  return (
    <div style={styles.container}>
      <div
        style={{
          ...styles.bar,
          width: `${fraction * 100}%`,
          background: isCritical
            ? '#e74c3c'
            : isLow
              ? '#f39c12'
              : 'linear-gradient(90deg, #2ecc71, #27ae60)',
        }}
      />
      <span
        style={{
          ...styles.text,
          color: isCritical ? '#e74c3c' : isLow ? '#f39c12' : '#e0e0e0',
        }}
      >
        {seconds}s
      </span>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    position: 'relative',
    width: '100%',
    height: '32px',
    background: '#1a1a2e',
    borderRadius: '8px',
    overflow: 'hidden',
    border: '1px solid #2a2a4e',
  },
  bar: {
    position: 'absolute',
    top: 0,
    left: 0,
    height: '100%',
    transition: 'width 0.1s linear',
    borderRadius: '8px',
  },
  text: {
    position: 'absolute',
    top: '50%',
    left: '50%',
    transform: 'translate(-50%, -50%)',
    fontFamily: 'monospace',
    fontWeight: 'bold',
    fontSize: '14px',
    zIndex: 1,
  },
};
