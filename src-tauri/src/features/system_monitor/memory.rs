use sysinfo::System;

#[derive(Debug, Clone, Copy)]
pub struct MemorySnapshot {
    pub used_mb: u64,
    pub total_mb: u64,
    pub percent: f32,
}

pub fn poll_memory(system: &mut System) -> MemorySnapshot {
    system.refresh_memory();
    let used = system.used_memory();
    let total = system.total_memory().max(1);
    let used_mb = used / 1024 / 1024;
    let total_mb = total / 1024 / 1024;
    let percent = (used as f32 / total as f32) * 100.0;

    MemorySnapshot {
        used_mb,
        total_mb,
        percent,
    }
}
