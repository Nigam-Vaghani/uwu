use super::types::{ProductivityBreakdown, WeeklySummary};
use crate::features::database::with_db;
use rusqlite::{params, Connection};
use tauri::AppHandle;

pub fn recalculate_score(connection: &Connection) -> Result<i64, String> {
    let breakdown = super::objectives::get_productivity_breakdown(connection)?;
    let score = breakdown.coding + breakdown.breaks + breakdown.hydration + breakdown.goals;
    let today = super::types::today_date();
    super::objectives::ensure_daily_activity(connection)?;
    connection
        .execute(
            "UPDATE daily_activity SET productivity_score = ?1 WHERE date = ?2",
            params![score, today],
        )
        .map_err(|error| error.to_string())?;
    Ok(score)
}

pub fn get_productivity_breakdown_command(
    app: AppHandle,
) -> Result<ProductivityBreakdown, String> {
    with_db(&app, super::objectives::get_productivity_breakdown)
}

pub fn generate_weekly_summary(connection: &Connection, week_start: &str) -> Result<(), String> {
    let coding_secs: i64 = connection
        .query_row(
            "SELECT COALESCE(SUM(coding_secs), 0)
             FROM daily_activity
             WHERE date >= ?1 AND date < date(?1, '+7 days')",
            [week_start],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;

    let score: i64 = connection
        .query_row(
            "SELECT CAST(COALESCE(AVG(productivity_score), 0) AS INTEGER)
             FROM daily_activity
             WHERE date >= ?1 AND date < date(?1, '+7 days')",
            [week_start],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;

    let objectives_completed: i64 = connection
        .query_row(
            "SELECT COALESCE(SUM(CASE WHEN completed = 1 THEN 1 ELSE 0 END), 0)
             FROM daily_objectives
             WHERE date >= ?1 AND date < date(?1, '+7 days')",
            [week_start],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;

    let achievements_unlocked: i64 = connection
        .query_row(
            "SELECT COUNT(*)
             FROM achievements
             WHERE unlocked_at >= strftime('%s', ?1)
               AND unlocked_at < strftime('%s', date(?1, '+7 days'))",
            [week_start],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;

    connection
        .execute(
            "INSERT INTO weekly_summaries (week_start, coding_minutes, score, objectives_completed, achievements_unlocked)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(week_start) DO UPDATE SET
               coding_minutes = excluded.coding_minutes,
               score = excluded.score,
               objectives_completed = excluded.objectives_completed,
               achievements_unlocked = excluded.achievements_unlocked",
            params![
                week_start,
                coding_secs / 60,
                score,
                objectives_completed,
                achievements_unlocked
            ],
        )
        .map_err(|error| error.to_string())?;

    Ok(())
}

pub fn get_weekly_summary(connection: &Connection) -> Result<WeeklySummary, String> {
    let week_start = super::types::week_start_date();
    generate_weekly_summary(connection, &week_start)?;

    connection
        .query_row(
            "SELECT week_start, coding_minutes, score, objectives_completed, achievements_unlocked
             FROM weekly_summaries WHERE week_start = ?1",
            [week_start],
            |row| {
                Ok(WeeklySummary {
                    week_start: row.get(0)?,
                    coding_minutes: row.get(1)?,
                    score: row.get(2)?,
                    objectives_completed: row.get(3)?,
                    achievements_unlocked: row.get(4)?,
                })
            },
        )
        .map_err(|error| error.to_string())
}

pub fn get_weekly_summary_command(app: AppHandle) -> Result<WeeklySummary, String> {
    with_db(&app, get_weekly_summary)
}

pub fn maybe_finalize_previous_week(connection: &Connection) -> Result<(), String> {
    use chrono::{Datelike, Duration, Local};
    let today = Local::now().date_naive();
    if today.weekday().num_days_from_monday() != 0 {
        return Ok(());
    }

    let previous_week = (today - Duration::days(7))
        .format("%Y-%m-%d")
        .to_string();
    generate_weekly_summary(connection, &previous_week)
}
