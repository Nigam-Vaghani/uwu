use super::types::{today_date, DailyObjective, ProductivityBreakdown};
use crate::features::database::{chrono_now, with_db};
use rusqlite::{params, Connection};
use tauri::AppHandle;

pub fn ensure_daily_objectives(connection: &Connection) -> Result<(), String> {
    let today = today_date();
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM daily_objectives WHERE date = ?1",
            [today.clone()],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;

    if count > 0 {
        return Ok(());
    }

    let level: i64 = connection
        .query_row("SELECT level FROM pet_state WHERE id = 1", [], |row| row.get(0))
        .unwrap_or(1);
    let reach_target = (level + 1).max(2);

    let mut templates: Vec<(&str, String, i64)> = vec![
        ("code_2h", "Code for 2 hours today".to_string(), 7200),
        ("take_breaks", "Take 3 breaks".to_string(), 3),
        ("hydrate", "Drink water 5 times".to_string(), 5),
        (
            "reach_level",
            format!("Reach level {reach_target}"),
            reach_target,
        ),
    ];

    let seed = chrono_now() as usize;
    templates.sort_by_key(|(category, _, _)| (seed + category.len() * 13) % 101);

    for (category, title, target) in templates.into_iter().take(3) {
        connection
            .execute(
                "INSERT INTO daily_objectives (date, title, category, target, progress, completed)
                 VALUES (?1, ?2, ?3, ?4, 0, 0)",
                params![today.clone(), title, category, target],
            )
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

pub fn get_daily_objectives(connection: &Connection) -> Result<Vec<DailyObjective>, String> {
    ensure_daily_objectives(connection)?;
    let today = today_date();
    let mut statement = connection
        .prepare(
            "SELECT id, date, title, category, target, progress, completed
             FROM daily_objectives
             WHERE date = ?1
             ORDER BY id ASC",
        )
        .map_err(|error| error.to_string())?;

    let objectives = statement
        .query_map([today], map_objective)
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    Ok(objectives)
}

fn map_objective(row: &rusqlite::Row<'_>) -> rusqlite::Result<DailyObjective> {
    Ok(DailyObjective {
        id: row.get(0)?,
        date: row.get(1)?,
        title: row.get(2)?,
        category: row.get(3)?,
        target: row.get(4)?,
        progress: row.get(5)?,
        completed: row.get::<_, i64>(6)? == 1,
    })
}

pub fn update_objective_progress(
    connection: &Connection,
    category: &str,
    progress: i64,
) -> Result<Vec<DailyObjective>, String> {
    let today = today_date();
    ensure_daily_objectives(connection)?;

    connection
        .execute(
            "UPDATE daily_objectives
             SET progress = MAX(progress, ?1),
                 completed = CASE WHEN MAX(progress, ?1) >= target THEN 1 ELSE completed END
             WHERE date = ?2 AND category = ?3 AND completed = 0",
            params![progress, today, category],
        )
        .map_err(|error| error.to_string())?;

    get_daily_objectives(connection)
}

pub fn increment_objective(
    connection: &Connection,
    category: &str,
    amount: i64,
) -> Result<Vec<DailyObjective>, String> {
    let today = today_date();
    ensure_daily_objectives(connection)?;

    connection
        .execute(
            "UPDATE daily_objectives
             SET progress = progress + ?1,
                 completed = CASE WHEN progress + ?1 >= target THEN 1 ELSE completed END
             WHERE date = ?2 AND category = ?3 AND completed = 0",
            params![amount, today, category],
        )
        .map_err(|error| error.to_string())?;

    get_daily_objectives(connection)
}

pub fn refresh_objectives_from_activity(connection: &Connection) -> Result<(), String> {
    ensure_daily_objectives(connection)?;
    let today = today_date();

    let (coding_secs, breaks, hydration): (i64, i64, i64) = connection
        .query_row(
            "SELECT coding_secs, breaks_taken, hydration_acknowledged
             FROM daily_activity WHERE date = ?1",
            [today.clone()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap_or((0, 0, 0));

    let level: i64 = connection
        .query_row("SELECT level FROM pet_state WHERE id = 1", [], |row| row.get(0))
        .unwrap_or(1);

    update_objective_progress(connection, "code_2h", coding_secs)?;
    update_objective_progress(connection, "take_breaks", breaks)?;
    update_objective_progress(connection, "hydrate", hydration)?;
    update_objective_progress(connection, "reach_level", level)?;

    Ok(())
}

pub fn get_productivity_breakdown(connection: &Connection) -> Result<ProductivityBreakdown, String> {
    let today = today_date();
    let (coding_secs, breaks, hydration, objectives_completed): (i64, i64, i64, i64) =
        connection
            .query_row(
                "SELECT da.coding_secs, da.breaks_taken, da.hydration_acknowledged,
                        (SELECT COUNT(*) FROM daily_objectives WHERE date = da.date AND completed = 1)
                 FROM daily_activity da
                 WHERE da.date = ?1",
                [today.clone()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap_or((0, 0, 0, 0));

    Ok(ProductivityBreakdown {
        coding: ((coding_secs.min(7200) as f64 / 7200.0) * 40.0).round() as i64,
        breaks: ((breaks.min(3) as f64 / 3.0) * 30.0).round() as i64,
        hydration: ((hydration.min(5) as f64 / 5.0) * 20.0).round() as i64,
        goals: ((objectives_completed.min(3) as f64 / 3.0) * 10.0).round() as i64,
    })
}

pub fn ensure_daily_activity(connection: &Connection) -> Result<(), String> {
    let today = today_date();
    connection
        .execute(
            "INSERT OR IGNORE INTO daily_activity (date) VALUES (?1)",
            [today],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn add_coding_secs(connection: &Connection, secs: i64) -> Result<(), String> {
    ensure_daily_activity(connection)?;
    let today = today_date();
    connection
        .execute(
            "UPDATE daily_activity SET coding_secs = coding_secs + ?1 WHERE date = ?2",
            params![secs, today],
        )
        .map_err(|error| error.to_string())?;
    refresh_objectives_from_activity(connection)?;
    super::productivity::recalculate_score(connection)?;
    Ok(())
}

pub fn record_hydration_ack(connection: &Connection) -> Result<(), String> {
    ensure_daily_activity(connection)?;
    let today = today_date();
    connection
        .execute(
            "UPDATE daily_activity
             SET hydration_acknowledged = hydration_acknowledged + 1,
                 reminders_acknowledged = reminders_acknowledged + 1
             WHERE date = ?1",
            [today],
        )
        .map_err(|error| error.to_string())?;
    increment_objective(connection, "hydrate", 1)?;
    super::productivity::recalculate_score(connection)?;
    Ok(())
}

pub fn record_break_ack(connection: &Connection) -> Result<(), String> {
    ensure_daily_activity(connection)?;
    let today = today_date();
    connection
        .execute(
            "UPDATE daily_activity
             SET breaks_taken = breaks_taken + 1,
                 reminders_acknowledged = reminders_acknowledged + 1
             WHERE date = ?1",
            [today],
        )
        .map_err(|error| error.to_string())?;
    increment_objective(connection, "take_breaks", 1)?;
    super::productivity::recalculate_score(connection)?;
    Ok(())
}

pub fn record_generic_ack(connection: &Connection) -> Result<(), String> {
    ensure_daily_activity(connection)?;
    let today = today_date();
    connection
        .execute(
            "UPDATE daily_activity SET reminders_acknowledged = reminders_acknowledged + 1 WHERE date = ?1",
            [today],
        )
        .map_err(|error| error.to_string())?;
    super::productivity::recalculate_score(connection)?;
    Ok(())
}

pub fn get_daily_objectives_command(app: AppHandle) -> Result<Vec<DailyObjective>, String> {
    with_db(&app, get_daily_objectives)
}
