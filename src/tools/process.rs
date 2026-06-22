use rmcp::handler::server::wrapper::{Json, Parameters};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sysinfo::{Pid, System};

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct ListProcessesArgs {
    pub filter_name: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ProcessInfo {
    pub pid: i32,
    pub name: String,
    pub status: String,
    pub memory_mb: i64,
    pub cpu_usage: f32,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ListProcessesOutput {
    pub processes: Vec<ProcessInfo>,
    pub total_count: i32,
}

pub fn list_processes_inner(args: ListProcessesArgs) -> ListProcessesOutput {
    let mut sys = System::new();
    sys.refresh_processes();

    let mut processes: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .map(|(pid, process)| {
            ProcessInfo {
                pid: pid.as_u32() as i32,
                name: process.name().to_string(),
                status: format!("{:?}", process.status()),
                memory_mb: process.memory() as i64 / 1024 / 1024,
                cpu_usage: process.cpu_usage(),
            }
        })
        .collect();

    if let Some(filter) = args.filter_name {
        let filter_lower = filter.to_lowercase();
        processes.retain(|p| p.name.to_lowercase().contains(&filter_lower));
    }

    processes.sort_by(|a, b| b.memory_mb.cmp(&a.memory_mb));

    let total_count = processes.len() as i32;
    ListProcessesOutput {
        processes,
        total_count,
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KillProcessArgs {
    pub pid: Option<u32>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct KillProcessOutput {
    pub success: bool,
    pub message: String,
}

pub fn kill_process_inner(args: KillProcessArgs) -> KillProcessOutput {
    let mut sys = System::new();
    sys.refresh_processes();

    if let Some(pid) = args.pid {
        let sys_pid = Pid::from_u32(pid);
        if let Some(process) = sys.process(sys_pid) {
            let name = process.name().to_string();
            match process.kill() {
                true => KillProcessOutput {
                    success: true,
                    message: format!("Killed process {} (PID: {})", name, pid),
                },
                false => KillProcessOutput {
                    success: false,
                    message: format!("Failed to kill process {} (PID: {})", name, pid),
                },
            }
        } else {
            KillProcessOutput {
                success: false,
                message: format!("Process with PID {} not found", pid),
            }
        }
    } else if let Some(name) = args.name {
        let mut killed = 0;
        let mut _failed = 0;
        let name_lower = name.to_lowercase();

        for (_pid, process) in sys.processes() {
            if process.name().to_lowercase() == name_lower {
                match process.kill() {
                    true => killed += 1,
                    false => _failed += 1,
                }
            }
        }

        if killed > 0 {
            KillProcessOutput {
                success: true,
                message: format!("Killed {} process(es) named '{}'", killed, name),
            }
        } else {
            KillProcessOutput {
                success: false,
                message: format!("No process named '{}' found or all kill attempts failed", name),
            }
        }
    } else {
        KillProcessOutput {
            success: false,
            message: "Either pid or name must be provided".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StartProcessArgs {
    pub command: String,
    pub args: Option<Vec<String>>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct StartProcessOutput {
    pub success: bool,
    pub pid: Option<u32>,
    pub message: String,
}

pub fn start_process_inner(args: StartProcessArgs) -> StartProcessOutput {
    use std::process::Command;

    let mut cmd = Command::new(&args.command);
    if let Some(extra_args) = args.args {
        cmd.args(&extra_args);
    }

    match cmd.spawn() {
        Ok(child) => StartProcessOutput {
            success: true,
            pid: Some(child.id()),
            message: format!("Started process '{}' with PID {}", args.command, child.id()),
        },
        Err(e) => StartProcessOutput {
            success: false,
            pid: None,
            message: format!("Failed to start process '{}': {}", args.command, e),
        },
    }
}

#[tracing::instrument(skip_all)]
pub fn list_processes(
    Parameters(args): Parameters<ListProcessesArgs>,
) -> Json<ListProcessesOutput> {
    Json(list_processes_inner(args))
}

#[tracing::instrument(skip_all)]
pub fn kill_process(Parameters(args): Parameters<KillProcessArgs>) -> Json<KillProcessOutput> {
    Json(kill_process_inner(args))
}

#[tracing::instrument(skip_all)]
pub fn start_process(Parameters(args): Parameters<StartProcessArgs>) -> Json<StartProcessOutput> {
    Json(start_process_inner(args))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn list_processes_returns_non_empty() {
        let output = list_processes_inner(ListProcessesArgs::default());
        assert!(output.total_count > 0, "should have at least one process");
    }

    #[test]
    fn list_processes_filter_by_name() {
        let output = list_processes_inner(ListProcessesArgs {
            filter_name: Some("system".to_string()),
        });
        for process in &output.processes {
            assert!(
                process.name.to_lowercase().contains("system"),
                "filtered process should contain 'system'"
            );
        }
    }

    #[test]
    fn kill_nonexistent_pid_fails() {
        let output = kill_process_inner(KillProcessArgs {
            pid: Some(999999),
            name: None,
        });
        assert!(!output.success);
    }
}
