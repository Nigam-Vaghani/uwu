use rusqlite::{Connection, OptionalExtension};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use tauri_plugin_sql::{Migration, MigrationKind};

pub const DB_URI: &str = "sqlite:uwu.db";

pub fn migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "create_phase2_tables",
            sql: include_str!("migrations/001_phase2.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "create_phase3_sessions",
            sql: include_str!("migrations/002_phase3.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 3,
            description: "create_phase6_gamification",
            sql: include_str!("migrations/003_phase6.sql"),
            kind: MigrationKind::Up,
        },
    ]
}

pub fn db_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("uwu.db"))
        .map_err(|error| error.to_string())
}

pub fn open_connection(app: &AppHandle) -> Result<Connection, String> {
    let path = db_path(app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let connection = Connection::open(path).map_err(|error| error.to_string())?;
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .map_err(|error| error.to_string())?;
    run_migrations(&connection)?;
    seed_defaults(&connection)?;
    Ok(connection)
}

fn ensure_column(
    connection: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<(), String> {
    let exists: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info(?1) WHERE name = ?2",
            rusqlite::params![table, column],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;

    if exists == 0 {
        let sql = format!("ALTER TABLE {table} ADD COLUMN {column} {definition}");
        connection
            .execute(&sql, [])
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn ensure_gamification_columns(connection: &Connection) -> Result<(), String> {
    ensure_column(connection, "pet_state", "xp", "INTEGER NOT NULL DEFAULT 0")?;
    ensure_column(connection, "pet_state", "level", "INTEGER NOT NULL DEFAULT 1")?;
    ensure_column(connection, "pet_state", "streak", "INTEGER NOT NULL DEFAULT 0")?;
    ensure_column(connection, "pet_state", "last_streak_date", "TEXT")?;
    ensure_column(connection, "pet_state", "last_daily_reset", "TEXT")?;
    Ok(())
}

fn run_migrations(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(include_str!("migrations/001_phase2.sql"))
        .map_err(|error| error.to_string())?;
    connection
        .execute_batch(include_str!("migrations/002_phase3.sql"))
        .map_err(|error| error.to_string())?;
    connection
        .execute_batch(include_str!("migrations/003_phase6.sql"))
        .map_err(|error| error.to_string())?;
    ensure_gamification_columns(connection)
}

fn seed_defaults(connection: &Connection) -> Result<(), String> {
    let defaults = [
        ("idle_timeout_minutes", "5"),
        ("random_message_min_minutes", "10"),
        ("random_message_max_minutes", "20"),
    ];

    for (key, value) in defaults {
        connection
            .execute(
                "INSERT OR IGNORE INTO config (key, value) VALUES (?1, ?2)",
                [key, value],
            )
            .map_err(|error| error.to_string())?;
    }

    let reminder_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM reminders", [], |row| row.get(0))
        .map_err(|error| error.to_string())?;

    if reminder_count == 0 {
        connection
            .execute(
                "INSERT INTO reminders (title, message, cron_expr, enabled, category)
                 VALUES (?1, ?2, ?3, 1, ?4)",
                (
                    "Hydration break",
                    "Time to drink some water!",
                    "@every:30",
                    "hydration",
                ),
            )
            .map_err(|error| error.to_string())?;

        connection
            .execute(
                "INSERT INTO reminders (title, message, cron_expr, enabled, category)
                 VALUES (?1, ?2, ?3, 1, ?4)",
                (
                    "Take a break",
                    "Step away from the screen for a bit.",
                    "@every:90",
                    "break",
                ),
            )
            .map_err(|error| error.to_string())?;
    }

    connection
        .execute(
            "INSERT OR IGNORE INTO pet_state (id, mood, last_interaction, excitement_until)
             VALUES (1, 'happy', ?1, NULL)",
            [chrono_now()],
        )
        .map_err(|error| error.to_string())?;

    Ok(())
}

pub fn get_config_value(connection: &Connection, key: &str) -> Result<Option<String>, String> {
    connection
        .query_row(
            "SELECT value FROM config WHERE key = ?1",
            [key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())
}

pub fn get_config_i64(connection: &Connection, key: &str, default: i64) -> Result<i64, String> {
    match get_config_value(connection, key)? {
        Some(value) => value.parse::<i64>().map_err(|error| error.to_string()),
        None => Ok(default),
    }
}

pub fn with_db<T, F>(app: &AppHandle, f: F) -> Result<T, String>
where
    F: FnOnce(&Connection) -> Result<T, String>,
{
    let state = app.state::<DbState>();
    let connection = state
        .0
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    f(&connection)
}

pub struct DbState(pub Mutex<Connection>);

pub fn init(app: &AppHandle) -> Result<(), String> {
    let connection = open_connection(app)?;
    app.manage(DbState(Mutex::new(connection)));
    Ok(())
}

pub fn chrono_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}
