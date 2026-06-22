use rmcp::handler::server::wrapper::{Json, Parameters};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct ReadEventLogArgs {
    pub log_name: Option<String>,
    pub max_entries: Option<u32>,
    pub level: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct EventLogEntry {
    pub time_created: String,
    pub id: u32,
    pub level: String,
    pub source: String,
    pub message: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ReadEventLogOutput {
    pub entries: Vec<EventLogEntry>,
    pub total_count: usize,
}

pub fn read_event_log_inner(args: ReadEventLogArgs) -> ReadEventLogOutput {
    let log_name = args.log_name.unwrap_or_else(|| "System".to_string());
    let max_entries = args.max_entries.unwrap_or(50);

    let level_filter = args.level
        .map(|l| format!("| Where-Object {{ $_.LevelDisplayName -eq '{}' }}", l))
        .unwrap_or_default();

    let script = format!(
        "Get-WinEvent -LogName '{}' -MaxEvents {} {} -ErrorAction SilentlyContinue | Select-Object TimeCreated, Id, LevelDisplayName, ProviderName, Message | ConvertTo-Json",
        log_name, max_entries, level_filter
    );

    let output = Command::new("powershell")
        .args(["-Command", &script])
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute powershell"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let entries: Vec<EventLogEntry> = if stdout.trim().is_empty() {
        Vec::new()
    } else if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&stdout) {
        let entry_list = match parsed {
            serde_json::Value::Array(arr) => arr,
            serde_json::Value::Object(obj) => vec![serde_json::Value::Object(obj)],
            _ => return ReadEventLogOutput { entries: Vec::new(), total_count: 0 },
        };

        entry_list
            .iter()
            .filter_map(|v| {
                let time = v["TimeCreated"].as_str().unwrap_or("").to_string();
                let message = v["Message"].as_str().unwrap_or("").to_string();
                let truncated = if message.len() > 200 {
                    format!("{}...", &message[..200])
                } else {
                    message
                };
                Some(EventLogEntry {
                    time_created: time,
                    id: v["Id"].as_u64().unwrap_or(0) as u32,
                    level: v["LevelDisplayName"].as_str().unwrap_or("Unknown").to_string(),
                    source: v["ProviderName"].as_str().unwrap_or("Unknown").to_string(),
                    message: truncated,
                })
            })
            .collect()
    } else {
        Vec::new()
    };

    let total_count = entries.len();
    ReadEventLogOutput {
        entries,
        total_count,
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ClearEventLogArgs {
    pub log_name: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ClearEventLogOutput {
    pub success: bool,
    pub message: String,
}

pub fn clear_event_log_inner(args: ClearEventLogArgs) -> ClearEventLogOutput {
    let output = Command::new("powershell")
        .args(["-Command", &format!("Clear-EventLog -LogName '{}' -ErrorAction Stop", args.log_name)])
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute powershell"));

    if output.status.success() {
        ClearEventLogOutput {
            success: true,
            message: format!("Event log '{}' cleared successfully", args.log_name),
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        ClearEventLogOutput {
            success: false,
            message: format!("Failed to clear event log '{}': {}", args.log_name, stderr.trim()),
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn read_event_log(Parameters(args): Parameters<ReadEventLogArgs>) -> Json<ReadEventLogOutput> {
    Json(read_event_log_inner(args))
}

#[tracing::instrument(skip_all)]
pub fn clear_event_log(Parameters(args): Parameters<ClearEventLogArgs>) -> Json<ClearEventLogOutput> {
    Json(clear_event_log_inner(args))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn read_event_log_returns_entries() {
        let output = read_event_log_inner(ReadEventLogArgs {
            log_name: Some("System".to_string()),
            max_entries: Some(10),
            level: None,
        });
        assert!(output.total_count > 0, "should have at least one event log entry");
    }

    #[test]
    fn clear_nonexistent_log_fails() {
        let output = clear_event_log_inner(ClearEventLogArgs {
            log_name: "NonExistentLog12345".to_string(),
        });
        assert!(!output.success);
    }
}
