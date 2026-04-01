import { useState, useEffect } from 'react';
import type { CaptchaInstance, PlayerAnswer } from '../../types/captcha';

interface GridCaptchaProps {
  instance: CaptchaInstance;
  onSubmit: (answer: PlayerAnswer) => void;
  disabled?: boolean;
}

export function GridCaptcha({ instance, onSubmit, disabled }: GridCaptchaProps) {
  const [selected, setSelected] = useState<Set<number>>(new Set());

  useEffect(() => {
    setSelected(new Set());
  }, [instance]);

  if (!('Grid' in instance.render_data)) return null;

  const { cols, cells, prompt } = instance.render_data.Grid;

  const toggleCell = (index: number) => {
    if (disabled) return;
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(index)) {
        next.delete(index);
      } else {
        next.add(index);
      }
      return next;
    });
  };

  const handleSubmit = () => {
    if (selected.size > 0 && !disabled) {
      onSubmit({ SelectedIndices: Array.from(selected).sort((a, b) => a - b) });
    }
  };

  const cellSize = cols <= 2 ? 140 : cols <= 3 ? 110 : 90;

  return (
    <div style={styles.container}>
      <div style={styles.prompt}>{prompt}</div>
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: `repeat(${cols}, ${cellSize}px)`,
          gap: '8px',
        }}
      >
        {cells.map((cell) => (
          <div
            key={cell.index}
            onClick={() => toggleCell(cell.index)}
            style={{
              ...styles.cell,
              width: cellSize,
              height: cellSize,
              border: selected.has(cell.index)
                ? '3px solid #ff6b6b'
                : '3px solid #2a2a4e',
              cursor: disabled ? 'default' : 'pointer',
              opacity: disabled ? 0.6 : 1,
            }}
            dangerouslySetInnerHTML={{
              __html: cell.svg
                .replace(/width="\d+"/, 'width="100%"')
                .replace(/height="\d+"/, 'height="100%"'),
            }}
          />
        ))}
      </div>
      <button
        onClick={handleSubmit}
        disabled={disabled || selected.size === 0}
        style={styles.submitBtn}
      >
        Submit ({selected.size} selected)
      </button>
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
  prompt: {
    color: '#e0e0e0',
    fontSize: '16px',
    fontWeight: 'bold',
    textAlign: 'center',
  },
  cell: {
    borderRadius: '8px',
    overflow: 'hidden',
    transition: 'border-color 0.15s',
  },
  submitBtn: {
    background: 'linear-gradient(135deg, #ff6b6b, #ee5a24)',
    border: 'none',
    borderRadius: '8px',
    color: '#fff',
    fontSize: '16px',
    fontWeight: 'bold',
    padding: '10px 32px',
    cursor: 'pointer',
  },
};
