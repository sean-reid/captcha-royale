import { useState, useEffect, useCallback } from 'react';
import type { CaptchaInstance, PlayerAnswer } from '../../types/captcha';

interface SvgClickCaptchaProps {
  instance: CaptchaInstance;
  onSubmit: (answer: PlayerAnswer) => void;
  disabled?: boolean;
}

/**
 * Generic renderer for SVG-based CAPTCHAs where the player clicks on an option.
 * Expects the SVG to contain transparent rects with data-index attributes.
 * Used by: RotatedObject, ColorPerception, SequenceCompletion, SpatialReasoning,
 *          SemanticOddity, MetamorphicCaptcha
 */
export function SvgClickCaptcha({ instance, onSubmit, disabled }: SvgClickCaptchaProps) {
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

  // Highlight the selected element by wrapping it in a styled group
  const styledSvg = (() => {
    if (selected === null) return svg;
    const attr = `data-index="${selected}"`;
    // Find the element with this data-index and add a highlight outline after it
    const idx = svg.indexOf(attr);
    if (idx === -1) return svg;
    // Find the closing /> or > of this element
    const closeIdx = svg.indexOf('/>', idx);
    if (closeIdx === -1) return svg;
    const afterClose = closeIdx + 2;
    // Extract the element to read its position (look for x/cx and y/cy)
    const elemStr = svg.substring(svg.lastIndexOf('<', idx), afterClose);
    // Clone the rect with highlight styling
    const highlight = elemStr
      .replace(/fill="[^"]*"/, 'fill="rgba(255,107,107,0.2)"')
      .replace(/stroke="[^"]*"/, 'stroke="#ff6b6b"')
      .replace(/stroke-width="[^"]*"/, 'stroke-width="3"')
      .replace(/data-index="[^"]*"/, '')
      .replace(/style="[^"]*"/, '');
    return svg.substring(0, afterClose) + highlight + svg.substring(afterClose);
  })();

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
