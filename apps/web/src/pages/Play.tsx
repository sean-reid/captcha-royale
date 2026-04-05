import { useState, useCallback, useRef, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useCaptchaEngine } from '../hooks/useCaptchaEngine';
import { CaptchaRenderer } from '../components/captcha/CaptchaRenderer';
import { Timer } from '../components/match/Timer';
import { RoundIndicator } from '../components/match/RoundIndicator';
import { Button } from '../components/ui/Button';
import { CaptchaType } from '../types/captcha';
import type { CaptchaInstance, PlayerAnswer, ScoreResult } from '../types/captcha';
import { validateAnswer, scoreAnswer } from '../lib/wasm';
import { useAuth } from '../hooks/useAuth';

type GameState = 'menu' | 'playing' | 'gameover';

const TIER1_TYPES = [
  CaptchaType.DistortedText, CaptchaType.SimpleMath, CaptchaType.ImageGrid,
  CaptchaType.DotCount, CaptchaType.ClockReading, CaptchaType.FractionComparison,
  CaptchaType.GraphReading,
];
const TIER2_TYPES = [
  CaptchaType.RotatedObject, CaptchaType.ColorPerception, CaptchaType.SequenceCompletion,
  CaptchaType.SemanticOddity, CaptchaType.MirrorMatch, CaptchaType.BalanceScale,
  CaptchaType.WordUnscramble, CaptchaType.GradientOrder, CaptchaType.OverlapCounting,
  CaptchaType.RotationPrediction, CaptchaType.PartialOcclusion,
];
const TIER3_TYPES = [
  CaptchaType.MultiStepVerification, CaptchaType.AdversarialTypography,
  CaptchaType.PathTracing, CaptchaType.BooleanLogic, CaptchaType.AdversarialImage,
];
const TIER4_TYPES = [
  CaptchaType.SpatialReasoning, CaptchaType.MetamorphicCaptcha,
  CaptchaType.TimePressureCascade, CaptchaType.CombinedModality,
];

function getAvailableTypes(round: number): CaptchaType[] {
  if (round > 30) {
    return [...TIER1_TYPES, ...TIER2_TYPES, ...TIER3_TYPES, ...TIER4_TYPES];
  }
  if (round > 20) {
    return [...TIER1_TYPES, ...TIER2_TYPES, ...TIER3_TYPES];
  }
  if (round > 10) {
    return [...TIER1_TYPES, ...TIER2_TYPES];
  }
  return TIER1_TYPES;
}

export function Play() {
  const navigate = useNavigate();
  const { ready, error, generate } = useCaptchaEngine();
  const { player } = useAuth();
  const [gameState, setGameState] = useState<GameState>('menu');
  const [round, setRound] = useState(0);
  const [score, setScore] = useState(0);

  // Derive effective level from player ELO (ELO 1000 → level 50, 2000 → 100)
  // Same formula as match-room.ts — single source of truth is compute_difficulty in WASM
  const eloLevel = Math.min(Math.floor((player?.elo ?? 1000) / 20), 100);
  const [currentCaptcha, setCurrentCaptcha] = useState<CaptchaInstance | null>(null);
  const [feedback, setFeedback] = useState<{ correct: boolean; result: ScoreResult } | null>(null);
  const [highScore, setHighScore] = useState(() => {
    const stored = localStorage.getItem('captcha-royale-highscore');
    return stored ? parseInt(stored, 10) : 0;
  });
  const roundStartRef = useRef(0);
  const feedbackTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const generateNextCaptcha = useCallback(
    (roundNum: number) => {
      const seed = BigInt(Date.now()) * BigInt(1000) + BigInt(roundNum);
      const types = getAvailableTypes(roundNum);
      const captchaType = types[roundNum % types.length];
      const effectiveLevel = eloLevel + roundNum;
      const captcha = generate(seed, captchaType, effectiveLevel, roundNum);
      if (captcha) {
        setCurrentCaptcha(captcha);
        roundStartRef.current = Date.now();
        setFeedback(null);
      }
    },
    [generate, eloLevel],
  );

  const startGame = useCallback(() => {
    setGameState('playing');
    setRound(1);
    setScore(0);
    setFeedback(null);
    generateNextCaptcha(1);
  }, [generateNextCaptcha]);

  const endGame = useCallback(() => {
    setGameState('gameover');
    setCurrentCaptcha(null);
    if (score > highScore) {
      setHighScore(score);
      localStorage.setItem('captcha-royale-highscore', score.toString());
    }
  }, [score, highScore]);

  const handleSubmit = useCallback(
    (answer: PlayerAnswer) => {
      if (!currentCaptcha) return;

      const solveTime = Date.now() - roundStartRef.current;
      const correct = validateAnswer(currentCaptcha, answer);
      const result = scoreAnswer(currentCaptcha, answer, solveTime);

      setFeedback({ correct, result });

      if (correct) {
        const newScore = score + result.total_points;
        setScore(newScore);

        // Brief feedback then next round
        if (feedbackTimeoutRef.current) clearTimeout(feedbackTimeoutRef.current);
        feedbackTimeoutRef.current = setTimeout(() => {
          const nextRound = round + 1;
          setRound(nextRound);
          generateNextCaptcha(nextRound);
        }, 800);
      } else {
        // Wrong answer — game over after brief delay
        if (feedbackTimeoutRef.current) clearTimeout(feedbackTimeoutRef.current);
        feedbackTimeoutRef.current = setTimeout(endGame, 1200);
      }
    },
    [currentCaptcha, score, round, generateNextCaptcha, endGame],
  );

  const handleTimeout = useCallback(() => {
    setFeedback({ correct: false, result: { correct: false, base_points: 0, speed_bonus: 0, total_points: 0 } });
    if (feedbackTimeoutRef.current) clearTimeout(feedbackTimeoutRef.current);
    feedbackTimeoutRef.current = setTimeout(endGame, 1200);
  }, [endGame]);

  useEffect(() => {
    return () => {
      if (feedbackTimeoutRef.current) clearTimeout(feedbackTimeoutRef.current);
    };
  }, []);

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

  if (gameState === 'menu') {
    return (
      <div style={styles.center}>
        <h1 style={styles.title}>Endless Mode</h1>
        <p style={styles.subtitle}>Solve CAPTCHAs until you fail. How far can you go?</p>
        {highScore > 0 && <p style={styles.highScore}>High Score: {highScore}</p>}
        <Button size="lg" onClick={startGame}>
          Start
        </Button>
      </div>
    );
  }

  if (gameState === 'gameover') {
    return (
      <div style={styles.center}>
        <h1 style={{ ...styles.title, color: '#e74c3c' }}>Game Over</h1>
        <div style={styles.stats}>
          <div style={styles.stat}>
            <span style={styles.statLabel}>Rounds Survived</span>
            <span style={styles.statValue}>{round - 1}</span>
          </div>
          <div style={styles.stat}>
            <span style={styles.statLabel}>Score</span>
            <span style={styles.statValue}>{score}</span>
          </div>
          <div style={styles.stat}>
            <span style={styles.statLabel}>High Score</span>
            <span style={styles.statValue}>{highScore}</span>
          </div>
        </div>
        <div style={{ display: 'flex', gap: '12px' }}>
          <Button size="lg" onClick={startGame}>
            Play Again
          </Button>
          <Button size="lg" variant="secondary" onClick={() => navigate('/')}>
            Home
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div style={styles.gameContainer}>
      <div style={styles.hud}>
        <RoundIndicator round={round} captchaType={currentCaptcha?.captcha_type} />
        <div style={styles.scoreDisplay}>
          Score: <span style={{ color: '#ff6b6b' }}>{score}</span>
        </div>
      </div>

      {currentCaptcha && (
        <Timer
          durationMs={currentCaptcha.time_limit_ms}
          onTimeout={handleTimeout}
          running={gameState === 'playing' && !feedback}
        />
      )}

      {feedback && (
        <div
          style={{
            ...styles.feedback,
            color: feedback.correct ? '#2ecc71' : '#e74c3c',
            background: feedback.correct ? 'rgba(46,204,113,0.1)' : 'rgba(231,76,60,0.1)',
          }}
        >
          {feedback.correct
            ? `Correct! +${feedback.result.total_points} pts`
            : 'Wrong!'}
        </div>
      )}

      {currentCaptcha && (
        <CaptchaRenderer
          instance={currentCaptcha}
          onSubmit={handleSubmit}
          disabled={!!feedback}
        />
      )}
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  center: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    minHeight: '60vh',
    gap: '16px',
  },
  title: {
    color: '#e0e0e0',
    fontSize: '40px',
    fontFamily: 'monospace',
  },
  subtitle: {
    color: '#888',
    fontSize: '16px',
    marginBottom: '8px',
  },
  highScore: {
    color: '#ffd700',
    fontSize: '18px',
    fontFamily: 'monospace',
  },
  gameContainer: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    padding: '24px 32px',
    gap: '20px',
    maxWidth: '700px',
    margin: '0 auto',
  },
  hud: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    width: '100%',
  },
  scoreDisplay: {
    color: '#e0e0e0',
    fontSize: '20px',
    fontFamily: 'monospace',
    fontWeight: 'bold',
  },
  feedback: {
    padding: '8px 24px',
    borderRadius: '8px',
    fontWeight: 'bold',
    fontSize: '16px',
    textAlign: 'center',
  },
  stats: {
    display: 'flex',
    gap: '32px',
    marginBottom: '16px',
  },
  stat: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    gap: '4px',
  },
  statLabel: {
    color: '#888',
    fontSize: '12px',
    textTransform: 'uppercase',
  },
  statValue: {
    color: '#e0e0e0',
    fontSize: '32px',
    fontFamily: 'monospace',
    fontWeight: 'bold',
  },
};
