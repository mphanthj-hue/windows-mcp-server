use rmcp::handler::server::wrapper::{Json, Parameters};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct DiskInfoArgs {}

#[derive(Debug, Serialize, JsonSchema)]
pub struct DiskInfo {
    pub drive: String,
    pub volume_name: String,
    pub file_system: String,
    pub total_size_gb: f64,
    pub used_size_gb: f64,
    pub free_size_gb: f64,
    pub used_percent: f64,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct DiskInfoOutput {
    pub disks: Vec<DiskInfo>,
    pub total_disks: usize,
}

pub fn disk_info_inner(_args: DiskInfoArgs) -> DiskInfoOutput {
    let output = Command::new("powershell")
        .args(["-Command", "Get-Volume | Where-Object { $_.DriveLetter } | Select-Object DriveLetter, FileSystemLabel, FileSystem, Size, SizeRemaining | ConvertTo-Json"])
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute powershell"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let disks: Vec<DiskInfo> = if stdout.trim().is_empty() {
        Vec::new()
    } else if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&stdout) {
        match parsed {
            serde_json::Value::Array(arr) => arr
                .iter()
                .filter_map(|v| {
                    let size = v["Size"].as_f64().unwrap_or(0.0);
                    let remaining = v["SizeRemaining"].as_f64().unwrap_or(0.0);
                    let used = size - remaining;
                    let percent = if size > 0.0 { (used / size) * 100.0 } else { 0.0 };
                    Some(DiskInfo {
                        drive: format!("{}:", v["DriveLetter"].as_str()?),
                        volume_name: v["FileSystemLabel"].as_str().unwrap_or("").to_string(),
                        file_system: v["FileSystem"].as_str().unwrap_or("Unknown").to_string(),
                        total_size_gb: (size / 1024.0 / 1024.0 / 1024.0 * 100.0).round() / 100.0,
                        used_size_gb: (used / 1024.0 / 1024.0 / 1024.0 * 100.0).round() / 100.0,
                        free_size_gb: (remaining / 1024.0 / 1024.0 / 1024.0 * 100.0).round() / 100.0,
                        used_percent: (percent * 100.0).round() / 100.0,
                    })
                })
                .collect(),
            serde_json::Value::Object(obj) => {
                if let Some(drive_letter) = obj.get("DriveLetter").and_then(|v| v.as_str()) {
                    let size = obj.get("Size").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let remaining = obj.get("SizeRemaining").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let used = size - remaining;
                    let percent = if size > 0.0 { (used / size) * 100.0 } else { 0.0 };
                    vec![DiskInfo {
                        drive: format!("{}:", drive_letter),
                        volume_name: obj.get("FileSystemLabel").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        file_system: obj.get("FileSystem").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                        total_size_gb: (size / 1024.0 / 1024.0 / 1024.0 * 100.0).round() / 100.0,
                        used_size_gb: (used / 1024.0 / 1024.0 / 1024.0 * 100.0).round() / 100.0,
                        free_size_gb: (remaining / 1024.0 / 1024.0 / 1024.0 * 100.0).round() / 100.0,
                        used_percent: (percent * 100.0).round() / 100.0,
                    }]
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new(),
        }
    } else {
        Vec::new()
    };

    let total_disks = disks.len();
    DiskInfoOutput {
        disks,
        total_disks,
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CleanupDiskArgs {
    pub drive: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CleanupDiskOutput {
    pub success: bool,
    pub message: String,
    pub freed_space_gb: Option<f64>,
}

pub fn cleanup_disk_inner(args: CleanupDiskArgs) -> CleanupDiskOutput {
    let script = format!(
        "$before = (Get-Volume -DriveLetter '{}').SizeRemaining; \
         Clear-RecycleBin -DriveLetter '{}' -Force -ErrorAction SilentlyContinue; \
         Remove-Item -Path 'C:\\Windows\\Temp\\*' -Recurse -Force -ErrorAction SilentlyContinue; \
         $after = (Get-Volume -DriveLetter '{}').SizeRemaining; \
         $freed = ($after - $before) / 1GB; \
         Write-Output \"FREED:$freed\"",
        args.drive, args.drive, args.drive
    );

    let output = Command::new("powershell")
        .args(["-Command", &script])
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute powershell"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let freed = stdout
        .lines()
        .find(|l| l.starts_with("FREED:"))
        .and_then(|l| l.strip_prefix("FREED:"))
        .and_then(|s| s.trim().parse::<f64>().ok());

    if output.status.success() {
        CleanupDiskOutput {
            success: true,
            message: format!("Cleanup completed for drive {}:", args.drive),
            freed_space_gb: freed.map(|f| (f * 100.0).round() / 100.0),
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        CleanupDiskOutput {
            success: false,
            message: format!("Cleanup failed for drive {}: {}", args.drive, stderr.trim()),
            freed_space_gb: None,
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn disk_info(Parameters(_args): Parameters<DiskInfoArgs>) -> Json<DiskInfoOutput> {
    Json(disk_info_inner(_args))
}

#[tracing::instrument(skip_all)]
pub fn cleanup_disk(Parameters(args): Parameters<CleanupDiskArgs>) -> Json<CleanupDiskOutput> {
    Json(cleanup_disk_inner(args))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn disk_info_returns_non_empty() {
        let output = disk_info_inner(DiskInfoArgs {});
        assert!(output.total_disks > 0, "should have at least one disk");
    }
}
