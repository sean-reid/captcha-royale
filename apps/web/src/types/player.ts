export interface PlayerInfo {
  id: string;
  display_name: string;
  avatar_url: string | null;
  elo: number;
  level: number;
}

export interface PlayerProfile extends PlayerInfo {
  xp: number;
  matches_played: number;
  wins: number;
  created_at: string;
}

export interface Achievement {
  id: string;
  name: string;
  description: string;
  icon: string | null;
  unlocked_at?: string;
}
