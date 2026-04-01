# CAPTCHA Royale — Game Design & Implementation Document

## 1. Vision

CAPTCHA Royale is a competitive, real-time multiplayer browser game where players race to solve procedurally generated CAPTCHAs. Up to 16+ players enter a room, CAPTCHAs get progressively harder as your rank increases, and the last player standing — or the fastest solver — wins. Think WarioWare meets competitive typing meets a Turing test fever dream.

**Core pillars:**

- **Speed under pressure**: Every CAPTCHA is timed. Hesitate and you're eliminated.
- **Procedural variety**: No two runs are the same. CAPTCHA generators are parameterized, not hand-authored.
- **Escalating absurdity**: Low-rank CAPTCHAs are straightforward. High-rank CAPTCHAs exploit the boundary between human perception and machine vision.
- **Competitive depth**: ELO-based matchmaking ensures fair lobbies. Progression unlocks harder (and weirder) CAPTCHA tiers.
- **Desktop only**: Keyboard and mouse input. No mobile, no touch. This simplifies CAPTCHA interaction design and eliminates input-method timing disparities.

---

## 2. Tech Stack

| Layer | Technology | Rationale |
|---|---|---|
| **Frontend** | React + TypeScript + Vite | Fast iteration, large ecosystem, SSR not needed for a game |
| **CAPTCHA Engine** | Rust → WebAssembly | Procedural generation is CPU-bound; WASM gives near-native speed and makes reverse-engineering the generation logic harder for bot authors |
| **Realtime Backend** | Cloudflare Workers + Durable Objects | Free tier supports WebSocket connections, per-room state isolation, global edge deployment |
| **Persistent Storage** | Cloudflare D1 (SQLite at edge) | Free tier: 5 GB storage, 5M reads/day, 100K writes/day — more than sufficient for player profiles and match history |
| **KV Cache** | Cloudflare KV | Matchmaking queue, session tokens, leaderboard snapshots. Free tier: 100K reads/day, 1K writes/day |
| **Auth** | Custom OAuth flow on Workers | Google / Discord / GitHub OAuth2 — no NextAuth needed, Workers handle the redirect flow directly |
| **Static Hosting** | Cloudflare Pages | Free, auto-deploys from Git, serves the React SPA + WASM bundle |
| **Monorepo** | Turborepo or Nx | Manages `packages/captcha-engine` (Rust/WASM), `apps/web` (React), `apps/worker` (CF Workers) |

### Cloudflare Free Tier Limits (as of 2025)

| Resource | Free Limit |
|---|---|
| Workers requests | 100,000 / day |
| Durable Object requests | 1,000,000 / month |
| Durable Object WebSocket messages | 1,000,000 / month |
| D1 reads | 5,000,000 / day |
| D1 writes | 100,000 / day |
| D1 storage | 5 GB |
| KV reads | 100,000 / day |
| KV writes | 1,000 / day |
| Pages builds | 500 / month |

These limits comfortably support hundreds of concurrent players. If the game grows beyond this, the paid tier is $5/month for 10x the limits.

---

## 3. Architecture

```
┌──────────────────────────────────────────────────────────┐
│                     CLOUDFLARE EDGE                      │
│                                                          │
│  ┌─────────────┐    ┌──────────────────────────────────┐ │
│  │  CF Pages    │    │  CF Workers                      │ │
│  │  (React SPA  │    │                                  │ │
│  │   + WASM)    │    │  /api/auth/*    → OAuth flow     │ │
│  │              │    │  /api/match/*   → Matchmaking     │ │
│  │              │    │  /api/profile/* → Player CRUD     │ │
│  │              │    │  /api/leaderboard/* → Rankings    │ │
│  └─────────────┘    │                                  │ │
│                      │  WebSocket upgrade →              │ │
│                      │    Durable Object (Match Room)   │ │
│                      └──────────┬───────────────────────┘ │
│                                 │                         │
│                      ┌──────────▼───────────────────────┐ │
│                      │  Durable Objects                  │ │
│                      │                                  │ │
│                      │  MatchRoom DO:                    │ │
│                      │    - Holds WebSocket connections  │ │
│                      │    - Manages round state          │ │
│                      │    - Validates answers            │ │
│                      │    - Broadcasts events            │ │
│                      │                                  │ │
│                      │  MatchmakerDO:                    │ │
│                      │    - Single instance queue        │ │
│                      │    - Groups by ELO bracket        │ │
│                      │    - Creates MatchRoom DOs        │ │
│                      └──────────┬───────────────────────┘ │
│                                 │                         │
│                      ┌──────────▼──────┐  ┌────────────┐ │
│                      │  Cloudflare D1   │  │  CF KV     │ │
│                      │  (SQLite)        │  │            │ │
│                      │  - Players       │  │  - Session │ │
│                      │  - Match history │  │    tokens  │ │
│                      │  - Ratings       │  │  - LB cache│ │
│                      └─────────────────┘  └────────────┘ │
└──────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────┐
│                     BROWSER CLIENT                        │
│                                                          │
│  ┌──────────────────┐  ┌──────────────────────────────┐  │
│  │  React UI         │  │  WASM CAPTCHA Engine          │  │
│  │  - Lobby/Queue    │  │  - Seeded RNG                 │  │
│  │  - Match HUD      │  │  - Text generator             │  │
│  │  - Results screen │  │  - Image grid generator       │  │
│  │  - Profile/Stats  │  │  - Puzzle generators          │  │
│  │  - Leaderboard    │  │  - Difficulty parameterizer   │  │
│  └────────┬─────────┘  └──────────┬───────────────────┘  │
│           │                       │                       │
│           └───────── WebSocket ───┘                       │
│                   ↕ Server                                │
└──────────────────────────────────────────────────────────┘
```

### Key Architectural Decisions

**Seed-based generation**: The server never transmits CAPTCHA images or layouts. It sends a seed (u64) and difficulty parameters. Each client's WASM module deterministically generates the identical CAPTCHA from that seed. This minimizes bandwidth, prevents man-in-the-middle image interception, and ensures fairness.

**Server-side answer validation**: The WASM engine runs inside the MatchRoom Durable Object. When a player submits an answer, the DO regenerates the CAPTCHA from the same seed using the embedded WASM module and validates the response. The client is never trusted. The WASM binary is compiled with `wasm-opt -Oz` and stripped of debug symbols to stay within the 10 MB Worker bundle limit.

**Durable Object per match**: Each match room is its own Durable Object with in-memory state. WebSocket connections are held by the DO. When the match ends, results are flushed to D1 and the DO hibernates (no cost while idle).

---

## 4. CAPTCHA Generation Engine

The engine is a Rust library compiled to WASM. It exposes a single entry point:

```rust
pub fn generate_captcha(seed: u64, captcha_type: CaptchaType, difficulty: DifficultyParams) -> CaptchaInstance
```

`CaptchaInstance` contains everything the client needs to render the challenge and everything the server needs to validate the answer.

### 4.1 CAPTCHA Types by Tier

#### Tier 1 — Foundations (Levels 1–10)

| Type | Description | Difficulty Knobs |
|---|---|---|
| **Distorted Text** | Classic warped alphanumeric string | Character count (4→8), warp amplitude, noise density, font randomization, color variance |
| **Simple Math** | Arithmetic expressions rendered as distorted images | Operand range, operator count, parentheses depth |
| **Image Grid (Basic)** | "Select all squares with [object]" using procedurally placed shapes | Grid size (2×2→4×4), shape complexity, distractor similarity |
| **Slider Alignment** | Drag a puzzle piece to complete an image | Piece shape complexity, background noise, number of decoy slots |

#### Tier 2 — Perceptual (Levels 11–25)

| Type | Description | Difficulty Knobs |
|---|---|---|
| **Rotated Object** | Identify the correctly oriented object among rotations | Rotation granularity, object complexity, number of candidates |
| **Partial Occlusion** | Identify an object with increasing portions hidden | Occlusion percentage (20%→80%), object familiarity |
| **Semantic Oddity** | "Which of these doesn't belong?" with generated abstract shapes | Feature subtlety, number of candidates, shared-feature count |
| **Tone/Rhythm Pattern** | Identify the odd rhythm, count beats, match a tone sequence, or find the missing note | Tempo range, note count, rhythm complexity, rest placement, timbre similarity |
| **Color Perception** | Find the differently-shaded tile in a grid (Ishihara-inspired) | Color distance (ΔE), grid density, time pressure |

#### Tier 3 — Cognitive (Levels 26–50)

| Type | Description | Difficulty Knobs |
|---|---|---|
| **Adversarial Image** | Select the "real" image among adversarial perturbations that fool classifiers but not humans | Perturbation magnitude, classifier confidence gap |
| **Sequence Completion** | Complete a visual or logical pattern | Pattern complexity (rotation, reflection, color cycling), sequence length |
| **Multi-step Verification** | Chain of 2–3 micro-CAPTCHAs that must all be solved correctly | Chain length, type mixing, cumulative time pressure |
| **Spatial Reasoning** | Mental rotation, unfolded cube identification | Dimensionality (2D→3D), rotation count, distractor quality |
| **Contextual Reasoning** | "Which caption best describes this scene?" with procedurally generated scenes | Scene complexity, caption plausibility spread |

#### Tier 4 — Nightmare (Levels 50+)

| Type | Description | Difficulty Knobs |
|---|---|---|
| **Metamorphic CAPTCHA** | The CAPTCHA changes as you interact with it — elements drift, swap, or animate | Mutation rate, interaction responsiveness |
| **Combined Modality** | Solve a tone/rhythm + visual + logic challenge simultaneously | Modality count, cross-modal dependency |
| **Adversarial Typography** | Read text that is optimized to confuse OCR while remaining human-readable | Font adversariality, kerning distortion, ligature abuse |
| **Procedural Novel Type** | A CAPTCHA type the player has never seen before, with rules explained inline | Rule complexity, explanation clarity (intentionally low at high difficulty) |
| **Time Pressure Cascade** | Solve a sequence where each correct answer reduces the time for the next | Decay rate, starting time, sequence length |

### 4.2 Procedural Generation Architecture

```rust
// Core trait all generators implement
pub trait CaptchaGenerator {
    /// Generate a renderable captcha from a seed and difficulty
    fn generate(&self, rng: &mut StdRng, difficulty: &DifficultyParams) -> CaptchaInstance;

    /// Validate a player's answer against the expected solution
    fn validate(&self, instance: &CaptchaInstance, answer: &PlayerAnswer) -> bool;
}

pub struct CaptchaInstance {
    /// Rendering data sent to client (SVG paths, pixel data, text, audio samples)
    pub render_data: RenderPayload,

    /// Correct answer (kept server-side only)
    pub solution: Solution,

    /// Metadata for scoring
    pub expected_solve_time_ms: u32,
    pub point_value: u32,
    pub captcha_type: CaptchaType,
}

pub struct DifficultyParams {
    pub level: u32,            // Player level (1–100+)
    pub round_number: u32,     // Round within current match (difficulty ramps)
    pub time_limit_ms: u32,    // Computed from level + round
    pub complexity: f32,       // 0.0–1.0, maps to generator-specific knobs
    pub noise: f32,            // 0.0–1.0, visual/audio noise
}
```

### 4.3 Seeded RNG Protocol

Every match round uses a seed derived from:

```
round_seed = HMAC-SHA256(match_secret, round_number || timestamp)
```

The `match_secret` is generated server-side when the MatchRoom DO is created and never sent to clients. The server sends `round_seed` at the start of each round. Both client WASM and server WASM feed this seed to `rand::SeedableRng` to produce identical CAPTCHAs.

This ensures:
- All players in a room see the exact same CAPTCHA
- Seeds are not predictable (match_secret is unknown to clients)
- Replay verification is possible (store match_secret + round_numbers → regenerate any past CAPTCHA)

### 4.4 Expected Solve Times

These are baseline estimates for a median-skill player. Time limits are set at ~1.5× the expected solve time to create pressure without being unfair. All times in milliseconds.

#### Tier 1 — Foundations

| Type | Min Difficulty | Max Difficulty | Time Limit (min) | Time Limit (max) |
|---|---|---|---|---|
| Distorted Text (4 chars) | 2,500 | — | 5,000 | — |
| Distorted Text (8 chars) | — | 6,000 | — | 10,000 |
| Simple Math (1 operator) | 2,000 | — | 4,000 | — |
| Simple Math (3 operators + parens) | — | 8,000 | — | 12,000 |
| Image Grid 2×2 | 2,000 | — | 4,000 | — |
| Image Grid 4×4 | — | 5,000 | — | 8,000 |
| Slider Alignment | 1,500 | 4,000 | 3,000 | 6,000 |

#### Tier 2 — Perceptual

| Type | Min Difficulty | Max Difficulty | Time Limit (min) | Time Limit (max) |
|---|---|---|---|---|
| Rotated Object (4 candidates) | 3,000 | — | 5,000 | — |
| Rotated Object (8 candidates, 15° increments) | — | 7,000 | — | 11,000 |
| Partial Occlusion (20%) | 2,000 | — | 4,000 | — |
| Partial Occlusion (80%) | — | 8,000 | — | 12,000 |
| Semantic Oddity (4 shapes) | 3,000 | — | 5,000 | — |
| Semantic Oddity (8 shapes, subtle) | — | 10,000 | — | 15,000 |
| Tone/Rhythm (count beats) | 3,000 | — | 5,000 | — |
| Tone/Rhythm (find missing note) | — | 8,000 | — | 12,000 |
| Color Perception (high ΔE) | 1,500 | — | 3,000 | — |
| Color Perception (low ΔE, dense grid) | — | 6,000 | — | 9,000 |

#### Tier 3 — Cognitive

| Type | Min Difficulty | Max Difficulty | Time Limit (min) | Time Limit (max) |
|---|---|---|---|---|
| Adversarial Image (2 candidates) | 4,000 | — | 7,000 | — |
| Adversarial Image (6 candidates) | — | 10,000 | — | 15,000 |
| Sequence Completion (3-step) | 5,000 | — | 8,000 | — |
| Sequence Completion (6-step, compound) | — | 15,000 | — | 22,000 |
| Multi-step (2 chain) | 6,000 | — | 10,000 | — |
| Multi-step (3 chain) | — | 14,000 | — | 20,000 |
| Spatial Reasoning (2D) | 4,000 | — | 7,000 | — |
| Spatial Reasoning (3D, 3 rotations) | — | 12,000 | — | 18,000 |
| Contextual Reasoning | 5,000 | 12,000 | 8,000 | 18,000 |

#### Tier 4 — Nightmare

| Type | Min Difficulty | Max Difficulty | Time Limit (min) | Time Limit (max) |
|---|---|---|---|---|
| Metamorphic | 6,000 | 15,000 | 10,000 | 20,000 |
| Combined Modality | 8,000 | 20,000 | 12,000 | 25,000 |
| Adversarial Typography | 4,000 | 12,000 | 7,000 | 18,000 |
| Procedural Novel Type | 10,000 | 25,000 | 15,000 | 35,000 |
| Time Pressure Cascade | starts 5,000 | decays to 1,500 | starts 8,000 | decays to 2,500 |

#### Scoring Formula

```
base_points = captcha_tier * 10                        // T1=10, T2=20, T3=30, T4=40
speed_bonus = max(0, (time_limit - solve_time) / time_limit) * base_points
round_score = base_points + speed_bonus                // Range: base_points to 2×base_points
```

A player who solves instantly gets double points. A player who barely beats the timer gets base points only.

### 4.5 Adversarial Pattern Library

Rather than running a classifier in the generation loop, the engine ships with a static library of adversarial perturbation patterns known to fool common vision models. These are applied procedurally to seed-generated base images.

#### Library Structure

```rust
pub struct AdversarialPattern {
    /// Human-readable name for telemetry
    pub name: &'static str,

    /// Pixel-space noise mask (normalized, applied with variable magnitude)
    pub noise_mask: &'static [f32],

    /// Dimensions of the mask
    pub width: u32,
    pub height: u32,

    /// Which classifier families this pattern is known to fool
    pub effective_against: &'static [ClassifierFamily],

    /// Minimum magnitude for the perturbation to be effective against classifiers
    pub min_magnitude: f32,

    /// Maximum magnitude before humans start struggling
    pub max_human_magnitude: f32,
}

pub enum ClassifierFamily {
    ResNet,
    VisionTransformer,
    CLIP,
    EfficientNet,
    ConvNeXt,
}
```

#### Pattern Categories

| Category | Description | Count (initial library) |
|---|---|---|
| **High-frequency noise** | Patterns that exploit convolutional filter sensitivity to specific frequency bands. Invisible to humans at low magnitude, devastating to CNNs. | ~20 masks |
| **Texture bias exploits** | Patterns that shift texture cues without changing shape. Humans see a dog, classifiers see a cat. Based on the Geirhos et al. texture-bias findings. | ~15 templates |
| **Patch attacks** | Small localized patches that dominate classifier attention. Humans ignore them as visual noise. | ~10 patches |
| **Color shift perturbations** | Subtle color-space rotations in regions that classifiers weight heavily. Imperceptible to humans. | ~10 masks |
| **Typographic adversarials** | Font perturbations that break OCR while remaining human-readable. Kerning distortion, ligature swaps, stroke noise. | ~25 font modifications |

#### Generation Flow

```
1. Seed RNG generates a base image (object, scene, or text)
2. Generator selects N patterns from the library (based on difficulty)
3. Patterns are applied at magnitude scaled by difficulty.complexity
4. One image is left unperturbed (or minimally perturbed) — this is the correct answer
5. Remaining images have perturbations strong enough to flip classifier predictions
6. Solution: select the "real" (unperturbed) image
```

The difficulty knob controls how many candidates there are, how similar the base images are, and how subtle the perturbation magnitude is (closer to `min_magnitude` = harder for humans to spot the difference, but still effective against classifiers).

#### Library Maintenance

The pattern library is versioned and shipped as a static asset alongside the WASM binary. Updates are a content patch — compile new patterns, rebuild WASM, redeploy. Plan for quarterly reviews as classifier architectures evolve.

---

## 5. Multiplayer System

### 5.1 Match Flow

```
MATCHMAKING         COUNTDOWN       ROUNDS              RESULTS
    │                   │              │                    │
    ▼                   ▼              ▼                    ▼
Players join queue → Lobby fills → Round 1 starts →  ... → Final standings
(ELO bracket)        (3s countdown)  (all solve same       (ELO adjustment)
                                      CAPTCHA)
                                        │
                                        ▼
                                   Wrong answer OR
                                   timeout = elimination
                                        │
                                        ▼
                                   Last player standing
                                   OR highest score wins
```

### 5.2 Game Modes

**Battle Royale (Primary)**
- 4–16 players per room
- Each round, all players solve the same CAPTCHA
- Wrong answer or timeout → eliminated
- Difficulty increases each round (even within a single match)
- Last player standing wins
- If multiple players survive all rounds, fastest cumulative time wins

**Sprint (Ranked)**
- 2–8 players
- Solve 10 CAPTCHAs as fast as possible
- No elimination — pure speed
- Final score = accuracy × speed bonus
- More granular ELO adjustment

**Endless (Unranked)**
- Solo or with friends (private room code)
- CAPTCHAs keep coming until you fail
- Personal best tracking
- Good for practice and warming up

### 5.3 WebSocket Protocol

All messages are JSON over WebSocket, managed by the MatchRoom Durable Object.

#### Server → Client Messages

```typescript
// Server broadcasts to all players in room
type ServerMessage =
  | { type: "lobby_update"; players: PlayerInfo[]; countdown?: number }
  | { type: "round_start"; round: number; seed: bigint; captcha_type: CaptchaType;
      difficulty: DifficultyParams; time_limit_ms: number }
  | { type: "player_solved"; player_id: string; time_ms: number }  // no answer leaked
  | { type: "player_eliminated"; player_id: string; reason: "wrong" | "timeout" }
  | { type: "round_end"; standings: Standing[] }
  | { type: "match_end"; final_standings: FinalStanding[]; elo_changes: EloChange[] }
  | { type: "error"; code: string; message: string };
```

#### Client → Server Messages

```typescript
type ClientMessage =
  | { type: "submit_answer"; round: number; answer: PlayerAnswer; client_time_ms: number }
  | { type: "heartbeat" }
  | { type: "forfeit" };
```

### 5.4 Durable Object: MatchRoom

```typescript
export class MatchRoom implements DurableObject {
  private state: DurableObjectState;
  private env: Env;
  private players: Map<string, { ws: WebSocket; info: PlayerInfo; alive: boolean }>;
  private matchState: {
    matchId: string;
    secret: Uint8Array;        // For seed derivation
    round: number;
    mode: GameMode;
    difficulty: DifficultyParams;
    roundStartTime: number;
    roundAnswers: Map<string, { answer: PlayerAnswer; time_ms: number }>;
  };

  async fetch(request: Request): Promise<Response> {
    // WebSocket upgrade + session validation
    const [client, server] = Object.values(new WebSocketPair());
    this.state.acceptWebSocket(server);
    // Tag with player ID for routing
    server.serializeAttachment({ playerId });
    return new Response(null, { status: 101, webSocket: client });
  }

  async webSocketMessage(ws: WebSocket, msg: string) {
    const data: ClientMessage = JSON.parse(msg);
    switch (data.type) {
      case "submit_answer":
        await this.handleAnswer(ws, data);
        break;
      case "forfeit":
        this.eliminatePlayer(ws, "forfeit");
        break;
    }
  }

  private async handleAnswer(ws: WebSocket, data: SubmitAnswer) {
    // Regenerate CAPTCHA server-side via embedded WASM module (same seed)
    const captcha = this.wasmEngine.generate(this.matchState.roundSeed,
      this.matchState.captchaType, this.matchState.difficulty);
    const correct = this.wasmEngine.validate(captcha, data.answer);
    // Broadcast result (solved or eliminated)
    // If all alive players answered, advance round
  }

  // Hibernation: Cloudflare hibernates the DO when no WebSocket
  // activity occurs, saving costs. State is persisted automatically.
  async webSocketClose(ws: WebSocket) {
    this.eliminatePlayer(ws, "disconnect");
    if (this.players.size === 0) {
      await this.flushMatchResults();
    }
  }
}
```

---

## 6. Matchmaking

### 6.1 ELO System

New players start at **1000 ELO**. After each match, ELO is adjusted using a modified multiplayer ELO formula:

```
For each pair of players (i, j) in the match:
  expected_i = 1 / (1 + 10^((elo_j - elo_i) / 400))
  actual_i   = 1 if i placed higher than j, 0.5 if tied, 0 if lower
  delta_i   += K * (actual_i - expected_i)

K factor:
  - New players (< 30 matches): K = 40
  - Established (30–100 matches): K = 24
  - Veterans (100+ matches): K = 16
```

In a 16-player battle royale, each player is compared against all 15 others, so a single match can produce significant ELO swings. The K factor is divided by `(N-1)` to normalize.

### 6.2 Matchmaking Queue

A single **MatchmakerDO** (Durable Object singleton) manages the global queue.

```
Queue Structure:
  Brackets: [0–800] [800–1000] [1000–1200] [1200–1500] [1500+]

  Each bracket has a waiting list with timestamps.

  Every tick (1 second via alarm):
    For each bracket:
      If players >= MIN_PLAYERS (4) AND (players >= TARGET (8) OR oldest_wait > 15s):
        Pop up to MAX_PLAYERS (16) from bracket
        Create new MatchRoom DO
        Send room ID to all popped players via their WebSocket

  Bracket expansion:
    If a player has waited > 30s, expand search to adjacent brackets
    If waited > 60s, expand to ±2 brackets
    Never match players more than 2 brackets apart
```

### 6.3 Matchmaker Durable Object

```typescript
export class Matchmaker implements DurableObject {
  private queues: Map<string, QueueEntry[]> = new Map();

  async fetch(request: Request): Promise<Response> {
    const [client, server] = Object.values(new WebSocketPair());
    this.state.acceptWebSocket(server);
    // Add player to appropriate bracket
    const { playerId, elo } = await validateSession(request, this.env);
    const bracket = this.getBracket(elo);
    this.queues.get(bracket)!.push({ playerId, elo, ws: server, joinedAt: Date.now() });
    this.scheduleAlarm();
    return new Response(null, { status: 101, webSocket: client });
  }

  async alarm() {
    // Run matchmaking tick
    for (const [bracket, queue] of this.queues) {
      if (this.shouldCreateMatch(queue)) {
        const players = queue.splice(0, MAX_PLAYERS);
        const roomId = crypto.randomUUID();
        const roomStub = this.env.MATCH_ROOM.get(
          this.env.MATCH_ROOM.idFromName(roomId)
        );
        // Initialize room with player list and mode
        await roomStub.fetch(new Request("https://internal/init", {
          method: "POST",
          body: JSON.stringify({ players: players.map(p => p.playerId), mode: "battle_royale" })
        }));
        // Notify players to connect to the match room
        for (const p of players) {
          p.ws.send(JSON.stringify({ type: "match_found", roomId }));
        }
      }
    }
    // Bracket expansion for long-waiting players
    this.expandBrackets();
    // Re-schedule if queue is non-empty
    if (this.totalQueued() > 0) {
      this.state.storage.setAlarm(Date.now() + 1000);
    }
  }

  private getBracket(elo: number): string {
    if (elo < 800) return "bronze";
    if (elo < 1000) return "silver";
    if (elo < 1200) return "gold";
    if (elo < 1500) return "platinum";
    return "diamond";
  }
}
```

---

## 7. Progression System

### 7.1 XP and Levels

XP is earned from every match, win or lose. Leveling unlocks new CAPTCHA tiers (which increases difficulty and variety).

```
XP Sources:
  - Each CAPTCHA solved:        10 × difficulty_multiplier
  - Fastest solve in a round:   +25 bonus
  - Match win:                  100 × (players_in_match / 4)
  - Top 3 finish:               50 × (players_in_match / 4)
  - Participation:              20

Level curve (exponential):
  Level N requires: 100 * N^1.5 total XP
  Level 1:  100 XP
  Level 10: 3,162 XP
  Level 25: 12,500 XP
  Level 50: 35,355 XP
```

### 7.2 CAPTCHA Tier Unlocks

| Level Range | Tiers Available | Notes |
|---|---|---|
| 1–10 | Tier 1 only | Learning the ropes — text, math, basic grids |
| 11–25 | Tier 1 + Tier 2 | Perceptual challenges mixed in, ~30% Tier 2 |
| 26–50 | Tier 1–3 | Cognitive challenges appear, ratio shifts toward higher tiers |
| 50+ | All tiers | Nightmare tier CAPTCHAs, ~40% Tier 3, ~20% Tier 4 |

Within a match, players share the same CAPTCHAs, so the tier selection is based on the **median level** of all players in the room (which is naturally close due to ELO-based matchmaking).

### 7.3 Titles and Cosmetics

Since there's no monetization, all cosmetics are progression-based.

- **Titles**: Earned at level milestones ("Bot Buster", "Turing Complete", "CAPTCHA Demon")
- **Profile borders**: Unlocked by winstreaks, total wins, specific achievements
- **Name colors**: Tied to current ELO bracket
- **Achievements**: "Solve 100 CAPTCHAs under 2 seconds", "Win a 16-player lobby", "Survive 50 rounds in Endless", etc.

---

## 8. Authentication

### 8.1 OAuth2 Flow on Cloudflare Workers

No framework needed. The Worker handles the OAuth dance directly.

```
Browser                    Worker (/api/auth)              OAuth Provider
   │                            │                              │
   ├─ GET /api/auth/google ────►│                              │
   │                            ├─ Generate state, store in KV │
   │                            ├─ 302 Redirect ──────────────►│
   │  ◄─────────────────────────┤                              │
   │                            │                              │
   │  (User consents)           │                              │
   │                            │                              │
   │  GET /api/auth/callback ──►│                              │
   │  ?code=xxx&state=yyy       ├─ Verify state from KV        │
   │                            ├─ Exchange code for token ────►│
   │                            │◄─ Access token + profile ────┤
   │                            ├─ Upsert player in D1         │
   │                            ├─ Create session token in KV  │
   │  ◄── Set-Cookie: session ──┤                              │
   │                            │                              │
   │  GET /api/profile ────────►│                              │
   │                            ├─ Read session from cookie    │
   │                            ├─ Lookup in KV → player_id    │
   │  ◄── { player profile } ──┤                              │
```

### 8.2 Session Management

- Session tokens are UUIDs stored in KV with a 7-day TTL
- Stored in `HttpOnly`, `Secure`, `SameSite=Strict` cookies
- Each KV entry maps `session:{token}` → `{ playerId, expiresAt }`
- On WebSocket upgrade, the session cookie is validated before accepting the connection

### 8.3 Provider Configuration

```typescript
// wrangler.toml secrets (set via `wrangler secret put`)
// GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET
// DISCORD_CLIENT_ID, DISCORD_CLIENT_SECRET
// GITHUB_CLIENT_ID, GITHUB_CLIENT_SECRET

const PROVIDERS = {
  google: {
    authUrl: "https://accounts.google.com/o/oauth2/v2/auth",
    tokenUrl: "https://oauth2.googleapis.com/token",
    profileUrl: "https://www.googleapis.com/oauth2/v2/userinfo",
    scopes: ["openid", "email", "profile"],
  },
  discord: {
    authUrl: "https://discord.com/api/oauth2/authorize",
    tokenUrl: "https://discord.com/api/oauth2/token",
    profileUrl: "https://discord.com/api/users/@me",
    scopes: ["identify", "email"],
  },
  github: {
    authUrl: "https://github.com/login/oauth/authorize",
    tokenUrl: "https://github.com/login/oauth/access_token",
    profileUrl: "https://api.github.com/user",
    scopes: ["read:user", "user:email"],
  },
};
```

---

## 9. Database Schema (Cloudflare D1)

```sql
-- Players
CREATE TABLE players (
  id            TEXT PRIMARY KEY,         -- UUID
  display_name  TEXT NOT NULL,
  avatar_url    TEXT,
  elo           INTEGER NOT NULL DEFAULT 1000,
  level         INTEGER NOT NULL DEFAULT 1,
  xp            INTEGER NOT NULL DEFAULT 0,
  matches_played INTEGER NOT NULL DEFAULT 0,
  wins          INTEGER NOT NULL DEFAULT 0,
  created_at    TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

-- OAuth identities (supports multiple providers per player)
CREATE TABLE oauth_identities (
  provider      TEXT NOT NULL,            -- 'google' | 'discord' | 'github'
  provider_id   TEXT NOT NULL,            -- ID from the provider
  player_id     TEXT NOT NULL REFERENCES players(id),
  email         TEXT,
  created_at    TEXT NOT NULL DEFAULT (datetime('now')),
  PRIMARY KEY (provider, provider_id)
);
CREATE INDEX idx_oauth_player ON oauth_identities(player_id);

-- Match history
CREATE TABLE matches (
  id            TEXT PRIMARY KEY,         -- UUID
  mode          TEXT NOT NULL,            -- 'battle_royale' | 'sprint' | 'endless'
  player_count  INTEGER NOT NULL,
  rounds_played INTEGER NOT NULL,
  median_level  INTEGER NOT NULL,
  started_at    TEXT NOT NULL,
  ended_at      TEXT NOT NULL
);

-- Per-player match results
CREATE TABLE match_results (
  match_id      TEXT NOT NULL REFERENCES matches(id),
  player_id     TEXT NOT NULL REFERENCES players(id),
  placement     INTEGER NOT NULL,         -- 1 = winner
  elo_before    INTEGER NOT NULL,
  elo_after     INTEGER NOT NULL,
  xp_earned     INTEGER NOT NULL,
  rounds_survived INTEGER NOT NULL,
  avg_solve_ms  INTEGER,                  -- Average solve time
  PRIMARY KEY (match_id, player_id)
);
CREATE INDEX idx_match_results_player ON match_results(player_id);

-- Achievements
CREATE TABLE achievements (
  id            TEXT PRIMARY KEY,         -- e.g., 'first_win', 'solve_100_under_2s'
  name          TEXT NOT NULL,
  description   TEXT NOT NULL,
  icon          TEXT                      -- SVG or emoji
);

CREATE TABLE player_achievements (
  player_id     TEXT NOT NULL REFERENCES players(id),
  achievement_id TEXT NOT NULL REFERENCES achievements(id),
  unlocked_at   TEXT NOT NULL DEFAULT (datetime('now')),
  PRIMARY KEY (player_id, achievement_id)
);

-- Leaderboards (materialized periodically, cached in KV)
CREATE TABLE leaderboard_snapshots (
  season        TEXT NOT NULL,            -- e.g., '2026-Q1'
  player_id     TEXT NOT NULL REFERENCES players(id),
  elo           INTEGER NOT NULL,
  rank          INTEGER NOT NULL,
  snapshot_at   TEXT NOT NULL DEFAULT (datetime('now')),
  PRIMARY KEY (season, player_id)
);
```

---

## 10. Anti-Cheat

### 10.1 Threat Model

| Threat | Mitigation |
|---|---|
| **Bot solvers (OCR/ML)** | High-tier CAPTCHAs are specifically designed to fool classifiers. Adversarial images, metamorphic CAPTCHAs, and combined-modality challenges resist automation. |
| **Solve time manipulation** | Server records its own timestamp at `round_start` and `answer_received`. Client-reported time is logged but not trusted for ranking. |
| **Seed prediction** | Seeds are derived from a server-side secret. Clients never see the secret. |
| **WASM reverse engineering** | The CAPTCHA engine WASM binary is obfuscated. Answer extraction would require understanding the generator logic. Combined with server-side validation, this is a high barrier. |
| **Multi-accounting** | OAuth providers give unique IDs. Flagging accounts with identical device fingerprints or suspiciously similar play patterns. |
| **Collusion** | In Battle Royale, all players see the same CAPTCHA, so there's no information asymmetry to exploit via collusion. |

### 10.2 Statistical Anomaly Detection

```
For each player, maintain rolling statistics:
  - Median solve time per CAPTCHA type per difficulty
  - Solve time variance
  - Accuracy rate per tier

Flag for review if:
  - Solve time < 3 standard deviations below population median for that type/difficulty
  - Accuracy > 99% over 100+ CAPTCHAs at Tier 3+
  - Solve time variance < threshold (bots are unnaturally consistent)

Automated actions:
  - Soft flag: shadow-queue into "verification" matches with known-hard CAPTCHAs
  - Hard flag: require re-authentication + a live proctored CAPTCHA sequence
  - Ban: remove from leaderboards, restrict matchmaking
```

---

## 11. API Routes

### Auth

| Method | Route | Description |
|---|---|---|
| GET | `/api/auth/:provider` | Initiate OAuth flow (google, discord, github) |
| GET | `/api/auth/callback` | OAuth callback, sets session cookie |
| POST | `/api/auth/logout` | Clear session |
| GET | `/api/auth/me` | Return current player profile (from session) |

### Player

| Method | Route | Description |
|---|---|---|
| GET | `/api/profile/:id` | Public player profile |
| PATCH | `/api/profile` | Update display name / settings |
| GET | `/api/profile/:id/history` | Match history (paginated) |
| GET | `/api/profile/:id/achievements` | Unlocked achievements |

### Matchmaking

| Method | Route | Description |
|---|---|---|
| WebSocket | `/api/match/queue` | Join matchmaking queue (upgrade to WS) |
| WebSocket | `/api/match/room/:id` | Connect to a specific match room |
| POST | `/api/match/private` | Create a private room, returns room code |
| POST | `/api/match/join/:code` | Join a private room by code |

### Leaderboard

| Method | Route | Description |
|---|---|---|
| GET | `/api/leaderboard?season=current` | Top 100 by ELO (cached in KV) |
| GET | `/api/leaderboard/around/:id` | 10 players around a specific player's rank |

---

## 12. Frontend Structure

**Desktop only.** The app renders a full-screen gate component if `window.innerWidth < 1024`, directing users to visit on a desktop browser. No responsive breakpoints, no touch handlers.

```
apps/web/src/
├── main.tsx                    # Entry point
├── App.tsx                     # Router setup
├── pages/
│   ├── Home.tsx                # Landing, play button, quick stats
│   ├── Queue.tsx               # Matchmaking waiting room
│   ├── Match.tsx               # Active match gameplay
│   ├── Results.tsx             # Post-match standings, ELO changes
│   ├── Profile.tsx             # Player stats, match history, achievements
│   ├── Leaderboard.tsx         # Global and bracket leaderboards
│   └── Login.tsx               # OAuth provider selection
├── components/
│   ├── captcha/
│   │   ├── CaptchaRenderer.tsx # Dispatches to type-specific renderers
│   │   ├── TextCaptcha.tsx
│   │   ├── GridCaptcha.tsx
│   │   ├── SliderCaptcha.tsx
│   │   ├── MathCaptcha.tsx
│   │   └── ...                 # One per CAPTCHA type
│   ├── match/
│   │   ├── PlayerList.tsx      # Shows all players, alive/eliminated status
│   │   ├── Timer.tsx           # Countdown per round
│   │   ├── RoundIndicator.tsx  # Current round / total
│   │   └── EliminationFeed.tsx # "Player X was eliminated"
│   ├── ui/
│   │   ├── Button.tsx
│   │   ├── Modal.tsx
│   │   └── ...
│   └── layout/
│       ├── Header.tsx
│       └── Footer.tsx
├── hooks/
│   ├── useWebSocket.ts         # WebSocket connection management
│   ├── useMatchState.ts        # Match state machine (lobby → playing → results)
│   ├── useCaptchaEngine.ts     # WASM module loader and interface
│   └── useAuth.ts              # Session state, login/logout
├── lib/
│   ├── wasm.ts                 # WASM initialization and bindings
│   ├── api.ts                  # REST API client
│   └── elo.ts                  # Client-side ELO display calculations
└── types/
    ├── captcha.ts              # CaptchaType, DifficultyParams, etc.
    ├── match.ts                # ServerMessage, ClientMessage, etc.
    └── player.ts               # PlayerInfo, Standing, etc.
```

---

## 13. Monorepo Structure

```
captcha-royale/
├── packages/
│   └── captcha-engine/         # Rust WASM crate
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs          # Public API: generate_captcha, validate_answer
│       │   ├── rng.rs          # Seeded RNG wrapper
│       │   ├── types.rs        # CaptchaInstance, DifficultyParams, Solution
│       │   ├── generators/
│       │   │   ├── mod.rs
│       │   │   ├── text.rs     # Distorted text generator
│       │   │   ├── grid.rs     # Image grid generator
│       │   │   ├── math.rs     # Math expression generator
│       │   │   ├── slider.rs   # Slider puzzle generator
│       │   │   ├── rotation.rs # Rotated object generator
│       │   │   ├── adversarial.rs
│       │   │   ├── sequence.rs
│       │   │   ├── metamorphic.rs
│       │   │   └── ...
│       │   └── difficulty.rs   # Maps level → DifficultyParams per type
│       └── tests/
├── apps/
│   ├── web/                    # React SPA (Vite)
│   │   ├── package.json
│   │   ├── vite.config.ts
│   │   ├── public/
│   │   └── src/                # (see Frontend Structure above)
│   └── worker/                 # Cloudflare Workers
│       ├── wrangler.toml
│       ├── src/
│       │   ├── index.ts        # Router: dispatches to handlers
│       │   ├── auth.ts         # OAuth handlers
│       │   ├── matchmaker.ts   # MatchmakerDO class
│       │   ├── match-room.ts   # MatchRoom DO class
│       │   ├── api/
│       │   │   ├── profile.ts
│       │   │   ├── leaderboard.ts
│       │   │   └── match.ts
│       │   └── lib/
│       │       ├── d1.ts       # D1 query helpers
│       │       ├── session.ts  # Session validation
│       │       └── elo.ts      # ELO calculation
│       └── migrations/         # D1 schema migrations
│           └── 0001_init.sql
├── turbo.json                  # Turborepo config
├── package.json                # Root workspace
└── README.md
```

---

## 14. Implementation Roadmap

### Phase 1 — Foundation (Weeks 1–3)

**Goal**: A playable single-player prototype with 3 CAPTCHA types.

- [ ] Initialize monorepo (Turborepo, pnpm workspaces)
- [ ] Scaffold Rust crate with `wasm-pack` build
- [ ] Implement generators: Distorted Text, Simple Math, Image Grid (Basic)
- [ ] Implement `DifficultyParams` mapping for all three types
- [ ] **Tests**: Seed determinism suite (§15.1), generator correctness tests for all 3 types, fuzz targets
- [ ] **CI**: Set up GitHub Actions with `cargo test`, `wasm-pack test`, `cargo clippy`
- [ ] Build React app with WASM loader
- [ ] Build `CaptchaRenderer` component (dispatches to type-specific renderers)
- [ ] Build solo "Endless" mode (no server, pure client-side)
- [ ] **Tests**: WASM integration tests (§15.3), CaptchaRenderer component tests, desktop gate test
- [ ] Verify seed determinism: same seed → identical CAPTCHA on different browsers (cross-browser wasm-pack test)

### Phase 2 — Backend & Auth (Weeks 4–5)

**Goal**: Persistent accounts and player profiles.

- [ ] Set up Cloudflare Workers project with `wrangler`
- [ ] Implement OAuth flow for Google, Discord, GitHub
- [ ] **Tests**: Auth Worker tests (§15.2) — redirect, callback, session, logout, multi-provider linking
- [ ] Set up D1 database with initial schema migration
- [ ] Implement session management with KV
- [ ] Build profile API (CRUD, match history)
- [ ] **Tests**: API route tests (§15.2) — profile CRUD, 404s, pagination, auth guards
- [ ] **Tests**: Security tests (§15.6) — forged sessions, SQL injection, XSS display names
- [ ] Connect frontend auth flow (login page, session hooks)
- [ ] **Tests**: `useAuth` hook tests, Login component tests
- [ ] Deploy to Cloudflare Pages + Workers
- [ ] **CI**: Add Miniflare integration tests and Vitest frontend tests to pipeline

### Phase 3 — Multiplayer (Weeks 6–8)

**Goal**: Real-time matches with 2–16 players.

- [ ] Implement MatchRoom Durable Object (WebSocket management, round state)
- [ ] Implement server-side CAPTCHA validation (WASM in Durable Object)
- [ ] **Tests**: MatchRoom DO tests (§15.2) — full round lifecycle, elimination, timeout, reconnection grace period, concurrent submissions, 16-player room
- [ ] Implement MatchmakerDO with ELO-bracket queuing
- [ ] **Tests**: Matchmaker DO tests (§15.2) — bracket assignment, match creation thresholds, bracket expansion, queue cleanup
- [ ] Build matchmaking UI (queue screen with player count, estimated wait)
- [ ] Build match HUD (player list, timer, elimination feed, round counter)
- [ ] Build results screen (standings, ELO changes, XP earned)
- [ ] **Tests**: Frontend component tests for Queue, PlayerList, Timer, EliminationFeed, Results (§15.3)
- [ ] **Tests**: WebSocket hook tests, match state machine tests (§15.3)
- [ ] Implement private rooms (create/join by code)
- [ ] **Tests**: E2E tests (§15.4) — 2-player match, private room, disconnection recovery
- [ ] **CI**: Add Playwright E2E tests to merge pipeline, k6 load tests to nightly
- [ ] Load test with simulated WebSocket connections
- [ ] **Tests**: Load tests (§15.5) — concurrent matches, matchmaker throughput, free tier budget

### Phase 4 — Progression & Polish (Weeks 9–11)

**Goal**: Rewarding meta-game and competitive depth.

- [ ] Implement XP/level system with tier unlocks
- [ ] Add Tier 2 CAPTCHA generators (Rotated Object, Partial Occlusion, Semantic Oddity, Tone/Rhythm, Color Perception)
- [ ] **Tests**: Generator correctness tests for all Tier 2 types, rhythm/tone-specific tests (§15.1)
- [ ] Implement achievement system (tracking + unlock notifications)
- [ ] **Tests**: E2E achievement unlock test (§15.4)
- [ ] Build leaderboard (global, bracket, and friends)
- [ ] **Tests**: Leaderboard API tests, E2E leaderboard update test
- [ ] Implement Sprint game mode
- [ ] Add spectator mode (watch ongoing matches)
- [ ] Sound design: countdown beeps, solve chime, elimination buzz, match win fanfare
- [ ] **Tests**: First balance playtest session (§15.7) — solve time distributions, accuracy by tier, ELO convergence

### Phase 5 — Nightmare Tier & Anti-Cheat (Weeks 12–14)

**Goal**: High-level content that resists bots and delights sweats.

- [ ] Implement Tier 3 generators (Adversarial Image, Sequence Completion, Multi-step, Spatial Reasoning)
- [ ] Implement Tier 4 generators (Metamorphic, Combined Modality, Adversarial Typography, Time Pressure Cascade)
- [ ] **Tests**: Generator correctness tests for all Tier 3–4 types, adversarial library tests (§15.1)
- [ ] Build statistical anomaly detection pipeline
- [ ] Implement soft/hard flagging system
- [ ] **Tests**: Security tests for answer spam, WASM tampering, cross-room messaging (§15.6)
- [ ] **Tests**: Stress tests — rapid answer submission, reconnect storm, max player overflow (§15.5)
- [ ] Seasonal leaderboard resets with rewards
- [ ] Performance optimization pass (WASM binary size, render pipeline, WebSocket message compression)
- [ ] **CI**: Add weekly security test suite, WASM bundle size alert (> 8 MB)
- [ ] **Tests**: Second balance playtest session — Tier 3/4 solve times, adversarial human solvability, elimination rate tuning

### Phase 6 — Launch (Week 15+)

- [ ] **Tests**: Full E2E suite green on staging (16-player match, all CAPTCHA types)
- [ ] **Tests**: Load test at 2× expected launch traffic
- [ ] **Tests**: Complete security audit pass
- [ ] Open beta with friends/small community
- [ ] **Tests**: Balance playtest with beta players — collect telemetry, tune difficulty curves
- [ ] Write Show HN post
- [ ] Monitor Cloudflare free tier usage, optimize if needed
- [ ] Iterate on CAPTCHA balance based on solve-time distributions
- [ ] Community feedback → new CAPTCHA types, balance changes

---

## 15. Test Plan

Testing is organized into six layers: unit, integration, end-to-end, load/stress, security, and balance. Every layer maps to a CI gate or a scheduled job.

### 15.1 CAPTCHA Engine (Rust — `cargo test`)

These tests run in pure Rust (no browser, no WASM target) for speed, plus a WASM-specific suite that runs in a headless browser via `wasm-pack test`.

#### Seed Determinism

The single most critical property of the engine: identical seeds must produce identical CAPTCHAs across platforms.

| Test | Description | Pass Criteria |
|---|---|---|
| `test_seed_determinism_same_platform` | Generate 1,000 CAPTCHAs per type with the same seed. Compare `RenderPayload` and `Solution` byte-for-byte. | 100% identical |
| `test_seed_determinism_cross_target` | Generate CAPTCHAs targeting both `x86_64` (native) and `wasm32-unknown-unknown`. Compare outputs. | Byte-identical `Solution` fields. `RenderPayload` may differ in floating-point edge cases — assert within ε = 1e-6 for f32 fields. |
| `test_seed_determinism_cross_browser` | WASM-pack test: run generation in Chrome, Firefox, and Safari headless. Compare serialized outputs. | Identical within ε for all browser engines. |
| `test_different_seeds_differ` | Generate CAPTCHAs with 10,000 distinct seeds. Assert no two produce identical `Solution` values. | 0 collisions (probabilistically guaranteed for u64 seeds). |
| `test_seed_from_hmac` | Verify `HMAC-SHA256(secret, round || timestamp)` produces the expected seed for known test vectors. | Matches reference values. |

#### Generator Correctness (per CAPTCHA type)

Each generator gets its own test module. Below is the pattern — repeat for all ~20 generator types.

| Test | Description | Pass Criteria |
|---|---|---|
| `test_{type}_generates_valid_instance` | Generate 100 instances at each difficulty level (1, 10, 25, 50, 75, 100). Assert `CaptchaInstance` fields are non-empty, `solution` is populated, `render_data` is parseable. | No panics, no empty fields. |
| `test_{type}_solution_validates` | Generate an instance, extract its `Solution`, feed it back to `validate()`. | Returns `true`. |
| `test_{type}_wrong_answer_rejects` | Generate an instance, submit deliberately wrong answers (empty, random, off-by-one, adjacent option). | Returns `false` for all wrong answers. |
| `test_{type}_difficulty_scaling` | Generate instances at difficulty 0.0, 0.5, 1.0. Assert that measurable complexity metrics increase monotonically: character count (text), grid size (grid), candidate count (rotation), noise magnitude (adversarial). | Monotonically non-decreasing. |
| `test_{type}_time_limit_scales` | Assert `expected_solve_time_ms` and time limits match the values in Section 4.4 within ±10%. | Within tolerance. |
| `test_{type}_render_data_bounds` | Assert rendered SVG/pixel dimensions fit within the maximum canvas size (800×600). Assert no negative coordinates, no NaN values, no empty paths. | All assertions pass. |

#### Adversarial Pattern Library

| Test | Description | Pass Criteria |
|---|---|---|
| `test_all_patterns_load` | Deserialize every pattern in the static library. Assert valid dimensions, non-zero mask data. | All ~80 patterns load without error. |
| `test_pattern_magnitude_range` | For each pattern, assert `min_magnitude < max_human_magnitude`. | Invariant holds for all patterns. |
| `test_adversarial_generation_applies_patterns` | Generate adversarial CAPTCHAs at varying difficulties. Assert that the "perturbed" images differ from the "clean" image by at least `min_magnitude` L2 norm. | Perturbation is non-trivial. |
| `test_adversarial_correct_answer_is_clean` | Generate 100 adversarial CAPTCHAs. Verify the solution always points to the unperturbed image. | 100% correct solution mapping. |

#### Tone/Rhythm Generator

| Test | Description | Pass Criteria |
|---|---|---|
| `test_rhythm_pattern_valid` | Generate 100 rhythm patterns. Assert all note durations are positive, total duration is within expected range, BPM is within configured bounds. | All valid. |
| `test_rhythm_odd_one_out_differs` | Generate "find the odd rhythm" challenges. Assert the odd rhythm differs from the base pattern by at least one beat. | Non-trivial difference in every case. |
| `test_tone_sequence_frequencies_valid` | Assert all generated frequencies are within the audible range (20 Hz – 20 kHz) and within the configured scale. | All within bounds. |

#### Fuzz Testing

| Test | Description | Pass Criteria |
|---|---|---|
| `fuzz_generate_captcha` | `cargo-fuzz` target: random `(seed, captcha_type, difficulty)` tuples. Run for 10M iterations. | No panics, no OOM, no infinite loops. |
| `fuzz_validate_answer` | Random `(CaptchaInstance, PlayerAnswer)` pairs. | No panics. Returns `true` or `false` only. |

### 15.2 Server — Durable Objects & Workers (Vitest + Miniflare)

Tests run against Miniflare (local Cloudflare Workers simulator) with real D1 databases and Durable Object stubs.

#### MatchRoom DO

| Test | Description | Pass Criteria |
|---|---|---|
| `test_room_initialization` | Create a MatchRoom DO, POST `/init` with a player list and mode. Assert room state is correctly initialized. | State has correct player list, mode, round=0, all players alive. |
| `test_websocket_upgrade` | Connect a WebSocket to the room with a valid session. Assert the connection is accepted and the player receives a `lobby_update` message. | WebSocket open, `lobby_update` received. |
| `test_websocket_rejects_invalid_session` | Attempt WebSocket upgrade with an expired or forged session token. | Connection rejected with 401. |
| `test_round_lifecycle` | Connect 4 players. Assert: (1) `round_start` is broadcast with seed and params, (2) submitting correct answers broadcasts `player_solved`, (3) all players answering triggers `round_end`, (4) next round starts automatically. | Full round cycle completes without error. |
| `test_elimination_on_wrong_answer` | Submit an incorrect answer. Assert the player receives `player_eliminated` with reason `"wrong"` and is excluded from subsequent rounds. | Player marked as eliminated, no longer receives `round_start`. |
| `test_elimination_on_timeout` | Connect a player but never submit an answer. Wait for time limit + buffer. Assert elimination with reason `"timeout"`. | Eliminated after time limit. |
| `test_last_player_wins` | Start a 4-player match. Eliminate 3 players. Assert `match_end` is broadcast with the survivor as winner. | Correct final standings. |
| `test_match_results_flushed_to_d1` | Complete a match. Query D1 for `matches` and `match_results` rows. | Rows exist with correct data (placements, ELO changes, XP). |
| `test_reconnection_grace_period` | Disconnect a player's WebSocket. Reconnect within 10 seconds. Assert the player is still alive in the match. | Player survives disconnection. |
| `test_reconnection_after_grace_period` | Disconnect and wait 11 seconds. Assert player is eliminated. | Eliminated after grace period. |
| `test_concurrent_answer_submission` | All 16 players submit answers within the same 100ms window. Assert no race conditions — each answer is processed exactly once, standings are consistent. | No duplicate `player_solved` events, correct round progression. |
| `test_forfeit` | Player sends `forfeit` message. Assert elimination and correct broadcast. | Clean elimination, match continues for remaining players. |
| `test_server_side_wasm_validation` | Submit a correct answer and a wrong answer. Assert the DO's embedded WASM module correctly validates/rejects each. | Matches client-side validation results for same seed. |
| `test_max_players_16` | Connect 16 players to a single room. Run 5 rounds. Assert performance is acceptable (round transitions < 500ms). | Completes without timeout or message loss. |

#### Matchmaker DO

| Test | Description | Pass Criteria |
|---|---|---|
| `test_queue_join` | Connect a player to the matchmaker WebSocket. Assert they are added to the correct ELO bracket queue. | Player appears in the expected bracket. |
| `test_match_creation_at_min_players` | Add 4 players to the same bracket. Trigger alarm tick. Assert a `match_found` message is sent to all 4 with the same `roomId`. | All 4 receive `match_found`. |
| `test_match_creation_at_max_players` | Add 20 players to the same bracket. Trigger alarm tick. Assert 16 are matched (one room) and 4 remain in queue. | 16 matched, 4 still queued. |
| `test_bracket_assignment` | Add players with ELO 500, 900, 1100, 1400, 1600. Assert each lands in the correct bracket (bronze, silver, gold, platinum, diamond). | Correct bracket for each. |
| `test_bracket_expansion_30s` | Add a player and wait 30 simulated seconds. Assert search expands to ±1 adjacent bracket. | Player's search range includes adjacent brackets. |
| `test_bracket_expansion_60s` | Wait 60 simulated seconds. Assert ±2 bracket expansion. | Expanded search range. |
| `test_no_cross_bracket_match_beyond_2` | Add one player in bronze (ELO 500) and one in diamond (ELO 1600). Wait 120 seconds. Assert they are never matched. | No match created between 3+ bracket gap. |
| `test_queue_removal_on_disconnect` | Player disconnects from matchmaker WebSocket. Assert they are removed from the queue. | Queue size decrements by 1. |
| `test_alarm_reschedule` | Assert alarm fires every 1 second while queue is non-empty, stops when queue empties. | Correct alarm behavior. |

#### Auth Workers

| Test | Description | Pass Criteria |
|---|---|---|
| `test_oauth_redirect_google` | GET `/api/auth/google`. Assert 302 redirect to Google OAuth URL with correct scopes, state param, and redirect_uri. | Valid OAuth redirect. |
| `test_oauth_redirect_discord` | Same for Discord. | Valid OAuth redirect. |
| `test_oauth_redirect_github` | Same for GitHub. | Valid OAuth redirect. |
| `test_oauth_callback_valid_code` | Mock provider token exchange. POST callback with valid code + state. Assert: session cookie is set, player record created in D1, KV session entry exists. | Full auth flow completes. |
| `test_oauth_callback_invalid_state` | POST callback with tampered state param. Assert 403. | Rejected. |
| `test_oauth_callback_expired_state` | POST callback after state TTL (5 min). Assert 403. | Rejected. |
| `test_session_validation` | Set a session cookie. GET `/api/auth/me`. Assert player profile is returned. | Valid session returns profile. |
| `test_session_expiry` | Set a session with 1-second TTL. Wait 2 seconds. GET `/api/auth/me`. Assert 401. | Expired session rejected. |
| `test_logout` | POST `/api/auth/logout`. Assert cookie is cleared and KV entry is deleted. | Clean logout. |
| `test_multiple_providers_same_player` | Auth via Google, then auth via Discord with the same email. Assert both `oauth_identities` rows point to the same `player_id`. | Account linking works. |

#### API Routes

| Test | Description | Pass Criteria |
|---|---|---|
| `test_get_profile` | GET `/api/profile/:id`. Assert correct fields returned (display_name, elo, level, xp, matches_played, wins). | Correct JSON response. |
| `test_get_profile_404` | GET `/api/profile/nonexistent`. Assert 404. | 404. |
| `test_patch_profile` | PATCH `/api/profile` with new display_name. Assert updated in D1. | Name persists. |
| `test_patch_profile_unauthenticated` | PATCH without session cookie. Assert 401. | Rejected. |
| `test_match_history_pagination` | Create 25 match results. GET `/api/profile/:id/history?page=1&limit=10`. Assert 10 results returned with correct pagination metadata. | Correct page. |
| `test_leaderboard` | Populate 150 players with various ELOs. GET `/api/leaderboard`. Assert top 100 returned, sorted by ELO descending. | Correct ordering and count. |
| `test_leaderboard_around` | GET `/api/leaderboard/around/:id`. Assert the target player is centered in the result with 5 above and 5 below. | Correct windowed result. |
| `test_private_room_create_join` | POST `/api/match/private`. Assert room code returned. POST `/api/match/join/:code`. Assert player connects to the correct room. | Room code round-trips. |
| `test_private_room_invalid_code` | POST `/api/match/join/XXXXXX`. Assert 404. | Rejected. |

### 15.3 Frontend (Vitest + React Testing Library + Playwright)

#### Component Tests (Vitest + RTL)

| Test | Description | Pass Criteria |
|---|---|---|
| `test_desktop_gate_blocks_mobile` | Render `App` with `window.innerWidth = 768`. Assert "desktop only" gate is visible, game content is not. | Gate shown. |
| `test_desktop_gate_passes_desktop` | Render with `window.innerWidth = 1280`. Assert game content is visible. | No gate. |
| `test_login_page_shows_providers` | Render `Login`. Assert Google, Discord, and GitHub buttons are visible. | All 3 buttons present. |
| `test_captcha_renderer_dispatches` | For each `CaptchaType`, render `CaptchaRenderer` with a mock instance. Assert the correct sub-component is mounted. | Correct component for each type. |
| `test_timer_counts_down` | Render `Timer` with 10,000ms. Advance clock by 5,000ms. Assert display shows ~5s remaining. | Correct countdown. |
| `test_timer_triggers_timeout` | Advance clock past the limit. Assert `onTimeout` callback fires. | Callback invoked. |
| `test_player_list_shows_alive_eliminated` | Render `PlayerList` with 4 alive and 2 eliminated players. Assert alive players have active styling, eliminated have struck-through or dimmed styling. | Correct visual states. |
| `test_elimination_feed_animates` | Push a `player_eliminated` event. Assert the `EliminationFeed` shows the player name with the correct reason. | Feed entry appears. |
| `test_results_screen_shows_elo_change` | Render `Results` with mock `FinalStanding` data including positive and negative ELO changes. Assert "+32" and "-18" are displayed correctly. | Correct signed display. |
| `test_queue_screen_shows_estimated_wait` | Render `Queue` with 3 players in bracket. Assert estimated wait time is displayed. | Wait time shown. |
| `test_match_hud_round_indicator` | Render match HUD at round 5 of a match. Assert "Round 5" is displayed. | Correct round number. |

#### WASM Integration Tests

| Test | Description | Pass Criteria |
|---|---|---|
| `test_wasm_module_loads` | Call `useCaptchaEngine()` hook. Assert the WASM module initializes without error. | Module ready. |
| `test_wasm_generates_captcha` | Call `generate_captcha(seed, type, difficulty)` via the JS bindings. Assert a valid `CaptchaInstance` is returned with renderable data. | Non-null, parseable result. |
| `test_wasm_validates_correct_answer` | Generate + validate with correct answer via JS bindings. | Returns `true`. |
| `test_wasm_rejects_wrong_answer` | Generate + validate with wrong answer via JS bindings. | Returns `false`. |
| `test_wasm_performance` | Time 100 consecutive `generate_captcha` calls. Assert p99 < 50ms per call. | Meets performance budget. |

#### WebSocket Hook Tests

| Test | Description | Pass Criteria |
|---|---|---|
| `test_use_websocket_connects` | Mock WebSocket server. Call `useWebSocket(url)`. Assert connection opens and `readyState` is OPEN. | Connected. |
| `test_use_websocket_reconnects` | Close the mock server. Assert the hook attempts reconnection with exponential backoff up to 3 retries. | Reconnection attempts observed. |
| `test_use_match_state_machine` | Feed mock server messages in sequence: `lobby_update` → `round_start` → `round_end` → `match_end`. Assert `useMatchState()` transitions through `lobby` → `playing` → `between_rounds` → `results`. | Correct state transitions. |

### 15.4 End-to-End Tests (Playwright)

Full browser tests against a local Miniflare backend + real WASM build.

| Test | Description | Pass Criteria |
|---|---|---|
| `e2e_full_auth_flow` | Click "Login with Google" → mock OAuth redirect → callback → assert profile page loads with display name. | Authenticated and profile visible. |
| `e2e_solo_endless_mode` | Start an Endless game. Solve 5 CAPTCHAs by interacting with the rendered UI (click grid squares, type text). Assert score increments. Deliberately fail one. Assert game over screen. | Full solo loop works. |
| `e2e_two_player_match` | Open 2 browser contexts. Both log in, both queue for matchmaking. Assert both are placed in the same room. Both solve round 1. One submits a wrong answer in round 2. Assert the correct player wins. | Full multiplayer match. |
| `e2e_16_player_match` | Open 16 browser contexts (headless). Queue all. Assert match creation. Run 5 rounds with scripted answers (some correct, some wrong). Assert final standings match expected eliminations. | Large lobby functions correctly. |
| `e2e_private_room` | Player A creates a private room, copies the code. Player B joins via the code. Assert both are in the same room and can play. | Private rooms work. |
| `e2e_leaderboard_updates` | Two players complete a ranked match. Navigate to leaderboard. Assert both players appear with updated ELO. | Leaderboard reflects match results. |
| `e2e_achievement_unlock` | A player wins their first match. Assert an achievement notification appears and the achievement is visible on their profile. | Achievement system works. |
| `e2e_disconnection_recovery` | Mid-match, forcefully close one player's WebSocket (via `page.evaluate`). Reopen the page within 10 seconds. Assert the player reconnects and is still alive in the match. | Reconnection grace period works. |

### 15.5 Load & Stress Testing (k6 or custom Rust harness)

These tests validate behavior at scale and identify breaking points. Run on a staging Cloudflare deployment (still free tier) with synthetic load.

| Test | Description | Pass Criteria | Tool |
|---|---|---|---|
| `load_concurrent_matches` | Simulate 50 concurrent matches (800 WebSocket connections). Measure message latency and DO response times. | p99 message latency < 200ms. No dropped messages. | k6 WebSocket |
| `load_matchmaker_throughput` | Queue 200 players in 10 seconds across all brackets. Measure time to create matches and notify players. | All players matched within 30 seconds. No orphaned queue entries. | k6 WebSocket |
| `load_d1_write_burst` | Simulate 50 matches ending simultaneously (50 `matches` rows + 800 `match_results` rows). Measure D1 write latency. | All writes complete within 5 seconds. No D1 errors. | k6 HTTP |
| `load_auth_burst` | Simulate 100 simultaneous OAuth callback requests. Measure session creation latency. | p99 < 500ms. No KV write failures. | k6 HTTP |
| `stress_single_room_max_players` | Connect 20 players to a single MatchRoom DO (exceeding the 16-player design max). Assert the room rejects connections beyond 16 with a clear error. | 17th connection rejected gracefully. | Custom script |
| `stress_rapid_answer_submission` | 16 players submit answers as fast as possible (< 10ms between submissions). Assert no race conditions or duplicate processing. | Exactly 16 `player_solved` or `player_eliminated` events per round. | Custom script |
| `stress_websocket_reconnect_storm` | Disconnect and reconnect all 16 players in a match simultaneously. Assert the DO handles the reconnection storm without crashing. | All players reconnect or are gracefully eliminated. | Custom script |
| `stress_free_tier_budget` | Run a simulated "busy day" workload (100 matches, 500 unique players, 2,000 API requests). Log total Workers requests, DO requests, D1 reads/writes, KV reads/writes. Assert all stay within free tier limits. | Under Cloudflare free tier quotas. | k6 + monitoring |

### 15.6 Security Tests

| Test | Description | Pass Criteria |
|---|---|---|
| `sec_forged_session_cookie` | Submit requests with a manually constructed session token not present in KV. | 401 on all protected routes. |
| `sec_replay_oauth_state` | Reuse a consumed OAuth state parameter. | 403 — state is single-use. |
| `sec_websocket_without_auth` | Attempt WebSocket upgrade to matchmaker and match room without a session cookie. | Connection rejected. |
| `sec_answer_for_wrong_round` | Submit an answer for round N+1 while the match is on round N. | Answer rejected, player not penalized. |
| `sec_answer_after_elimination` | An eliminated player submits an answer. | Answer ignored, no effect on match state. |
| `sec_cross_room_message` | Player in Room A sends a message intended for Room B (via tampered roomId). | Message is scoped to the DO — impossible to cross rooms by design. Verify no effect. |
| `sec_d1_sql_injection` | Submit display names and other user input containing SQL injection payloads. | D1 parameterized queries prevent injection. No errors, no data leaks. |
| `sec_xss_display_name` | Set display_name to `<script>alert(1)</script>`. Assert it is sanitized or escaped in all rendered contexts. | No script execution. |
| `sec_rate_limit_answer_spam` | Submit 100 answers per second from a single client. | Answers beyond 1 per round are rejected. No server degradation. |
| `sec_wasm_binary_tampering` | Serve a modified WASM binary (with altered generation logic). Submit answers. | Server-side validation rejects all answers generated by the tampered binary (since server uses its own WASM copy). |

### 15.7 CAPTCHA Balance & Telemetry Tests

These are not automated pass/fail tests — they are instrumented playtesting sessions that produce data for tuning.

| Test | Description | Output |
|---|---|---|
| `balance_solve_time_distribution` | 20 human playtesters solve 50 CAPTCHAs per type at each difficulty level. Record solve times. | Histogram of solve times per (type, difficulty). Compare against Section 4.4 estimates. |
| `balance_accuracy_by_tier` | Same playtest sessions. Record accuracy rates. | Accuracy should be >95% at Tier 1, 80–90% at Tier 2, 60–80% at Tier 3, 40–60% at Tier 4. If outside these bands, adjust difficulty knobs. |
| `balance_elimination_rate` | Run 50 simulated Battle Royale matches with human playtesters. Record rounds survived per player. | Median match should last 8–12 rounds. If matches are too short (< 5 rounds) or too long (> 20 rounds), adjust difficulty ramp. |
| `balance_elo_convergence` | Simulate 1,000 matches with players of known skill levels. Assert ELO converges to a stable ranking within 30 matches per player. | ELO reflects true skill after convergence period. |
| `balance_matchmaker_wait_times` | Log queue wait times across all brackets during a playtest session (20+ concurrent players). | Median wait < 15 seconds. p95 wait < 45 seconds. |
| `balance_adversarial_human_solvability` | Present adversarial CAPTCHAs to 20 humans. Record accuracy. | Humans should achieve >70% accuracy at all difficulty levels. If below, reduce perturbation magnitude. |

### 15.8 CI/CD Integration

```
┌─────────────────────────────────────────────────────────────┐
│ CI Pipeline (GitHub Actions)                                │
│                                                             │
│  On every PR:                                               │
│   1. cargo test          (Engine unit + fuzz 1K iterations) │
│   2. cargo clippy        (Lint)                             │
│   3. wasm-pack build     (Verify WASM compilation)          │
│   4. wasm-pack test      (Cross-browser determinism)        │
│   5. vitest run          (Frontend components + hooks)      │
│   6. miniflare test      (DO + Worker integration)          │
│   7. eslint + tsc        (Frontend lint + type check)       │
│                                                             │
│  On merge to main:                                          │
│   8. All of the above                                       │
│   9. playwright test     (E2E against staging)              │
│  10. wasm-opt -Oz        (Optimize WASM binary)             │
│  11. Deploy to CF Pages + Workers (staging)                 │
│                                                             │
│  Nightly:                                                   │
│  12. cargo-fuzz          (10M iterations)                   │
│  13. k6 load tests       (Against staging deployment)       │
│                                                             │
│  Weekly:                                                    │
│  14. Full security test suite                               │
│  15. WASM bundle size report (alert if > 8 MB)             │
│                                                             │
│  On release tag:                                            │
│  16. All of the above                                       │
│  17. Deploy to production                                   │
│  18. Smoke test production endpoints                        │
└─────────────────────────────────────────────────────────────┘
```

### 15.9 Test File Structure

```
captcha-royale/
├── packages/
│   └── captcha-engine/
│       ├── tests/
│       │   ├── determinism.rs
│       │   ├── generators/
│       │   │   ├── text_tests.rs
│       │   │   ├── grid_tests.rs
│       │   │   ├── math_tests.rs
│       │   │   ├── slider_tests.rs
│       │   │   ├── rotation_tests.rs
│       │   │   ├── occlusion_tests.rs
│       │   │   ├── oddity_tests.rs
│       │   │   ├── rhythm_tests.rs
│       │   │   ├── color_tests.rs
│       │   │   ├── adversarial_tests.rs
│       │   │   ├── sequence_tests.rs
│       │   │   ├── multistep_tests.rs
│       │   │   ├── spatial_tests.rs
│       │   │   ├── contextual_tests.rs
│       │   │   ├── metamorphic_tests.rs
│       │   │   ├── combined_tests.rs
│       │   │   ├── typography_tests.rs
│       │   │   ├── novel_tests.rs
│       │   │   └── cascade_tests.rs
│       │   ├── adversarial_library_tests.rs
│       │   └── scoring_tests.rs
│       └── fuzz/
│           ├── fuzz_generate.rs
│           └── fuzz_validate.rs
├── apps/
│   ├── web/
│   │   ├── src/__tests__/
│   │   │   ├── components/
│   │   │   │   ├── CaptchaRenderer.test.tsx
│   │   │   │   ├── Timer.test.tsx
│   │   │   │   ├── PlayerList.test.tsx
│   │   │   │   ├── EliminationFeed.test.tsx
│   │   │   │   ├── Results.test.tsx
│   │   │   │   └── DesktopGate.test.tsx
│   │   │   ├── hooks/
│   │   │   │   ├── useWebSocket.test.ts
│   │   │   │   ├── useMatchState.test.ts
│   │   │   │   ├── useCaptchaEngine.test.ts
│   │   │   │   └── useAuth.test.ts
│   │   │   └── wasm/
│   │   │       ├── wasm-load.test.ts
│   │   │       ├── wasm-generate.test.ts
│   │   │       └── wasm-performance.test.ts
│   │   └── e2e/
│   │       ├── auth.spec.ts
│   │       ├── solo-endless.spec.ts
│   │       ├── two-player-match.spec.ts
│   │       ├── sixteen-player-match.spec.ts
│   │       ├── private-room.spec.ts
│   │       ├── leaderboard.spec.ts
│   │       ├── achievement.spec.ts
│   │       └── disconnection.spec.ts
│   └── worker/
│       ├── tests/
│       │   ├── match-room.test.ts
│       │   ├── matchmaker.test.ts
│       │   ├── auth.test.ts
│       │   ├── api-profile.test.ts
│       │   ├── api-leaderboard.test.ts
│       │   ├── api-match.test.ts
│       │   └── security.test.ts
│       └── load/
│           ├── concurrent-matches.js      # k6 script
│           ├── matchmaker-throughput.js    # k6 script
│           ├── d1-write-burst.js          # k6 script
│           ├── auth-burst.js              # k6 script
│           └── free-tier-budget.js        # k6 script
└── .github/
    └── workflows/
        ├── ci.yml                         # PR + merge pipeline
        ├── nightly.yml                    # Fuzz + load tests
        ├── weekly-security.yml            # Security suite
        └── release.yml                    # Production deploy
```

---

## 16. Resolved Decisions

| Decision | Resolution |
|---|---|
| **WASM in Durable Objects** | Confirmed. The Rust CAPTCHA engine compiles to WASM and runs inside Durable Objects for server-side validation. The same binary is used client-side (generation) and server-side (validation). Keep the WASM binary lean — strip debug symbols, use `wasm-opt -Oz`, target the 10 MB Worker bundle limit. If binary size becomes an issue, split into a validation-only WASM (smaller) for the server and a full generation WASM for the client. |
| **Audio CAPTCHAs** | Limited to tone/rhythm patterns only. No speech synthesis. Challenges include: identify the odd rhythm in a sequence, count beats in a pattern, match a tone sequence from memory, identify the missing note in a scale fragment. Generated via Web Audio API oscillators and gain nodes. |
| **Cloudflare free tier** | Sufficient for target scale. No paid tier planning needed at this stage. |
| **CAPTCHA solve times** | Estimated per type and difficulty tier (see Section 4.4 below). Will be refined via playtesting telemetry. |
| **Adversarial CAPTCHA generation** | Uses a bundled library of known adversarial patterns (static adversarial perturbation templates, pre-computed noise masks, known classifier failure modes). The generator applies these patterns procedurally to seed-generated base images rather than running a classifier in-loop. See Section 4.5. |
| **Platform** | Desktop only. The frontend enforces a minimum viewport width (1024px) and shows a "desktop only" gate on smaller screens. No touch input support, no mobile-responsive layouts. This simplifies CAPTCHA interaction design (no tap-vs-click ambiguity, no on-screen keyboard timing penalties) and reduces the testing surface. Mobile support is a future consideration. |

## 17. Remaining Risks

| Risk | Mitigation |
|---|---|
| **WASM bundle size** | Profile early. The validation-only path is the escape hatch. |
| **Adversarial pattern library staleness** | Patterns that fool 2024 classifiers may not fool 2026 classifiers. Plan for periodic library updates as a content patch. |
| **CAPTCHA balance** | Solve-time estimates are educated guesses. Instrument everything from day one — log solve times, accuracy rates, and elimination rates per type/difficulty. Adjust curves monthly. |
| **WebSocket reliability** | Players on flaky connections will drop. Implement a 10-second reconnection grace period in MatchRoom DOs before marking a player as eliminated. |
| **Cheating via browser devtools** | WASM obfuscation raises the bar but isn't bulletproof. Server-side validation is the real defense. Statistical detection catches the rest. |
