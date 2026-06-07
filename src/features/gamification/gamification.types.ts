export type PetStats = {
  xp: number;
  level: number;
  streak: number;
  productivityScore: number;
  xpIntoLevel: number;
  xpForNextLevel: number;
  levelName: string;
};

export type DailyObjective = {
  id: number;
  date: string;
  title: string;
  category: string;
  target: number;
  progress: number;
  completed: boolean;
};

export type AchievementStatus = {
  id: string;
  name: string;
  description: string;
  icon: string;
  unlocked: boolean;
  unlockedAt: number | null;
};

export type WeeklySummary = {
  weekStart: string;
  codingMinutes: number;
  score: number;
  objectivesCompleted: number;
  achievementsUnlocked: number;
};

export type ProductivityBreakdown = {
  coding: number;
  breaks: number;
  hydration: number;
  goals: number;
};

export type GamificationState = {
  stats: PetStats | null;
  objectives: DailyObjective[];
  achievements: AchievementStatus[];
  weeklySummary: WeeklySummary | null;
  breakdown: ProductivityBreakdown | null;
  loading: boolean;
};

export type AchievementToastPayload = {
  id: string;
  name: string;
  icon: string;
};
