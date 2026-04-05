import { useState, useCallback, useRef, useEffect } from 'react';
import { useCaptchaEngine } from '../hooks/useCaptchaEngine';
import { CaptchaRenderer } from '../components/captcha/CaptchaRenderer';
import { Timer } from '../components/match/Timer';
import { Button } from '../components/ui/Button';
import { CaptchaType } from '../types/captcha';
import type { CaptchaInstance, PlayerAnswer, ScoreResult } from '../types/captcha';
import { generateCaptcha, validateAnswer, scoreAnswer, computeDifficulty } from '../lib/wasm';

const ALL_TYPES: { type: CaptchaType; label: string; tier: number }[] = [
  // Tier 1 — Foundations
  { type: CaptchaType.DistortedText, label: 'Distorted Text', tier: 1 },
  { type: CaptchaType.SimpleMath, label: 'Simple Math', tier: 1 },
  { type: CaptchaType.ImageGrid, label: 'Image Grid', tier: 1 },
  { type: CaptchaType.DotCount, label: 'Dot Count', tier: 1 },
  { type: CaptchaType.ClockReading, label: 'Clock Reading', tier: 1 },
  { type: CaptchaType.FractionComparison, label: 'Fraction Comparison', tier: 1 },
  { type: CaptchaType.GraphReading, label: 'Graph Reading', tier: 1 },
  // Tier 2 — Perceptual
  { type: CaptchaType.RotatedObject, label: 'Rotated Object', tier: 2 },
  { type: CaptchaType.ColorPerception, label: 'Color Perception', tier: 2 },
  { type: CaptchaType.SequenceCompletion, label: 'Sequence Completion', tier: 2 },
  { type: CaptchaType.SemanticOddity, label: 'Semantic Oddity', tier: 2 },
  { type: CaptchaType.MirrorMatch, label: 'Mirror Match', tier: 2 },
  { type: CaptchaType.BalanceScale, label: 'Balance Scale', tier: 2 },
  { type: CaptchaType.WordUnscramble, label: 'Word Unscramble', tier: 2 },
  { type: CaptchaType.GradientOrder, label: 'Gradient Order', tier: 2 },
  { type: CaptchaType.OverlapCounting, label: 'Overlap Counting', tier: 2 },
  { type: CaptchaType.RotationPrediction, label: 'Gear Rotation', tier: 2 },
  { type: CaptchaType.PartialOcclusion, label: 'Jigsaw Fit', tier: 2 },
  // Tier 3 — Cognitive
  { type: CaptchaType.MultiStepVerification, label: 'Multi-step', tier: 3 },
  { type: CaptchaType.AdversarialTypography, label: 'Adversarial Typography', tier: 3 },
  { type: CaptchaType.PathTracing, label: 'Path Tracing', tier: 3 },
  { type: CaptchaType.BooleanLogic, label: 'Boolean Logic', tier: 3 },
  { type: CaptchaType.AdversarialImage, label: 'Shadow Matching', tier: 3 },
  // Tier 4 — Nightmare
  { type: CaptchaType.SpatialReasoning, label: 'Spatial Reasoning', tier: 4 },
  { type: CaptchaType.MetamorphicCaptcha, label: 'Metamorphic', tier: 4 },
  { type: CaptchaType.TimePressureCascade, label: 'Time Pressure Cascade', tier: 4 },
  { type: CaptchaType.CombinedModality, label: 'Matrix Pattern', tier: 4 },
];

const TIER_COLORS: Record<number, string> = {
  1: '#2ecc71',
  2: '#3498db',
  3: '#f39c12',
  4: '#e74c3c',
};

export function Playtest() {
  const { ready, error } = useCaptchaEngine();
  const [selectedType, setSelectedType] = useState<CaptchaType | null>(null);
  const [complexity, setComplexity] = useState(0.5);
  const [currentCaptcha, setCurrentCaptcha] = useState<CaptchaInstance | null>(null);
  const [feedback, setFeedback] = useState<{ correct: boolean; result: ScoreResult } | null>(null);
  const [seed, setSeed] = useState(BigInt(Date.now()));
  const [captchaKey, setCaptchaKey] = useState(0);
  const roundStartRef = useRef(0);

  const generate = useCallback(() => {
    if (!selectedType || !ready) return;
    const newSeed = BigInt(Date.now());
    setSeed(newSeed);
    const difficulty = computeDifficulty(selectedType, Math.floor(complexity * 100), 1);
    // Override complexity/noise with slider values
    difficulty.complexity = complexity;
    difficulty.noise = complexity * 0.8;
    try {
      const instance = generateCaptcha(newSeed, selectedType, difficulty);
      setCurrentCaptcha(instance);
      setFeedback(null);
      setCaptchaKey((k) => k + 1);
      roundStartRef.current = Date.now();
    } catch (err) {
      console.error('Generation failed:', err);
    }
  }, [selectedType, complexity, ready]);

  // Auto-generate when type or complexity changes
  useEffect(() => {
    if (selectedType && ready) generate();
  }, [selectedType, complexity, ready]);

  const handleSubmit = useCallback(
    (answer: PlayerAnswer) => {
      if (!currentCaptcha) return;
      const solveTime = Date.now() - roundStartRef.current;
      const correct = validateAnswer(currentCaptcha, answer);
      const result = scoreAnswer(currentCaptcha, answer, solveTime);
      setFeedback({ correct, result });
    },
    [currentCaptcha],
  );

  if (!ready) {
    return (
      <div style={styles.center}>
        {error ? (
          <p style={{ color: '#e74c3c' }}>Failed to load WASM: {error}</p>
        ) : (
          <p style={{ color: '#888' }}>Loading CAPTCHA engine...</p>
        )}
      </div>
    );
  }

  return (
    <div style={styles.layout}>
      <div style={styles.sidebar}>
        <h2 style={styles.sidebarTitle}>Playtest Mode</h2>

        <div style={styles.section}>
          <label style={styles.label}>Complexity: {complexity.toFixed(2)}</label>
          <input
            type="range"
            min="0"
            max="1"
            step="0.05"
            value={complexity}
            onChange={(e) => setComplexity(parseFloat(e.target.value))}
            style={styles.slider}
          />
          <div style={styles.sliderLabels}>
            <span>Easy</span>
            <span>Hard</span>
          </div>
        </div>

        <div style={styles.section}>
          <label style={styles.label}>CAPTCHA Type</label>
          {[1, 2, 3, 4].map((tier) => (
            <div key={tier}>
              <div style={{ ...styles.tierHeader, color: TIER_COLORS[tier] }}>
                Tier {tier}
              </div>
              {ALL_TYPES.filter((t) => t.tier === tier).map((t) => (
                <button
                  key={t.type}
                  onClick={() => setSelectedType(t.type)}
                  style={{
                    ...styles.typeBtn,
                    background: selectedType === t.type ? '#2a2a4e' : 'transparent',
                    borderColor: selectedType === t.type ? TIER_COLORS[tier] : '#1a1a3e',
                  }}
                >
                  {t.label}
                </button>
              ))}
            </div>
          ))}
        </div>

        {selectedType && (
          <Button onClick={generate} size="sm" style={{ width: '100%' }}>
            Regenerate (New Seed)
          </Button>
        )}

        <div style={styles.info}>
          <p>Seed: {seed.toString().slice(-8)}</p>
          {currentCaptcha && (
            <p>Time limit: {(currentCaptcha.time_limit_ms / 1000).toFixed(1)}s</p>
          )}
        </div>
      </div>

      <div style={styles.main}>
        {!selectedType ? (
          <div style={styles.center}>
            <p style={{ color: '#888', fontSize: '18px' }}>
              Select a CAPTCHA type from the sidebar
            </p>
          </div>
        ) : (
          <div style={styles.captchaArea}>
            {currentCaptcha && (
              <>
                <Timer
                  key={captchaKey}
                  durationMs={currentCaptcha.time_limit_ms}
                  onTimeout={() => setFeedback({
                    correct: false,
                    result: { correct: false, base_points: 0, speed_bonus: 0, total_points: 0 },
                  })}
                  running={!feedback}
                />

                {feedback && (
                  <div
                    style={{
                      ...styles.feedback,
                      color: feedback.correct ? '#2ecc71' : '#e74c3c',
                      background: feedback.correct ? 'rgba(46,204,113,0.1)' : 'rgba(231,76,60,0.1)',
                    }}
                  >
                    {feedback.correct
                      ? `Correct! +${feedback.result.total_points} pts (${feedback.result.base_points} base + ${feedback.result.speed_bonus} speed)`
                      : 'Wrong!'}
                    <Button
                      size="sm"
                      variant="secondary"
                      onClick={generate}
                      style={{ marginLeft: '16px' }}
                    >
                      Next
                    </Button>
                  </div>
                )}

                <CaptchaRenderer
                  instance={currentCaptcha}
                  onSubmit={handleSubmit}
                  disabled={!!feedback}
                />
              </>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  layout: {
    display: 'flex',
    minHeight: 'calc(100vh - 120px)',
  },
  sidebar: {
    width: '260px',
    background: '#16213e',
    borderRight: '1px solid #2a2a4e',
    padding: '20px',
    display: 'flex',
    flexDirection: 'column',
    gap: '16px',
    overflowY: 'auto',
  },
  sidebarTitle: {
    color: '#ff6b6b',
    fontSize: '18px',
    fontFamily: 'monospace',
    margin: 0,
  },
  section: {
    display: 'flex',
    flexDirection: 'column',
    gap: '6px',
  },
  label: {
    color: '#888',
    fontSize: '11px',
    textTransform: 'uppercase',
    letterSpacing: '1px',
  },
  slider: {
    width: '100%',
    accentColor: '#ff6b6b',
  },
  sliderLabels: {
    display: 'flex',
    justifyContent: 'space-between',
    color: '#555',
    fontSize: '10px',
  },
  tierHeader: {
    fontSize: '11px',
    fontWeight: 'bold',
    textTransform: 'uppercase',
    letterSpacing: '1px',
    marginTop: '8px',
    marginBottom: '4px',
  },
  typeBtn: {
    display: 'block',
    width: '100%',
    textAlign: 'left',
    padding: '6px 10px',
    border: '1px solid #1a1a3e',
    borderRadius: '4px',
    background: 'transparent',
    color: '#e0e0e0',
    fontSize: '13px',
    cursor: 'pointer',
    marginBottom: '2px',
    fontFamily: 'inherit',
  },
  info: {
    color: '#555',
    fontSize: '11px',
    fontFamily: 'monospace',
    marginTop: 'auto',
  },
  main: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    padding: '24px',
  },
  center: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    flex: 1,
  },
  captchaArea: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    gap: '16px',
    maxWidth: '700px',
    width: '100%',
  },
  feedback: {
    padding: '8px 24px',
    borderRadius: '8px',
    fontWeight: 'bold',
    fontSize: '16px',
    display: 'flex',
    alignItems: 'center',
  },
};
