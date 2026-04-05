import { useState, useRef, useEffect, useCallback } from 'react';
import type { CaptchaInstance, PlayerAnswer } from '../../types/captcha';

interface SvgMultiTextCaptchaProps {
  instance: CaptchaInstance;
  onSubmit: (answer: PlayerAnswer) => void;
  disabled?: boolean;
  label?: string;
  fieldLabels?: string[];
  fieldCount: number;
  placeholders?: string[];
}

/**
 * Renderer for SVG-based CAPTCHAs that require multiple text answers.
 * Shows separate input fields instead of requiring pipe-separated input.
 */
export function SvgMultiTextCaptcha({
  instance,
  onSubmit,
  disabled,
  label,
  fieldLabels,
  fieldCount,
  placeholders,
}: SvgMultiTextCaptchaProps) {
  const [values, setValues] = useState<string[]>(() => Array(fieldCount).fill(''));
  const inputRefs = useRef<(HTMLInputElement | null)[]>([]);

  useEffect(() => {
    setValues(Array(fieldCount).fill(''));
    inputRefs.current[0]?.focus();
  }, [instance, fieldCount]);

  const handleChange = useCallback((index: number, value: string) => {
    setValues((prev) => {
      const next = [...prev];
      next[index] = value;
      return next;
    });
  }, []);

  const handleKeyDown = useCallback(
    (index: number, e: React.KeyboardEvent<HTMLInputElement>) => {
      // Auto-advance to next field on Enter (unless last field)
      if (e.key === 'Enter' && index < fieldCount - 1 && values[index].trim()) {
        e.preventDefault();
        inputRefs.current[index + 1]?.focus();
      }
    },
    [fieldCount, values],
  );

  const allFilled = values.every((v) => v.trim().length > 0);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (allFilled && !disabled) {
      onSubmit({ Text: values.map((v) => v.trim()).join('|') });
    }
  };

  const svg = 'Svg' in instance.render_data ? instance.render_data.Svg : '';

  return (
    <div style={styles.container}>
      {label && <div style={styles.label}>{label}</div>}
      <div style={styles.svgContainer} dangerouslySetInnerHTML={{ __html: svg }} />
      <form onSubmit={handleSubmit} style={styles.form}>
        <div style={styles.fields}>
          {values.map((val, i) => (
            <div key={i} style={styles.fieldRow}>
              {fieldLabels?.[i] && <span style={styles.fieldLabel}>{fieldLabels[i]}</span>}
              <input
                ref={(el) => { inputRefs.current[i] = el; }}
                type="text"
                value={val}
                onChange={(e) => handleChange(i, e.target.value)}
                onKeyDown={(e) => handleKeyDown(i, e)}
                placeholder={placeholders?.[i] || `Answer ${i + 1}`}
                disabled={disabled}
                style={styles.input}
                autoComplete="off"
              />
            </div>
          ))}
        </div>
        <button type="submit" disabled={disabled || !allFilled} style={styles.submitBtn}>
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
    fontSize: '13px',
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
    flexDirection: 'column',
    alignItems: 'center',
    gap: '12px',
  },
  fields: {
    display: 'flex',
    gap: '8px',
    flexWrap: 'wrap',
    justifyContent: 'center',
  },
  fieldRow: {
    display: 'flex',
    alignItems: 'center',
    gap: '4px',
  },
  fieldLabel: {
    color: '#666',
    fontSize: '12px',
    fontFamily: 'monospace',
  },
  input: {
    background: '#1a1a2e',
    border: '2px solid #2a2a4e',
    borderRadius: '8px',
    color: '#e0e0e0',
    fontSize: '18px',
    fontFamily: 'monospace',
    padding: '8px 12px',
    textAlign: 'center',
    width: '100px',
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
