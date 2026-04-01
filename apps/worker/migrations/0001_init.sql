-- Players
CREATE TABLE players (
  id            TEXT PRIMARY KEY,
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
  provider      TEXT NOT NULL,
  provider_id   TEXT NOT NULL,
  player_id     TEXT NOT NULL REFERENCES players(id),
  email         TEXT,
  created_at    TEXT NOT NULL DEFAULT (datetime('now')),
  PRIMARY KEY (provider, provider_id)
);
CREATE INDEX idx_oauth_player ON oauth_identities(player_id);
CREATE INDEX idx_oauth_email ON oauth_identities(email);

-- Match history
CREATE TABLE matches (
  id            TEXT PRIMARY KEY,
  mode          TEXT NOT NULL,
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
  placement     INTEGER NOT NULL,
  elo_before    INTEGER NOT NULL,
  elo_after     INTEGER NOT NULL,
  xp_earned     INTEGER NOT NULL,
  rounds_survived INTEGER NOT NULL,
  avg_solve_ms  INTEGER,
  PRIMARY KEY (match_id, player_id)
);
CREATE INDEX idx_match_results_player ON match_results(player_id);

-- Achievements
CREATE TABLE achievements (
  id            TEXT PRIMARY KEY,
  name          TEXT NOT NULL,
  description   TEXT NOT NULL,
  icon          TEXT
);

CREATE TABLE player_achievements (
  player_id     TEXT NOT NULL REFERENCES players(id),
  achievement_id TEXT NOT NULL REFERENCES achievements(id),
  unlocked_at   TEXT NOT NULL DEFAULT (datetime('now')),
  PRIMARY KEY (player_id, achievement_id)
);

-- Leaderboard snapshots
CREATE TABLE leaderboard_snapshots (
  season        TEXT NOT NULL,
  player_id     TEXT NOT NULL REFERENCES players(id),
  elo           INTEGER NOT NULL,
  rank          INTEGER NOT NULL,
  snapshot_at   TEXT NOT NULL DEFAULT (datetime('now')),
  PRIMARY KEY (season, player_id)
);

-- Seed initial achievements
INSERT INTO achievements (id, name, description, icon) VALUES
  ('first_win', 'First Blood', 'Win your first match', '🏆'),
  ('solve_100', 'Century Solver', 'Solve 100 CAPTCHAs', '💯'),
  ('solve_100_under_2s', 'Speed Demon', 'Solve 100 CAPTCHAs under 2 seconds', '⚡'),
  ('win_16_player', 'Battle Royale Champion', 'Win a 16-player lobby', '👑'),
  ('endless_50', 'Endurance Runner', 'Survive 50 rounds in Endless mode', '🏃'),
  ('tier3_solve', 'Big Brain', 'Solve a Tier 3 CAPTCHA correctly', '🧠'),
  ('tier4_solve', 'Nightmare Slayer', 'Solve a Tier 4 CAPTCHA correctly', '😈'),
  ('win_streak_5', 'On Fire', 'Win 5 matches in a row', '🔥'),
  ('elo_1500', 'Platinum Player', 'Reach 1500 ELO', '💎'),
  ('elo_2000', 'Grandmaster', 'Reach 2000 ELO', '⭐');
