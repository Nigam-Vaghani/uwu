use super::types::{today_date, XP_STREAK_DAILY};
use crate::features::events::emit_streak_milestone;
use rusqlite::{params, Connection};
use tauri::AppHandle;

const STREAK_MILESTONES: &[i64] = &[3, 7, 14, 30];

pub fn mark_daily_activity(connection: &Connection) -> Result<(), String> {
    let today = today_date();
    connection
        .execute(
            "UPDATE pet_state SET last_streak_date = ?1 WHERE id = 1",
            [today],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn had_activity_yesterday(connection: &Connection, yesterday: &str) -> Result<bool, String> {
    let reminders: i64 = connection
        .query_row(
            "SELECT reminders_acknowledged FROM daily_activity WHERE date = ?1",
            [yesterday],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let coding: i64 = connection
        .query_row(
            "SELECT coding_secs FROM daily_activity WHERE date = ?1",
            [yesterday],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let sessions: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sessions
             WHERE date(started_at, 'unixepoch', 'localtime') = ?1",
            [yesterday],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(reminders > 0 || coding > 0 || sessions > 0)
}

pub fn process_midnight_streak(app: &AppHandle, connection: &Connection) -> Result<(), String> {
    use chrono::{Duration, Local};

    let today = today_date();
    let last_reset: Option<String> = connection
        .query_row(
            "SELECT last_daily_reset FROM pet_state WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;

    if last_reset.as_deref() == Some(today.as_str()) {
        return Ok(());
    }

    let yesterday = (Local::now().date_naive() - Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    let (mut streak, last_streak_date): (i64, Option<String>) = connection
        .query_row(
            "SELECT streak, last_streak_date FROM pet_state WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|error| error.to_string())?;

    if had_activity_yesterday(connection, &yesterday)?
        || last_streak_date.as_deref() == Some(yesterday.as_str())
    {
        streak += 1;
        super::xp::award_xp_internal(app, connection, XP_STREAK_DAILY)?;
        if STREAK_MILESTONES.contains(&streak) {
            let _ = emit_streak_milestone(app, streak);
        }
    } else if last_streak_date.is_some() {
        streak = 0;
    }

    connection
        .execute(
            "UPDATE pet_state SET streak = ?1, last_daily_reset = ?2 WHERE id = 1",
            params![streak, today],
        )
        .map_err(|error| error.to_string())?;

    super::objectives::ensure_daily_objectives(connection)?;
    super::productivity::maybe_finalize_previous_week(connection)?;

    Ok(())
}

pub fn on_activity(connection: &Connection) -> Result<(), String> {
    mark_daily_activity(connection)
}
