use crate::features::database::{chrono_now, with_db};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use sysinfo::{ProcessesToUpdate, System};
use tauri::AppHandle;

#[derive(Debug, Clone, Copy)]
struct CodingTool {
    label: &'static str,
    window_patterns: &'static [&'static str],
    process_names: &'static [&'static str],
}

const CODING_TOOLS: &[CodingTool] = &[
    CodingTool {
        label: "Visual Studio Code",
        window_patterns: &["Visual Studio Code", "Code - OSS"],
        process_names: &["code.exe", "code - oss.exe"],
    },
    CodingTool {
        label: "Cursor",
        window_patterns: &["Cursor"],
        process_names: &["cursor.exe"],
    },
    CodingTool {
        label: "IntelliJ IDEA",
        window_patterns: &["IntelliJ IDEA"],
        process_names: &["idea64.exe", "idea.exe"],
    },
    CodingTool {
        label: "WebStorm",
        window_patterns: &["WebStorm"],
        process_names: &["webstorm64.exe", "webstorm.exe"],
    },
    CodingTool {
        label: "PyCharm",
        window_patterns: &["PyCharm"],
        process_names: &["pycharm64.exe", "pycharm.exe"],
    },
    CodingTool {
        label: "Rider",
        window_patterns: &["Rider"],
        process_names: &["rider64.exe", "rider.exe"],
    },
    CodingTool {
        label: "Neovim",
        window_patterns: &["Neovim"],
        process_names: &["nvim.exe", "neovide.exe"],
    },
    CodingTool {
        label: "Vim",
        window_patterns: &["Vim"],
        process_names: &["vim.exe", "gvim.exe"],
    },
    CodingTool {
        label: "Zed",
        window_patterns: &["Zed"],
        process_names: &["zed.exe"],
    },
    CodingTool {
        label: "Antigravity IDE",
        window_patterns: &["Antigravity IDE", "Antigravity"],
        process_names: &["antigravity ide.exe", "antigravity.exe"],
    },
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodingSession {
    pub id: i64,
    pub app_name: String,
    pub window_title: Option<String>,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub duration_secs: Option<i64>,
}

fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}

fn match_window_title(title: &str) -> Option<&'static str> {
    let lower = normalize(title);
    for tool in CODING_TOOLS {
        if tool
            .window_patterns
            .iter()
            .any(|pattern| lower.contains(&normalize(pattern)))
        {
            return Some(tool.label);
        }
    }
    None
}

fn match_process_name(process_name: &str) -> Option<&'static str> {
    let lower = normalize(process_name);
    for tool in CODING_TOOLS {
        if tool.process_names.iter().any(|name| lower == *name) {
            return Some(tool.label);
        }
    }
    None
}

pub fn is_coding_window(title: &str) -> bool {
    match_window_title(title).is_some()
}

pub fn detect_app_name(title: &str) -> String {
    match_window_title(title)
        .map(str::to_string)
        .unwrap_or_else(|| "Unknown IDE".to_string())
}

pub fn detect_running_coding_tool(system: &mut System) -> Option<String> {
    system.refresh_processes(ProcessesToUpdate::All, false);

    for process in system.processes().values() {
        let name = process.name().to_string_lossy();
        if let Some(label) = match_process_name(&name) {
            return Some(label.to_string());
        }
    }

    None
}

pub fn is_coding_active(
    system: &mut System,
    window_title: Option<&str>,
) -> (bool, Option<String>, Option<String>) {
    let foreground_tool = window_title.and_then(match_window_title).map(str::to_string);
    let running_tool = detect_running_coding_tool(system);

    let is_active = foreground_tool.is_some() || running_tool.is_some();
    let app_name = foreground_tool
        .clone()
        .or_else(|| running_tool.clone());

    let session_context = window_title
        .map(str::to_string)
        .or_else(|| running_tool.map(|tool| format!("{tool} (running in background)")));

    (is_active, app_name, session_context)
}

pub fn start_session(
    app: &AppHandle,
    window_title: &str,
    app_name: &str,
) -> Result<(i64, String), String> {
    let now = chrono_now();

    with_db(app, |connection| {
        connection
            .execute(
                "INSERT INTO sessions (app_name, window_title, started_at)
                 VALUES (?1, ?2, ?3)",
                params![app_name, window_title, now],
            )
            .map_err(|error| error.to_string())?;
        Ok((connection.last_insert_rowid(), app_name.to_string()))
    })
}

pub fn end_session(app: &AppHandle, session_id: i64) -> Result<i64, String> {
    let now = chrono_now();

    with_db(app, |connection| {
        let started_at: i64 = connection
            .query_row(
                "SELECT started_at FROM sessions WHERE id = ?1 AND ended_at IS NULL",
                [session_id],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;

        let duration_secs = now.saturating_sub(started_at);
        connection
            .execute(
                "UPDATE sessions
                 SET ended_at = ?1, duration_secs = ?2
                 WHERE id = ?3",
                params![now, duration_secs, session_id],
            )
            .map_err(|error| error.to_string())?;
        Ok(duration_secs)
    })
}

pub fn get_coding_sessions(app: &AppHandle) -> Result<Vec<CodingSession>, String> {
    with_db(app, |connection| {
        let mut statement = connection
            .prepare(
                "SELECT id, app_name, window_title, started_at, ended_at, duration_secs
                 FROM sessions
                 ORDER BY started_at DESC",
            )
            .map_err(|error| error.to_string())?;

        let sessions = statement
            .query_map([], |row| {
                Ok(CodingSession {
                    id: row.get(0)?,
                    app_name: row.get(1)?,
                    window_title: row.get(2)?,
                    started_at: row.get(3)?,
                    ended_at: row.get(4)?,
                    duration_secs: row.get(5)?,
                })
            })
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;

        Ok(sessions)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_antigravity_window_and_process() {
        assert_eq!(
            match_window_title("main.rs - project - Antigravity IDE"),
            Some("Antigravity IDE")
        );
        assert_eq!(match_process_name("Antigravity IDE.exe"), Some("Antigravity IDE"));
        assert_eq!(match_process_name("Antigravity.exe"), Some("Antigravity IDE"));
    }

    #[test]
    fn cursor_window_still_matches() {
        assert!(is_coding_window("index.ts - uwu-companion - Cursor"));
    }
}
