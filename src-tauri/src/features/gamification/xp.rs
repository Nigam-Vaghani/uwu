use super::types::{
    level_from_xp, level_name, level_progress, AwardXpResult, PetStats, XP_ACHIEVEMENT,
};
use crate::features::database::with_db;
use crate::features::events::{emit_achievement_unlocked, emit_level_up};
use crate::features::gamification::achievements::evaluate_achievements;
use rusqlite::{params, Connection};
use tauri::AppHandle;

pub fn load_pet_stats(connection: &Connection) -> Result<PetStats, String> {
    let (xp, level, streak): (i64, i64, i64) = connection
        .query_row(
            "SELECT xp, level, streak FROM pet_state WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|error| error.to_string())?;

    let today = super::types::today_date();
    let productivity_score: i64 = connection
        .query_row(
            "SELECT productivity_score FROM daily_activity WHERE date = ?1",
            [today],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let computed_level = level_from_xp(xp);
    let (xp_into_level, xp_for_next_level) = level_progress(xp, computed_level);

    Ok(PetStats {
        xp,
        level: computed_level.max(level),
        streak,
        productivity_score,
        xp_into_level,
        xp_for_next_level,
        level_name: level_name(computed_level.max(level)),
    })
}

fn sync_level(connection: &Connection, xp: i64) -> Result<i64, String> {
    let level = level_from_xp(xp);
    connection
        .execute(
            "UPDATE pet_state SET xp = ?1, level = ?2 WHERE id = 1",
            params![xp, level],
        )
        .map_err(|error| error.to_string())?;
    Ok(level)
}

pub fn award_xp_internal(
    app: &AppHandle,
    connection: &Connection,
    amount: i64,
) -> Result<AwardXpResult, String> {
    if amount <= 0 {
        return Ok(AwardXpResult {
            stats: load_pet_stats(connection)?,
            leveled_up: false,
            new_level: None,
            achievements_unlocked: Vec::new(),
        });
    }

    let previous_xp: i64 = connection
        .query_row("SELECT xp FROM pet_state WHERE id = 1", [], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    let previous_level = level_from_xp(previous_xp);
    let new_xp = previous_xp + amount;
    let new_level = sync_level(connection, new_xp)?;

    let mut leveled_up = false;
    if new_level > previous_level {
        leveled_up = true;
        let _ = emit_level_up(app, new_level);
        super::objectives::refresh_objectives_from_activity(connection)?;
    }

    let achievements = evaluate_achievements(app, connection)?;
    for achievement in &achievements {
        let _ = emit_achievement_unlocked(
            app,
            achievement.id.clone(),
            achievement.name.clone(),
            achievement.icon.clone(),
        );
        let current_xp: i64 = connection
            .query_row("SELECT xp FROM pet_state WHERE id = 1", [], |row| row.get(0))
            .map_err(|error| error.to_string())?;
        sync_level(connection, current_xp + XP_ACHIEVEMENT)?;
    }

    Ok(AwardXpResult {
        stats: load_pet_stats(connection)?,
        leveled_up,
        new_level: if leveled_up { Some(new_level) } else { None },
        achievements_unlocked: achievements,
    })
}

pub fn award_xp_with_db(app: &AppHandle, amount: i64) -> Result<AwardXpResult, String> {
    with_db(app, |connection| award_xp_internal(app, connection, amount))
}

pub fn xp_for_coding_session(duration_secs: i64) -> i64 {
    let blocks = duration_secs / 1800;
    blocks * super::types::XP_CODING_PER_30_MIN
}

#[cfg(test)]
mod tests {
    use super::super::types::{level_from_xp, total_xp_for_level, xp_required_for_level_up};

    #[test]
    fn level_curve_matches_spec() {
        assert_eq!(xp_required_for_level_up(7), 4900);
        assert_eq!(total_xp_for_level(8), 14000);
        assert_eq!(level_from_xp(3100 + total_xp_for_level(7)), 7);
    }
}
