import type { CaptchaInstance, PlayerAnswer } from '../../types/captcha';
import { TextCaptcha } from './TextCaptcha';
import { MathCaptcha } from './MathCaptcha';
import { GridCaptcha } from './GridCaptcha';
import { SvgClickCaptcha } from './SvgClickCaptcha';
import { SvgTextCaptcha } from './SvgTextCaptcha';
import { ColorCaptcha } from './ColorCaptcha';
import { CaptchaType } from '../../types/captcha';

interface CaptchaRendererProps {
  instance: CaptchaInstance;
  onSubmit: (answer: PlayerAnswer) => void;
  disabled?: boolean;
}

export function CaptchaRenderer({ instance, onSubmit, disabled }: CaptchaRendererProps) {
  switch (instance.captcha_type) {
    // Tier 1 — text input
    case CaptchaType.DistortedText:
      return <TextCaptcha instance={instance} onSubmit={onSubmit} disabled={disabled} />;
    case CaptchaType.SimpleMath:
      return <MathCaptcha instance={instance} onSubmit={onSubmit} disabled={disabled} />;
    case CaptchaType.ImageGrid:
      return <GridCaptcha instance={instance} onSubmit={onSubmit} disabled={disabled} />;

    // Tier 2 — click-based
    case CaptchaType.RotatedObject:
    case CaptchaType.SequenceCompletion:
    case CaptchaType.SemanticOddity:
      return <SvgClickCaptcha instance={instance} onSubmit={onSubmit} disabled={disabled} />;

    // Tier 2 — color uses special highlight
    case CaptchaType.ColorPerception:
      return <ColorCaptcha instance={instance} onSubmit={onSubmit} disabled={disabled} />;

    // Tier 3 — click-based
    case CaptchaType.SpatialReasoning:
      return <SvgClickCaptcha instance={instance} onSubmit={onSubmit} disabled={disabled} />;

    // Tier 3 — text input
    case CaptchaType.AdversarialTypography:
      return (
        <SvgTextCaptcha
          instance={instance}
          onSubmit={onSubmit}
          disabled={disabled}
          label="Read the adversarial text"
          placeholder="Type what you see..."
        />
      );
    case CaptchaType.MultiStepVerification:
      return (
        <SvgTextCaptcha
          instance={instance}
          onSubmit={onSubmit}
          disabled={disabled}
          label="Solve all challenges (separate answers with |)"
          placeholder="answer1|answer2|answer3"
        />
      );

    // Tier 4 — click-based
    case CaptchaType.MetamorphicCaptcha:
      return <SvgClickCaptcha instance={instance} onSubmit={onSubmit} disabled={disabled} />;

    // Tier 4 — text input
    case CaptchaType.TimePressureCascade:
      return (
        <SvgTextCaptcha
          instance={instance}
          onSubmit={onSubmit}
          disabled={disabled}
          label="Read all numbers top to bottom (separate with |)"
          placeholder="12|7|45|3..."
        />
      );

    default:
      // Auto-detect: if SVG has data-index, use click; otherwise text input
      if ('Svg' in instance.render_data) {
        const svg = instance.render_data.Svg;
        if (svg.includes('data-index')) {
          return <SvgClickCaptcha instance={instance} onSubmit={onSubmit} disabled={disabled} />;
        }
        return <SvgTextCaptcha instance={instance} onSubmit={onSubmit} disabled={disabled} />;
      }
      if ('Grid' in instance.render_data) {
        return <GridCaptcha instance={instance} onSubmit={onSubmit} disabled={disabled} />;
      }
      return (
        <div style={{ color: '#ff6b6b', textAlign: 'center', padding: '40px' }}>
          CAPTCHA type not yet implemented: {instance.captcha_type}
        </div>
      );
  }
}
