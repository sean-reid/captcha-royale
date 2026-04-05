import type { CaptchaInstance, PlayerAnswer } from '../../types/captcha';
import { TextCaptcha } from './TextCaptcha';
import { MathCaptcha } from './MathCaptcha';
import { GridCaptcha } from './GridCaptcha';
import { SvgClickCaptcha } from './SvgClickCaptcha';
import { SvgTextCaptcha } from './SvgTextCaptcha';
import { SvgMultiTextCaptcha } from './SvgMultiTextCaptcha';
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

    // Tier 1 — click-based
    case CaptchaType.FractionComparison:
    case CaptchaType.GraphReading:
      return <SvgClickCaptcha instance={instance} onSubmit={onSubmit} disabled={disabled} />;

    // Tier 1 — number input (wrap text→number for validation)
    case CaptchaType.DotCount:
      return <SvgTextCaptcha instance={instance} onSubmit={(a) => { if ('Text' in a) onSubmit({ Number: parseFloat(a.Text) }); else onSubmit(a); }} disabled={disabled} label="Count the dots" placeholder="How many dots?" />;
    case CaptchaType.ClockReading:
      return <SvgTextCaptcha instance={instance} onSubmit={onSubmit} disabled={disabled} label="Read the clock" placeholder="H:MM (e.g. 3:30)" />;

    // Tier 2 — click-based
    case CaptchaType.RotatedObject:
    case CaptchaType.SequenceCompletion:
    case CaptchaType.SemanticOddity:
    case CaptchaType.MirrorMatch:
    case CaptchaType.BalanceScale:
    case CaptchaType.GradientOrder:
    case CaptchaType.RotationPrediction:
    case CaptchaType.PartialOcclusion:
      return <SvgClickCaptcha instance={instance} onSubmit={onSubmit} disabled={disabled} />;

    // Tier 2 — text input
    case CaptchaType.WordUnscramble:
      return <SvgTextCaptcha instance={instance} onSubmit={onSubmit} disabled={disabled} label="Unscramble the word" placeholder="Type the word..." />;
    case CaptchaType.OverlapCounting:
      return <SvgTextCaptcha instance={instance} onSubmit={(a) => { if ('Text' in a) onSubmit({ Number: parseFloat(a.Text) }); else onSubmit(a); }} disabled={disabled} label="Count the shapes" placeholder="How many shapes?" />;

    // Tier 2 — color uses special highlight
    case CaptchaType.ColorPerception:
      return <ColorCaptcha instance={instance} onSubmit={onSubmit} disabled={disabled} />;

    // Tier 3 — click-based
    case CaptchaType.SpatialReasoning:
    case CaptchaType.PathTracing:
    case CaptchaType.AdversarialImage:
    case CaptchaType.CombinedModality:
    case CaptchaType.ProceduralNovelType:
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
    case CaptchaType.BooleanLogic:
      return <SvgTextCaptcha instance={instance} onSubmit={(a) => { if ('Text' in a) onSubmit({ Number: parseFloat(a.Text) }); else onSubmit(a); }} disabled={disabled} label="Compute the output" placeholder="0 or 1" />;

    // Tier 3 — multi-input
    case CaptchaType.MultiStepVerification: {
      const msSvg = 'Svg' in instance.render_data ? instance.render_data.Svg : '';
      const msMatch = msSvg.match(/data-challenge-count="(\d+)"/);
      const msCount = msMatch ? parseInt(msMatch[1], 10) : 2;
      return (
        <SvgMultiTextCaptcha
          instance={instance}
          onSubmit={onSubmit}
          disabled={disabled}
          label="Solve each challenge"
          fieldCount={msCount}
        />
      );
    }

    // Tier 4 — click-based
    case CaptchaType.MetamorphicCaptcha:
      return <SvgClickCaptcha instance={instance} onSubmit={onSubmit} disabled={disabled} />;

    // Tier 4 — multi-input
    case CaptchaType.TimePressureCascade: {
      const tcSvg = 'Svg' in instance.render_data ? instance.render_data.Svg : '';
      const tcMatch = tcSvg.match(/data-challenge-count="(\d+)"/);
      const tcCount = tcMatch ? parseInt(tcMatch[1], 10) : 4;
      return (
        <SvgMultiTextCaptcha
          instance={instance}
          onSubmit={onSubmit}
          disabled={disabled}
          label="Read each number top to bottom"
          fieldCount={tcCount}
          placeholders={Array(tcCount).fill('').map((_, i) => `#${i + 1}`)}
        />
      );
    }

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
