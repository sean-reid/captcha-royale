import { useState, useRef, useEffect } from 'react';
import type { CaptchaInstance, PlayerAnswer } from '../../types/captcha';

interface MathCaptchaProps {
  instance: CaptchaInstance;
  onSubmit: (answer: PlayerAnswer) => void;
  disabled?: boolean;
}

export function MathCaptcha({ instance, onSubmit, disabled }: MathCaptchaProps) {
  const [value, setValue] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    setValue('');
    inputRef.current?.focus();
  }, [instance]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const num = parseFloat(value);
    if (!isNaN(num) && !disabled) {
      onSubmit({ Number: num });
    }
  };

  const svg = 'Svg' in instance.render_data ? instance.render_data.Svg : '';

  return (
    <div style={styles.container}>
      <div style={styles.label}>Solve the equation</div>
      <div
        style={styles.svgContainer}
        dangerouslySetInnerHTML={{ __html: svg }}
      />
      <form onSubmit={handleSubmit} style={styles.form}>
        <input
          ref={inputRef}
          type="text"
          inputMode="numeric"
          value={value}
          onChange={(e) => setValue(e.target.value.replace(/[^0-9.\-]/g, ''))}
          placeholder="Answer..."
          disabled={disabled}
          style={styles.input}
          autoComplete="off"
        />
        <button type="submit" disabled={disabled || !value} style={styles.submitBtn}>
          Submit
        </button>
      </form>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    gap: '16px',
  },
  label: {
    color: '#888',
    fontSize: '14px',
    textTransform: 'uppercase',
    letterSpacing: '1px',
  },
  svgContainer: {
    borderRadius: '8px',
    overflow: 'hidden',
    border: '2px solid #2a2a4e',
  },
  form: {
    display: 'flex',
    gap: '8px',
  },
  input: {
    background: '#1a1a2e',
    border: '2px solid #2a2a4e',
    borderRadius: '8px',
    color: '#e0e0e0',
    fontSize: '24px',
    fontFamily: 'monospace',
    padding: '8px 16px',
    textAlign: 'center',
    width: '160px',
    outline: 'none',
  },
  submitBtn: {
    background: 'linear-gradient(135deg, #ff6b6b, #ee5a24)',
    border: 'none',
    borderRadius: '8px',
    color: '#fff',
    fontSize: '16px',
    fontWeight: 'bold',
    padding: '8px 24px',
    cursor: 'pointer',
  },
};
