import { useState, useEffect, useCallback } from 'react';
import { initWasm, generateCaptcha, computeDifficulty } from '../lib/wasm';
import type { CaptchaInstance, CaptchaType } from '../types/captcha';

export function useCaptchaEngine() {
  const [ready, setReady] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    initWasm()
      .then(() => setReady(true))
      .catch((err) => setError(err.message));
  }, []);

  const generate = useCallback(
    (seed: bigint, captchaType: CaptchaType, level: number, roundNumber: number): CaptchaInstance | null => {
      if (!ready) return null;
      const difficulty = computeDifficulty(captchaType, level, roundNumber);
      return generateCaptcha(seed, captchaType, difficulty);
    },
    [ready],
  );

  return { ready, error, generate };
}
