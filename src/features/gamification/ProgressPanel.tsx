import type {
  AchievementStatus,
  DailyObjective,
  PetStats,
  ProductivityBreakdown,
  WeeklySummary,
} from "./gamification.types";
import { formatObjectiveProgress } from "./useGamification";

type ProgressPanelProps = {
  stats: PetStats | null;
  objectives: DailyObjective[];
  achievements: AchievementStatus[];
  weeklySummary: WeeklySummary | null;
  breakdown: ProductivityBreakdown | null;
  loading: boolean;
  error?: string | null;
  onRetry?: () => void;
};

export function ProgressPanel({
  stats,
  objectives,
  achievements,
  weeklySummary,
  breakdown,
  loading,
  error,
  onRetry,
}: ProgressPanelProps) {
  if (loading && !stats) {
    return <p className="settings-hint">Loading progress...</p>;
  }

  if (!stats) {
    return (
      <div className="progress-panel-empty">
        <p className="settings-hint">
          {error
            ? `Could not load progress data: ${error}`
            : "Progress data is unavailable."}
        </p>
        {onRetry ? (
          <button type="button" className="settings-secondary" onClick={onRetry}>
            Retry
          </button>
        ) : null}
      </div>
    );
  }

  const xpPercent =
    stats.xpForNextLevel > 0
      ? Math.min(100, Math.round((stats.xpIntoLevel / stats.xpForNextLevel) * 100))
      : 0;

  return (
    <section className="progress-panel">
      <h2>🏆 Progress</h2>

      <div className="progress-level-row">
        <div className="progress-level-badge">{stats.level}</div>
        <div className="progress-level-info">
          <div className="progress-level-name">{stats.levelName}</div>
          <div className="progress-xp-bar-bg">
            <div className="progress-xp-bar-fill" style={{ width: `${xpPercent}%` }} />
          </div>
          <div className="progress-xp-label">
            {stats.xpIntoLevel.toLocaleString()} / {stats.xpForNextLevel.toLocaleString()} XP to
            Level {stats.level + 1}
          </div>
        </div>
      </div>

      <div className="progress-streak">
        <div className="progress-streak-num">🔥 {stats.streak}</div>
        <div>
          <div className="progress-streak-title">Day Streak</div>
          <div className="progress-streak-subtitle">Keep it up!</div>
        </div>
      </div>

      <div className="progress-objectives">
        <div className="progress-section-label">Today&apos;s Objectives</div>
        {objectives.map((objective) => (
          <div className="progress-objective" key={objective.id}>
            <div
              className={`progress-objective-check${objective.completed ? " progress-objective-check--done" : ""}`}
            />
            <span>{formatObjectiveProgress(objective)}</span>
          </div>
        ))}
      </div>

      <div className="progress-score-row">
        <div>
          <div className="progress-section-label">Today&apos;s Score</div>
          <div className="progress-score-num">{stats.productivityScore}</div>
        </div>
        {breakdown ? (
          <div className="progress-score-breakdown">
            Coding: {breakdown.coding}/40
            <br />
            Breaks: {breakdown.breaks}/30
            <br />
            Hydration: {breakdown.hydration}/20
            <br />
            Goals: {breakdown.goals}/10
          </div>
        ) : null}
      </div>

      {weeklySummary ? (
        <div className="progress-weekly">
          <div className="progress-section-label">This Week</div>
          <p className="progress-weekly-copy">
            {weeklySummary.codingMinutes} min coded · avg score {weeklySummary.score} ·{" "}
            {weeklySummary.objectivesCompleted} objectives · {weeklySummary.achievementsUnlocked}{" "}
            achievements
          </p>
        </div>
      ) : null}

      <div className="progress-achievements">
        <div className="progress-section-label">Achievements</div>
        <div className="progress-achievements-grid">
          {achievements.map((achievement) => (
            <div
              key={achievement.id}
              className={`progress-achievement${achievement.unlocked ? "" : " progress-achievement--locked"}`}
              title={`${achievement.name}: ${achievement.description}`}
            >
              {achievement.icon}
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
