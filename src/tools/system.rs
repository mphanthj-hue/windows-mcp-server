use rmcp::handler::server::wrapper::{Json, Parameters};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct SystemInfoArgs {}

#[derive(Debug, Serialize, JsonSchema)]
pub struct SystemInfoOutput {
    pub hostname: String,
    pub os_version: String,
pub total_memory_mb: i64,
pub cpu_count: i32,
}

pub fn system_info_inner(_args: SystemInfoArgs) -> anyhow::Result<SystemInfoOutput> {
    use std::process::Command;

    let hostname = Command::new("hostname")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let os_version = Command::new("cmd")
        .args(["/C", "ver"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let total_memory = get_total_memory_mb();
    let cpu_count = get_cpu_count();

    Ok(SystemInfoOutput {
        hostname,
        os_version,
        total_memory_mb: total_memory as i64,
        cpu_count: cpu_count as i32,
    })
}

fn get_total_memory_mb() -> u64 {
    use std::process::Command;
    let output = Command::new("cmd")
        .args([
            "/C",
            "wmic memorychip get capacity /format:value",
        ])
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let mut total: u64 = 0;
            for line in stdout.lines() {
                if let Some(val) = line.strip_prefix("Capacity=") {
                    if let Ok(bytes) = val.trim().parse::<u64>() {
                        total += bytes / (1024 * 1024);
                    }
                }
            }
            total
        }
        Err(_) => 0,
    }
}

fn get_cpu_count() -> u32 {
    std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(1)
}

#[tracing::instrument(skip_all)]
pub fn system_info(Parameters(args): Parameters<SystemInfoArgs>) -> Json<SystemInfoOutput> {
    match system_info_inner(args) {
        Ok(output) => Json(output),
        Err(e) => {
            tracing::error!("system_info failed: {}", e);
            Json(SystemInfoOutput {
                hostname: "error".to_string(),
                os_version: "error".to_string(),
                total_memory_mb: 0,
                cpu_count: 0,
            })
        }
    }
}
