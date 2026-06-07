use sysinfo::System;

pub fn poll_cpu(system: &mut System) -> f32 {
    system.refresh_cpu_usage();
    system.global_cpu_usage()
}
