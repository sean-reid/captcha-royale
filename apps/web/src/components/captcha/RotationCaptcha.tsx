import { useState, useEffect, useCallback } from 'react';
import type { CaptchaInstance, PlayerAnswer } from '../../types/captcha';

interface RotationCaptchaProps {
  instance: CaptchaInstance;
  onSubmit: (answer: PlayerAnswer) => void;
  disabled?: boolean;
}

export function RotationCaptcha({ instance, onSubmit, disabled }: RotationCaptchaProps) {
  const [selected, setSelected] = useState<number | null>(null);

  useEffect(() => {
    setSelected(null);
  }, [instance]);

  const svg = 'Svg' in instance.render_data ? instance.render_data.Svg : '';

  const handleClick = useCallback(
    (e: React.MouseEvent) => {
      if (disabled) return;
      const target = e.target as SVGElement;
      const indexAttr = target.getAttribute?.('data-index');
      if (indexAttr !== null) {
        setSelected(parseInt(indexAttr, 10));
      }
    },
    [disabled],
  );

  const handleSubmit = () => {
    if (selected !== null && !disabled) {
      onSubmit({ SelectedIndices: [selected] });
    }
  };

  // Highlight selected cell
  const styledSvg = selected !== null
    ? svg.replace(
        `data-index="${selected}"`,
        `data-index="${selected}" stroke="#ff6b6b" stroke-width="3"`,
      )
    : svg;

  return (
    <div style={styles.container}>
      <div
        style={styles.svgContainer}
        onClick={handleClick}
        dangerouslySetInnerHTML={{ __html: styledSvg }}
      />
      {selected !== null && (
        <button onClick={handleSubmit} disabled={disabled} style={styles.submitBtn}>
          Submit
        </button>
      )}
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
  svgContainer: {
    borderRadius: '8px',
    overflow: 'hidden',
    border: '2px solid #2a2a4e',
    cursor: 'pointer',
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
