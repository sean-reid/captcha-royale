export function getEloBracket(elo: number): string {
  if (elo < 800) return 'Bronze';
  if (elo < 1000) return 'Silver';
  if (elo < 1200) return 'Gold';
  if (elo < 1500) return 'Platinum';
  return 'Diamond';
}

export function getBracketColor(elo: number): string {
  if (elo < 800) return '#cd7f32';
  if (elo < 1000) return '#c0c0c0';
  if (elo < 1200) return '#ffd700';
  if (elo < 1500) return '#e5e4e2';
  return '#b9f2ff';
}

const BRACKET_COLORS: Record<string, string> = {
  bronze: '#cd7f32',
  silver: '#c0c0c0',
  gold: '#ffd700',
  platinum: '#e5e4e2',
  diamond: '#b9f2ff',
};

export function getBracketColorByName(name: string): string {
  return BRACKET_COLORS[name.toLowerCase()] || '#888';
}

export function xpForLevel(level: number): number {
  return Math.floor(100 * Math.pow(level, 1.5));
}

export function levelFromXp(xp: number): number {
  let level = 1;
  while (xpForLevel(level + 1) <= xp) {
    level++;
  }
  return level;
}
