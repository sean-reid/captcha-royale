# CAPTCHA Royale

Competitive real-time multiplayer browser game where players race to solve procedurally generated CAPTCHAs. Up to 16 players enter a room, CAPTCHAs get progressively harder, and the last player standing wins.

## How It Works

- **Seed-based generation** — the server sends a seed and difficulty parameters; both client and server WASM modules generate identical CAPTCHAs deterministically
- **Server-side validation** — answers are validated by the WASM engine running inside Cloudflare Durable Objects; the client is never trusted
- **ELO matchmaking** — players are grouped into brackets (Bronze through Diamond) with automatic bracket expansion for long waits
- **Desktop only** — keyboard and mouse input, minimum 1024px window width

## Tech Stack

| Layer | Technology |
|---|---|
| Frontend | React 19 + TypeScript + Vite |
| CAPTCHA Engine | Rust compiled to WebAssembly (~230KB) |
| Realtime Backend | Cloudflare Workers + Durable Objects |
| Database | Cloudflare D1 (SQLite at edge) |
| Cache | Cloudflare KV |
| Auth | Google OAuth2 (Discord/GitHub ready) |
| Frontend Hosting | GitHub Pages |
| Monorepo | Turborepo + pnpm workspaces |

## Project Structure

```
captcha-royale/
├── packages/
│   └── captcha-engine/          # Rust WASM crate
│       └── src/
│           ├── lib.rs           # WASM entry points
│           ├── types.rs         # CaptchaInstance, Solution, PlayerAnswer
│           ├── rng.rs           # HMAC-SHA256 seed derivation, ChaCha8 RNG
│           ├── difficulty.rs    # Level/round -> difficulty params
│           └── generators/      # One per CAPTCHA type
│               ├── text.rs      # Distorted text with multi-layer noise
│               ├── math.rs      # Math expressions with visual disruption
│               ├── grid.rs      # "Select all squares with X" shape grids
│               ├── rotation.rs  # Find the correctly oriented object
│               ├── color.rs     # Ishihara-style color perception
│               └── sequence.rs  # Visual pattern completion
├── apps/
│   ├── web/                     # React SPA
│   │   └── src/
│   │       ├── pages/           # Home, Play (Endless), Queue, Match, Results, Profile, Leaderboard, Login
│   │       ├── components/
│   │       │   ├── captcha/     # CaptchaRenderer + type-specific renderers
│   │       │   ├── match/       # Timer, PlayerList, EliminationFeed, RoundIndicator
│   │       │   ├── ui/          # Button, Modal
│   │       │   └── layout/      # Header, Footer
│   │       ├── hooks/           # useAuth, useCaptchaEngine, useWebSocket, useMatchState
│   │       └── lib/             # WASM bindings, API client, config, ELO helpers
│   └── worker/                  # Cloudflare Workers
│       └── src/
│           ├── index.ts         # Router
│           ├── auth.ts          # OAuth2 flow
│           ├── match-room.ts    # MatchRoom Durable Object (game loop)
│           ├── matchmaker.ts    # Matchmaker Durable Object (queue + brackets)
│           ├── api/             # Profile, leaderboard, match endpoints
│           └── lib/             # D1 helpers, session, ELO, progression, achievements
└── .github/workflows/           # CI + GitHub Pages deploy
```

## CAPTCHA Types

**Tier 1 — Foundations** (always available)
- Distorted Text — warped characters with bezier noise, decoys, and overlapping strokes
- Simple Math — arithmetic rendered with visual disruption and decoy digits
- Image Grid — select all cells containing a target shape

**Tier 2 — Perceptual** (unlocked after round 10 in Endless, level 11+ in multiplayer)
- Rotated Object — find the correctly oriented object among rotated variants
- Color Perception — Ishihara-inspired grid, find the differently shaded tile
- Sequence Completion — identify the next item in a visual pattern

## Game Modes

- **Endless** (solo) — solve CAPTCHAs until you fail, track your high score. No backend needed.
- **Battle Royale** (multiplayer) — 4-16 players, wrong answer or timeout = elimination, last standing wins
- **Sprint** (multiplayer) — 2-8 players, solve 10 CAPTCHAs as fast as possible, no elimination, highest score wins

## Development

### Prerequisites

- Node.js 22+
- pnpm 10+
- Rust with `wasm32-unknown-unknown` target
- wasm-pack

### Setup

```bash
# Install dependencies
pnpm install

# Build the WASM engine
cd packages/captcha-engine
wasm-pack build --target web --out-dir pkg

# Run frontend dev server
cd apps/web
pnpm dev
```

### Running Tests

```bash
# Rust engine tests (35 tests — determinism, validation, difficulty scaling)
cd packages/captcha-engine
cargo test

# Clippy lint
cargo clippy -- -D warnings

# Frontend type check
cd apps/web
npx tsc --noEmit

# Worker type check
cd apps/worker
npx tsc --noEmit
```

## Deployment

### Frontend (GitHub Pages)

Automatically deploys on push to `main` via GitHub Actions. Or manually:

```bash
cd apps/web
VITE_API_URL=https://your-worker.workers.dev npx vite build
cp dist/index.html dist/404.html  # SPA fallback
```

### Backend (Cloudflare Workers)

```bash
cd apps/worker

# First time: apply D1 migration
npx wrangler d1 migrations apply captcha-royale-db --remote

# Set secrets
npx wrangler secret put GOOGLE_CLIENT_ID
npx wrangler secret put GOOGLE_CLIENT_SECRET
npx wrangler secret put FRONTEND_URL  # https://yourusername.github.io/captcha-royale

# Deploy
npx wrangler deploy
```

### Environment Variables

| Variable | Where | Purpose |
|---|---|---|
| `VITE_API_URL` | Frontend build | Worker URL for API/WebSocket calls |
| `GOOGLE_CLIENT_ID` | Worker secret | Google OAuth |
| `GOOGLE_CLIENT_SECRET` | Worker secret | Google OAuth |
| `FRONTEND_URL` | Worker secret | OAuth redirect destination |
| `BASE_URL` | Worker secret | OAuth callback URL base (defaults to request origin) |

## Scoring

```
base_points = captcha_tier * 10          (T1=10, T2=20, T3=30, T4=40)
speed_bonus = max(0, (time_limit - solve_time) / time_limit) * base_points
total       = base_points + speed_bonus  (range: base to 2x base)
```

## ELO System

- New players start at 1000
- K-factor: 40 (< 30 matches), 24 (30-100), 16 (100+)
- Brackets: Bronze (< 800), Silver (800-1000), Gold (1000-1200), Platinum (1200-1500), Diamond (1500+)
- Matchmaker expands search to adjacent brackets after 30s, +/-2 after 60s
