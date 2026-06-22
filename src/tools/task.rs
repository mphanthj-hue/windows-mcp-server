use rmcp::handler::server::wrapper::{Json, Parameters};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct ListTasksArgs {}

#[derive(Debug, Serialize, JsonSchema)]
pub struct TaskInfo {
    pub name: String,
    pub state: String,
    pub next_run: Option<String>,
    pub last_run: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ListTasksOutput {
    pub tasks: Vec<TaskInfo>,
    pub total_count: usize,
}

pub fn list_tasks_inner(_args: ListTasksArgs) -> ListTasksOutput {
    let output = Command::new("powershell")
        .args(["-Command", "Get-ScheduledTask | Where-Object { $_.TaskPath -notlike '\\Microsoft\\*' } | Select-Object TaskName, State, @{N='NextRunTime';E={$_.NextRunTime}}, @{N='LastRunTime';E={$_.LastRunTime}} | ConvertTo-Json"])
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute powershell"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let tasks: Vec<TaskInfo> = if stdout.trim().is_empty() {
        Vec::new()
    } else if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&stdout) {
        match parsed {
            serde_json::Value::Array(arr) => arr
                .iter()
                .filter_map(|v| {
                    Some(TaskInfo {
                        name: v["TaskName"].as_str()?.to_string(),
                        state: v["State"].as_str().unwrap_or("Unknown").to_string(),
                        next_run: v["NextRunTime"].as_str().map(String::from),
                        last_run: v["LastRunTime"].as_str().map(String::from),
                    })
                })
                .collect(),
            serde_json::Value::Object(obj) => {
                if let Some(name) = obj.get("TaskName").and_then(|v| v.as_str()) {
                    vec![TaskInfo {
                        name: name.to_string(),
                        state: obj.get("State").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                        next_run: obj.get("NextRunTime").and_then(|v| v.as_str()).map(String::from),
                        last_run: obj.get("LastRunTime").and_then(|v| v.as_str()).map(String::from),
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

    let total_count = tasks.len();
    ListTasksOutput {
        tasks,
        total_count,
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RunTaskArgs {
    pub name: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct RunTaskOutput {
    pub success: bool,
    pub message: String,
}

pub fn run_task_inner(args: RunTaskArgs) -> RunTaskOutput {
    let output = Command::new("powershell")
        .args(["-Command", &format!("Start-ScheduledTask -TaskName '{}' -ErrorAction Stop", args.name)])
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute powershell"));

    if output.status.success() {
        RunTaskOutput {
            success: true,
            message: format!("Task '{}' started successfully", args.name),
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        RunTaskOutput {
            success: false,
            message: format!("Failed to start task '{}': {}", args.name, stderr.trim()),
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn list_tasks(Parameters(args): Parameters<ListTasksArgs>) -> Json<ListTasksOutput> {
    Json(list_tasks_inner(args))
}

#[tracing::instrument(skip_all)]
pub fn run_task(Parameters(args): Parameters<RunTaskArgs>) -> Json<RunTaskOutput> {
    Json(run_task_inner(args))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn list_tasks_returns_list() {
        let output = list_tasks_inner(ListTasksArgs {});
        assert!(output.total_count > 0, "should have at least one task");
    }

    #[test]
    fn run_nonexistent_task_fails() {
        let output = run_task_inner(RunTaskArgs {
            name: "NonExistentTask12345".to_string(),
        });
        assert!(!output.success);
    }
}
