use rmcp::handler::server::wrapper::{Json, Parameters};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct PerformanceArgs {}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CpuInfo {
    pub usage_percent: f64,
    pub core_count: usize,
    pub brand: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct MemoryInfo {
    pub total_gb: f64,
    pub used_gb: f64,
    pub free_gb: f64,
    pub usage_percent: f64,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct DiskIoInfo {
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct PerformanceOutput {
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub top_processes: Vec<TopProcess>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct TopProcess {
    pub name: String,
    pub memory_mb: u64,
    pub cpu_usage: f32,
}

pub fn performance_inner(_args: PerformanceArgs) -> PerformanceOutput {
    let mut sys = System::new();
    sys.refresh_cpu_usage();
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_cpu_usage();

    sys.refresh_memory();
    sys.refresh_processes();

    let cpu_info = CpuInfo {
        usage_percent: sys.global_cpu_info().cpu_usage() as f64,
        core_count: sys.cpus().len(),
        brand: sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or_default(),
    };

    let total_memory = sys.total_memory();
    let available_memory = sys.available_memory();
    let used_memory = total_memory - available_memory;
    let memory_percent = if total_memory > 0 {
        (used_memory as f64 / total_memory as f64) * 100.0
    } else {
        0.0
    };

    let memory_info = MemoryInfo {
        total_gb: (total_memory as f64 / 1024.0 / 1024.0 / 1024.0 * 100.0).round() / 100.0,
        used_gb: (used_memory as f64 / 1024.0 / 1024.0 / 1024.0 * 100.0).round() / 100.0,
        free_gb: (available_memory as f64 / 1024.0 / 1024.0 / 1024.0 * 100.0).round() / 100.0,
        usage_percent: (memory_percent * 100.0).round() / 100.0,
    };

    let mut top_processes: Vec<TopProcess> = sys
        .processes()
        .iter()
        .map(|(_, process)| TopProcess {
            name: process.name().to_string(),
            memory_mb: process.memory() / 1024 / 1024,
            cpu_usage: process.cpu_usage(),
        })
        .collect();

    top_processes.sort_by(|a, b| b.memory_mb.cmp(&a.memory_mb));
    top_processes.truncate(10);

    PerformanceOutput {
        cpu: cpu_info,
        memory: memory_info,
        top_processes,
    }
}

#[tracing::instrument(skip_all)]
pub fn performance(Parameters(args): Parameters<PerformanceArgs>) -> Json<PerformanceOutput> {
    Json(performance_inner(args))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn performance_returns_valid_data() {
        let output = performance_inner(PerformanceArgs {});
        assert!(output.cpu.core_count > 0, "should have at least one core");
        assert!(output.memory.total_gb > 0.0, "should have positive total memory");
        assert!(!output.top_processes.is_empty(), "should have processes");
    }

    #[test]
    fn cpu_usage_in_range() {
        let output = performance_inner(PerformanceArgs {});
        assert!(output.cpu.usage_percent >= 0.0 && output.cpu.usage_percent <= 100.0);
    }

    #[test]
    fn memory_usage_in_range() {
        let output = performance_inner(PerformanceArgs {});
        assert!(output.memory.usage_percent >= 0.0 && output.memory.usage_percent <= 100.0);
    }
}
