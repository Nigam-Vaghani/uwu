mod active_window;
mod battery;
mod cpu;
mod memory;
mod network;
mod session;

use crate::features::database::chrono_now;
use crate::features::events::{
    emit_coding_session_ended, emit_coding_session_started, emit_critical_battery,
    emit_high_cpu, emit_high_ram, emit_low_battery, emit_system_stats,
};
use cpu::poll_cpu;
use memory::poll_memory;
use network::NetworkTracker;
use serde::{Deserialize, Serialize};
use session::{
    detect_app_name, end_session, get_coding_sessions as fetch_coding_sessions, is_coding_active,
    is_coding_window, start_session, CodingSession,
};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use sysinfo::System;
use tauri::{AppHandle, Manager, State};

const CPU_INTERVAL: Duration = Duration::from_secs(5);
const RAM_INTERVAL: Duration = Duration::from_secs(10);
const BATTERY_INTERVAL: Duration = Duration::from_secs(30);
const NETWORK_INTERVAL: Duration = Duration::from_secs(5);
const WINDOW_INTERVAL: Duration = Duration::from_secs(2);
const CODING_INTERVAL: Duration = Duration::from_secs(5);
const UPTIME_INTERVAL: Duration = Duration::from_secs(60);

const CPU_DELTA_THRESHOLD: f32 = 1.0;
const RAM_DELTA_THRESHOLD: f32 = 1.0;
const BATTERY_DELTA_THRESHOLD: f32 = 1.0;
const NETWORK_DELTA_THRESHOLD: f64 = 5.0;

const HIGH_CPU_THRESHOLD: f32 = 80.0;
const HIGH_CPU_DURATION_SECS: i64 = 10;
const HIGH_RAM_THRESHOLD: f32 = 85.0;
const LOW_BATTERY_THRESHOLD: f32 = 20.0;
const CRITICAL_BATTERY_THRESHOLD: f32 = 10.0;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStats {
    pub cpu_percent: f32,
    pub ram_used_mb: u64,
    pub ram_total_mb: u64,
    pub ram_percent: f32,
    pub battery_percent: Option<f32>,
    pub battery_charging: Option<bool>,
    pub network_send_kbps: f64,
    pub network_recv_kbps: f64,
    pub active_window: Option<String>,
    pub active_app: Option<String>,
    pub coding_session_active: bool,
    pub coding_session_duration_secs: u64,
    pub uptime_secs: u64,
}

#[derive(Default)]
pub(crate) struct MonitorInner {
    stats: SystemStats,
    last_cpu_poll: Option<Instant>,
    last_ram_poll: Option<Instant>,
    last_battery_poll: Option<Instant>,
    last_network_poll: Option<Instant>,
    last_window_poll: Option<Instant>,
    last_coding_poll: Option<Instant>,
    cached_coding_active: bool,
    cached_coding_app: Option<String>,
    cached_coding_context: Option<String>,
    last_uptime_poll: Option<Instant>,
    high_cpu_since: Option<i64>,
    high_cpu_emitted: bool,
    high_ram_emitted: bool,
    low_battery_emitted: bool,
    critical_battery_emitted: bool,
    active_session_id: Option<i64>,
    active_session_app: Option<String>,
    active_session_started_at: Option<i64>,
}

pub struct SystemMonitorState(pub Mutex<MonitorInner>);

impl SystemMonitorState {
    pub fn new() -> Self {
        Self(Mutex::new(MonitorInner::default()))
    }

    pub fn snapshot(&self) -> Result<SystemStats, String> {
        let inner = self
            .0
            .lock()
            .map_err(|_| "system monitor lock poisoned".to_string())?;
        Ok(inner.stats.clone())
    }
}

fn should_emit_stats(previous: &SystemStats, current: &SystemStats) -> bool {
    (current.cpu_percent - previous.cpu_percent).abs() >= CPU_DELTA_THRESHOLD
        || (current.ram_percent - previous.ram_percent).abs() >= RAM_DELTA_THRESHOLD
        || current.ram_used_mb != previous.ram_used_mb
        || battery_changed(previous, current)
        || (current.network_send_kbps - previous.network_send_kbps).abs()
            >= NETWORK_DELTA_THRESHOLD
        || (current.network_recv_kbps - previous.network_recv_kbps).abs()
            >= NETWORK_DELTA_THRESHOLD
        || current.active_window != previous.active_window
        || current.active_app != previous.active_app
        || current.coding_session_active != previous.coding_session_active
        || current.coding_session_duration_secs != previous.coding_session_duration_secs
        || current.uptime_secs != previous.uptime_secs
}

fn battery_changed(previous: &SystemStats, current: &SystemStats) -> bool {
    match (previous.battery_percent, current.battery_percent) {
        (Some(prev), Some(next)) => (next - prev).abs() >= BATTERY_DELTA_THRESHOLD,
        (None, Some(_)) | (Some(_), None) => true,
        (None, None) => previous.battery_charging != current.battery_charging,
    }
}

fn extract_app_name(title: &str) -> String {
    if let Some((app, _)) = title.split_once(" - ") {
        app.trim().to_string()
    } else {
        title.trim().to_string()
    }
}

#[derive(Debug, Clone)]
enum MonitorAlert {
    HighCpu(f32),
    HighRam(f32),
    LowBattery(f32),
    CriticalBattery(f32),
    CodingSessionStarted(String),
    CodingSessionEnded(String),
}

fn sync_coding_detection(inner: &mut MonitorInner, system: &mut System, now: Instant) {
    let window_title = inner.stats.active_window.as_deref();

    if poll_due(&mut inner.last_coding_poll, CODING_INTERVAL, now) {
        let (is_active, app_name, session_context) = is_coding_active(system, window_title);
        inner.cached_coding_active = is_active;
        inner.cached_coding_app = app_name;
        inner.cached_coding_context = session_context;
        return;
    }

    if let Some(title) = window_title {
        if is_coding_window(title) {
            inner.cached_coding_active = true;
            inner.cached_coding_app = Some(detect_app_name(title));
            inner.cached_coding_context = Some(title.to_string());
            return;
        }
    }

    if inner.active_session_id.is_some() {
        inner.cached_coding_active = true;
    }
}

fn update_coding_session(
    app: &AppHandle,
    inner: &mut MonitorInner,
) -> Result<Vec<MonitorAlert>, String> {
    let mut alerts = Vec::new();
    let is_coding = inner.cached_coding_active;

    if is_coding {
        if inner.active_session_id.is_none() {
            let app_name = inner
                .cached_coding_app
                .clone()
                .unwrap_or_else(|| "Unknown IDE".to_string());
            let title = inner
                .cached_coding_context
                .clone()
                .unwrap_or_else(|| app_name.clone());
            let (session_id, started_app) = start_session(app, &title, &app_name)?;
            inner.active_session_id = Some(session_id);
            inner.active_session_app = Some(started_app.clone());
            inner.active_session_started_at = Some(chrono_now());
            alerts.push(MonitorAlert::CodingSessionStarted(started_app));
        }

        let started_at = inner.active_session_started_at.unwrap_or(chrono_now());
        inner.stats.coding_session_active = true;
        inner.stats.coding_session_duration_secs =
            chrono_now().saturating_sub(started_at) as u64;
    } else if let Some(session_id) = inner.active_session_id.take() {
        let duration_secs = end_session(app, session_id)?;
        let _ = crate::features::gamification::on_coding_session_ended(app, duration_secs);
        if let Some(app_name) = inner.active_session_app.take() {
            alerts.push(MonitorAlert::CodingSessionEnded(app_name));
        }
        inner.active_session_started_at = None;
        inner.stats.coding_session_active = false;
        inner.stats.coding_session_duration_secs = 0;
    } else {
        inner.stats.coding_session_active = false;
        inner.stats.coding_session_duration_secs = 0;
    }

    Ok(alerts)
}

fn evaluate_alerts(inner: &mut MonitorInner) -> Vec<MonitorAlert> {
    let mut alerts = Vec::new();
    let now = chrono_now();
    let cpu = inner.stats.cpu_percent;

    if cpu > HIGH_CPU_THRESHOLD {
        if inner.high_cpu_since.is_none() {
            inner.high_cpu_since = Some(now);
        } else if !inner.high_cpu_emitted {
            let since = inner.high_cpu_since.unwrap_or(now);
            if now.saturating_sub(since) >= HIGH_CPU_DURATION_SECS {
                alerts.push(MonitorAlert::HighCpu(cpu));
                inner.high_cpu_emitted = true;
            }
        }
    } else {
        inner.high_cpu_since = None;
        inner.high_cpu_emitted = false;
    }

    if inner.stats.ram_percent > HIGH_RAM_THRESHOLD && !inner.high_ram_emitted {
        alerts.push(MonitorAlert::HighRam(inner.stats.ram_percent));
        inner.high_ram_emitted = true;
    } else if inner.stats.ram_percent <= HIGH_RAM_THRESHOLD {
        inner.high_ram_emitted = false;
    }

    if let Some(battery) = inner.stats.battery_percent {
        if battery < CRITICAL_BATTERY_THRESHOLD && !inner.critical_battery_emitted {
            alerts.push(MonitorAlert::CriticalBattery(battery));
            inner.critical_battery_emitted = true;
            inner.low_battery_emitted = true;
        } else if battery < LOW_BATTERY_THRESHOLD && !inner.low_battery_emitted {
            alerts.push(MonitorAlert::LowBattery(battery));
            inner.low_battery_emitted = true;
        } else if battery >= LOW_BATTERY_THRESHOLD {
            inner.low_battery_emitted = false;
            inner.critical_battery_emitted = false;
        }
    }

    alerts
}

fn emit_alerts(app: &AppHandle, alerts: Vec<MonitorAlert>) {
    for alert in alerts {
        let _ = match alert {
            MonitorAlert::HighCpu(value) => emit_high_cpu(app, value),
            MonitorAlert::HighRam(value) => emit_high_ram(app, value),
            MonitorAlert::LowBattery(value) => emit_low_battery(app, value),
            MonitorAlert::CriticalBattery(value) => emit_critical_battery(app, value),
            MonitorAlert::CodingSessionStarted(app_name) => {
                emit_coding_session_started(app, app_name)
            }
            MonitorAlert::CodingSessionEnded(app_name) => emit_coding_session_ended(app, app_name),
        };
    }
}

fn poll_due(
    last_poll: &mut Option<Instant>,
    interval: Duration,
    now: Instant,
) -> bool {
    match last_poll {
        Some(previous) if now.duration_since(*previous) < interval => false,
        slot => {
            *slot = Some(now);
            true
        }
    }
}

pub fn start_monitor(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut system = System::new_all();
        let mut network_tracker = NetworkTracker::new();

        system.refresh_cpu_all();
        tokio::time::sleep(Duration::from_millis(200)).await;
        system.refresh_cpu_all();

        if let Ok(mut inner) = app.state::<SystemMonitorState>().0.lock() {
            inner.stats.uptime_secs = System::uptime();
            inner.last_uptime_poll = Some(Instant::now());
        }

        loop {
            let now = Instant::now();
            let state = app.state::<SystemMonitorState>();

            let mut pending_alerts = Vec::new();
            let mut stats_to_emit = None;

            if let Ok(mut inner) = state.0.lock() {
                let previous = inner.stats.clone();

                if poll_due(&mut inner.last_cpu_poll, CPU_INTERVAL, now) {
                    inner.stats.cpu_percent = poll_cpu(&mut system);
                }

                if poll_due(&mut inner.last_ram_poll, RAM_INTERVAL, now) {
                    let memory = poll_memory(&mut system);
                    inner.stats.ram_used_mb = memory.used_mb;
                    inner.stats.ram_total_mb = memory.total_mb;
                    inner.stats.ram_percent = memory.percent;
                }

                if poll_due(&mut inner.last_battery_poll, BATTERY_INTERVAL, now) {
                    let battery = battery::poll_battery();
                    inner.stats.battery_percent = battery.percent;
                    inner.stats.battery_charging = battery.charging;
                }

                if poll_due(&mut inner.last_network_poll, NETWORK_INTERVAL, now) {
                    let rates = network_tracker.poll();
                    inner.stats.network_send_kbps = rates.send_kbps;
                    inner.stats.network_recv_kbps = rates.recv_kbps;
                }

                if poll_due(&mut inner.last_window_poll, WINDOW_INTERVAL, now) {
                    let title = active_window::get_active_window_title();
                    inner.stats.active_window = title.clone();
                    inner.stats.active_app = title.as_deref().map(extract_app_name);
                }

                sync_coding_detection(&mut inner, &mut system, now);

                if let Ok(session_alerts) = update_coding_session(&app, &mut inner) {
                    pending_alerts.extend(session_alerts);
                }

                if inner.stats.coding_session_active {
                    let _ = crate::features::gamification::tick_coding_progress(
                        &app,
                        inner.stats.coding_session_duration_secs,
                    );
                }

                if poll_due(&mut inner.last_uptime_poll, UPTIME_INTERVAL, now) {
                    inner.stats.uptime_secs = System::uptime();
                }

                pending_alerts.extend(evaluate_alerts(&mut inner));

                if should_emit_stats(&previous, &inner.stats) {
                    stats_to_emit = Some(inner.stats.clone());
                }
            }

            emit_alerts(&app, pending_alerts);
            if let Some(stats) = stats_to_emit {
                let _ = emit_system_stats(&app, stats);
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });
}

#[tauri::command]
pub fn get_system_stats(state: State<'_, SystemMonitorState>) -> Result<SystemStats, String> {
    state.snapshot()
}

#[tauri::command]
pub fn get_coding_sessions(app: AppHandle) -> Result<Vec<CodingSession>, String> {
    fetch_coding_sessions(&app)
}

#[tauri::command]
pub fn get_uptime(state: State<'_, SystemMonitorState>) -> Result<u64, String> {
    Ok(state.snapshot()?.uptime_secs)
}
