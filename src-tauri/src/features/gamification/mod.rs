mod achievements;
mod objectives;
mod productivity;
mod streak;
mod types;
mod xp;

pub use types::*;
pub use xp::{award_xp_with_db, load_pet_stats, xp_for_coding_session};

use crate::features::database::with_db;
use crate::features::events::emit_objective_completed;
use objectives::{
    add_coding_secs, record_break_ack, record_generic_ack, record_hydration_ack,
};
use rusqlite::Connection;
use std::time::Duration;
use tauri::AppHandle;
use xp::award_xp_internal;

#[tauri::command]
pub fn get_pet_stats(app: AppHandle) -> Result<PetStats, String> {
    with_db(&app, load_pet_stats)
}

#[tauri::command]
pub fn get_daily_objectives(app: AppHandle) -> Result<Vec<DailyObjective>, String> {
    objectives::get_daily_objectives_command(app)
}

#[tauri::command]
pub fn get_achievements(app: AppHandle) -> Result<Vec<AchievementStatus>, String> {
    achievements::get_achievements_command(app)
}

#[tauri::command]
pub fn get_weekly_summary(app: AppHandle) -> Result<WeeklySummary, String> {
    productivity::get_weekly_summary_command(app)
}

#[tauri::command]
pub fn get_productivity_breakdown(app: AppHandle) -> Result<ProductivityBreakdown, String> {
    productivity::get_productivity_breakdown_command(app)
}

#[tauri::command]
pub fn award_xp(app: AppHandle, input: AwardXpInput) -> Result<AwardXpResult, String> {
    award_xp_with_db(&app, input.amount)
}

#[tauri::command]
pub fn acknowledge_reminder(
    app: AppHandle,
    input: AcknowledgeReminderInput,
) -> Result<AwardXpResult, String> {
    with_db(&app, |connection| {
        streak::on_activity(connection)?;

        let xp_amount = match input.category.as_str() {
            "hydration" => {
                record_hydration_ack(connection)?;
                XP_HYDRATION_ACK
            }
            "break" => {
                record_break_ack(connection)?;
                XP_BREAK_ON_TIME
            }
            _ => {
                record_generic_ack(connection)?;
                0
            }
        };

        let result = if xp_amount > 0 {
            award_xp_internal(&app, connection, xp_amount)?
        } else {
            AwardXpResult {
                stats: load_pet_stats(connection)?,
                leveled_up: false,
                new_level: None,
                achievements_unlocked: Vec::new(),
            }
        };

        check_newly_completed_objectives(&app, connection)?;
        Ok(result)
    })
}

pub fn on_coding_session_ended(app: &AppHandle, duration_secs: i64) -> Result<(), String> {
    with_db(app, |connection| {
        add_coding_secs(connection, duration_secs)?;
        streak::on_activity(connection)?;

        let xp = xp_for_coding_session(duration_secs);
        if xp > 0 {
            award_xp_internal(app, connection, xp)?;
        }

        check_newly_completed_objectives(app, connection)?;
        Ok(())
    })
}

pub fn tick_coding_progress(app: &AppHandle, active_secs: u64) -> Result<(), String> {
    with_db(app, |connection| {
        objectives::ensure_daily_objectives(connection)?;
        objectives::update_objective_progress(connection, "code_2h", active_secs as i64)?;
        productivity::recalculate_score(connection)?;
        Ok(())
    })
}

fn check_newly_completed_objectives(app: &AppHandle, connection: &Connection) -> Result<(), String> {
    let objectives = objectives::get_daily_objectives(connection)?;
    for objective in objectives {
        if !objective.completed {
            continue;
        }

        let key = format!("objective_awarded_{}", objective.id);
        let already: Option<String> = crate::features::database::get_config_value(connection, &key)?;
        if already.is_some() {
            continue;
        }

        connection
            .execute(
                "INSERT OR REPLACE INTO config (key, value) VALUES (?1, '1')",
                [key],
            )
            .map_err(|error| error.to_string())?;

        let _ = emit_objective_completed(app, objective.title.clone());
        award_xp_internal(app, connection, XP_OBJECTIVE_COMPLETE)?;
    }
    Ok(())
}

pub fn start_scheduler(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(60));

        loop {
            tick.tick().await;
            let _ = with_db(&app, |connection| {
                streak::process_midnight_streak(&app, connection)
            });
        }
    });
}

pub fn init(app: &AppHandle) -> Result<(), String> {
    with_db(app, |connection| {
        objectives::ensure_daily_objectives(connection)?;
        productivity::recalculate_score(connection)?;
        Ok(())
    })?;
    start_scheduler(app.clone());
    Ok(())
}
