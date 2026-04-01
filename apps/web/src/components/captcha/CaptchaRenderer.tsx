import type { CaptchaInstance, PlayerAnswer } from '../../types/captcha';
import { TextCaptcha } from './TextCaptcha';
import { MathCaptcha } from './MathCaptcha';
import { GridCaptcha } from './GridCaptcha';
import { RotationCaptcha } from './RotationCaptcha';
import { ColorCaptcha } from './ColorCaptcha';
import { SequenceCaptcha } from './SequenceCaptcha';
import { CaptchaType } from '../../types/captcha';

interface CaptchaRendererProps {
  instance: CaptchaInstance;
  onSubmit: (answer: PlayerAnswer) => void;
  disabled?: boolean;
}

export function CaptchaRenderer({ instance, onSubmit, disabled }: CaptchaRendererProps) {
  switch (instance.captcha_type) {
    case CaptchaType.DistortedText:
      return <TextCaptcha instance={instance} onSubmit={onSubmit} disabled={disabled} />;
    case CaptchaType.SimpleMath:
      return <MathCaptcha instance={instance} onSubmit={onSubmit} disabled={disabled} />;
    case CaptchaType.ImageGrid:
      return <GridCaptcha instance={instance} onSubmit={onSubmit} disabled={disabled} />;
    case CaptchaType.RotatedObject:
      return <RotationCaptcha instance={instance} onSubmit={onSubmit} disabled={disabled} />;
    case CaptchaType.ColorPerception:
      return <ColorCaptcha instance={instance} onSubmit={onSubmit} disabled={disabled} />;
    case CaptchaType.SequenceCompletion:
      return <SequenceCaptcha instance={instance} onSubmit={onSubmit} disabled={disabled} />;
    default:
      // Fall back to grid-like interface for grid-based types, text input for others
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
