export interface EloResult {
  playerId: string;
  eloBefore: number;
  eloAfter: number;
  delta: number;
}

export function calculateMultiplayerElo(
  placements: Array<{ playerId: string; elo: number; placement: number }>,
  matchesPlayed: Map<string, number>,
): EloResult[] {
  const n = placements.length;
  const results: EloResult[] = [];

  for (const player of placements) {
    let totalDelta = 0;
    const k = getKFactor(matchesPlayed.get(player.playerId) ?? 0);

    for (const opponent of placements) {
      if (player.playerId === opponent.playerId) continue;

      const expected = 1 / (1 + Math.pow(10, (opponent.elo - player.elo) / 400));
      const actual =
        player.placement < opponent.placement
          ? 1
          : player.placement === opponent.placement
            ? 0.5
            : 0;

      totalDelta += k * (actual - expected);
    }

    // Normalize by number of opponents
    const normalizedDelta = Math.round(totalDelta / (n - 1));

    results.push({
      playerId: player.playerId,
      eloBefore: player.elo,
      eloAfter: player.elo + normalizedDelta,
      delta: normalizedDelta,
    });
  }

  return results;
}

function getKFactor(matchesPlayed: number): number {
  if (matchesPlayed < 30) return 40;
  if (matchesPlayed < 100) return 24;
  return 16;
}
