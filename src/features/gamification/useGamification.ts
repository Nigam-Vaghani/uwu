import { useCallback, useEffect, useState } from "react";
import { invokeCommand } from "../../hooks/useTauriStore";
import type {
  AchievementStatus,
  DailyObjective,
  GamificationState,
  PetStats,
  ProductivityBreakdown,
  WeeklySummary,
} from "./gamification.types";

const initialState: GamificationState = {
  stats: null,
  objectives: [],
  achievements: [],
  weeklySummary: null,
  breakdown: null,
  loading: true,
};

function formatInvokeError(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}

export function useGamification(autoRefreshMs = 30_000) {
  const [state, setState] = useState<GamificationState>(initialState);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setState((current) => ({ ...current, loading: true }));
    setError(null);

    const results = await Promise.allSettled([
      invokeCommand("get_pet_stats", {}),
      invokeCommand("get_daily_objectives", {}),
      invokeCommand("get_achievements", {}),
      invokeCommand("get_weekly_summary", {}),
      invokeCommand("get_productivity_breakdown", {}),
    ]);

    const failures = results
      .map((result, index) =>
        result.status === "rejected"
          ? `${["stats", "objectives", "achievements", "weekly summary", "breakdown"][index]}: ${formatInvokeError(result.reason)}`
          : null,
      )
      .filter((message): message is string => message != null);

    const [statsResult, objectivesResult, achievementsResult, weeklySummaryResult, breakdownResult] =
      results;

    setState({
      stats: statsResult.status === "fulfilled" ? (statsResult.value as PetStats) : null,
      objectives:
        objectivesResult.status === "fulfilled" ? (objectivesResult.value as DailyObjective[]) : [],
      achievements:
        achievementsResult.status === "fulfilled"
          ? (achievementsResult.value as AchievementStatus[])
          : [],
      weeklySummary:
        weeklySummaryResult.status === "fulfilled"
          ? (weeklySummaryResult.value as WeeklySummary)
          : null,
      breakdown:
        breakdownResult.status === "fulfilled"
          ? (breakdownResult.value as ProductivityBreakdown)
          : null,
      loading: false,
    });

    if (failures.length > 0) {
      setError(failures.join(" · "));
    }
  }, []);

  useEffect(() => {
    void refresh();
    const timer = window.setInterval(() => {
      void refresh();
    }, autoRefreshMs);
    return () => window.clearInterval(timer);
  }, [autoRefreshMs, refresh]);

  return { ...state, error, refresh };
}

export function formatObjectiveProgress(objective: DailyObjective) {
  if (objective.completed) {
    return objective.title;
  }

  if (objective.category === "code_2h") {
    const minutes = Math.floor(objective.progress / 60);
    const targetMinutes = Math.floor(objective.target / 60);
    return `${objective.title} (${minutes}/${targetMinutes} min)`;
  }

  return `${objective.title} (${objective.progress}/${objective.target})`;
}
