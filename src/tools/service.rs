use rmcp::handler::server::wrapper::{Json, Parameters};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct ListServicesArgs {
    pub filter_name: Option<String>,
    pub filter_status: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ServiceInfo {
    pub name: String,
    pub display_name: String,
    pub status: String,
    pub startup_type: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ListServicesOutput {
    pub services: Vec<ServiceInfo>,
    pub total_count: usize,
}

pub fn list_services_inner(args: ListServicesArgs) -> ListServicesOutput {
    let output = Command::new("powershell")
        .args(["-Command", "Get-Service | Select-Object Name, DisplayName, Status, StartType | ConvertTo-Json"])
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute powershell"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let services: Vec<ServiceInfo> = if stdout.trim().is_empty() {
        Vec::new()
    } else if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&stdout) {
        match parsed {
            serde_json::Value::Array(arr) => arr
                .iter()
                .filter_map(|v| {
                    Some(ServiceInfo {
                        name: v["Name"].as_str()?.to_string(),
                        display_name: v["DisplayName"].as_str().unwrap_or("").to_string(),
                        status: v["Status"].as_str().unwrap_or("Unknown").to_string(),
                        startup_type: v["StartType"].as_str().unwrap_or("Unknown").to_string(),
                    })
                })
                .collect(),
            serde_json::Value::Object(obj) => {
                if let Some(name) = obj.get("Name").and_then(|v| v.as_str()) {
                    vec![ServiceInfo {
                        name: name.to_string(),
                        display_name: obj.get("DisplayName").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        status: obj.get("Status").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                        startup_type: obj.get("StartType").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
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

    let mut filtered = services;
    if let Some(filter) = args.filter_name {
        let filter_lower = filter.to_lowercase();
        filtered.retain(|s| s.name.to_lowercase().contains(&filter_lower));
    }
    if let Some(status_filter) = args.filter_status {
        let status_lower = status_filter.to_lowercase();
        filtered.retain(|s| s.status.to_lowercase().contains(&status_lower));
    }

    filtered.sort_by(|a, a2| a.name.cmp(&a2.name));

    let total_count = filtered.len();
    ListServicesOutput {
        services: filtered,
        total_count,
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ServiceActionArgs {
    pub name: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ServiceActionOutput {
    pub success: bool,
    pub message: String,
}

pub fn start_service_inner(args: ServiceActionArgs) -> ServiceActionOutput {
    let output = Command::new("powershell")
        .args(["-Command", &format!("Start-Service -Name '{}' | Out-Null", args.name)])
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute powershell"));

    if output.status.success() {
        ServiceActionOutput {
            success: true,
            message: format!("Service '{}' started successfully", args.name),
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        ServiceActionOutput {
            success: false,
            message: format!("Failed to start service '{}': {}", args.name, stderr.trim()),
        }
    }
}

pub fn stop_service_inner(args: ServiceActionArgs) -> ServiceActionOutput {
    let output = Command::new("powershell")
        .args(["-Command", &format!("Stop-Service -Name '{}' -Force | Out-Null", args.name)])
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute powershell"));

    if output.status.success() {
        ServiceActionOutput {
            success: true,
            message: format!("Service '{}' stopped successfully", args.name),
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        ServiceActionOutput {
            success: false,
            message: format!("Failed to stop service '{}': {}", args.name, stderr.trim()),
        }
    }
}

pub fn restart_service_inner(args: ServiceActionArgs) -> ServiceActionOutput {
    let output = Command::new("powershell")
        .args(["-Command", &format!("Restart-Service -Name '{}' -Force | Out-Null", args.name)])
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute powershell"));

    if output.status.success() {
        ServiceActionOutput {
            success: true,
            message: format!("Service '{}' restarted successfully", args.name),
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        ServiceActionOutput {
            success: false,
            message: format!("Failed to restart service '{}': {}", args.name, stderr.trim()),
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn list_services(Parameters(args): Parameters<ListServicesArgs>) -> Json<ListServicesOutput> {
    Json(list_services_inner(args))
}

#[tracing::instrument(skip_all)]
pub fn start_service(Parameters(args): Parameters<ServiceActionArgs>) -> Json<ServiceActionOutput> {
    Json(start_service_inner(args))
}

#[tracing::instrument(skip_all)]
pub fn stop_service(Parameters(args): Parameters<ServiceActionArgs>) -> Json<ServiceActionOutput> {
    Json(stop_service_inner(args))
}

#[tracing::instrument(skip_all)]
pub fn restart_service(Parameters(args): Parameters<ServiceActionArgs>) -> Json<ServiceActionOutput> {
    Json(restart_service_inner(args))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn list_services_returns_non_empty() {
        let output = list_services_inner(ListServicesArgs::default());
        assert!(output.total_count > 0, "should have at least one service");
    }

    #[test]
    fn list_services_filter_by_name() {
        let output = list_services_inner(ListServicesArgs {
            filter_name: Some("Win".to_string()),
            filter_status: None,
        });
        for service in &output.services {
            assert!(
                service.name.to_lowercase().contains("win"),
                "filtered service should contain 'Win'"
            );
        }
    }

    #[test]
    fn list_services_filter_by_status() {
        let output = list_services_inner(ListServicesArgs {
            filter_name: None,
            filter_status: Some("Running".to_string()),
        });
        for service in &output.services {
            assert_eq!(service.status, "Running");
        }
    }

    #[test]
    fn start_nonexistent_service_fails() {
        let output = start_service_inner(ServiceActionArgs {
            name: "NonExistentService12345".to_string(),
        });
        assert!(!output.success);
    }

    #[test]
    fn stop_nonexistent_service_fails() {
        let output = stop_service_inner(ServiceActionArgs {
            name: "NonExistentService12345".to_string(),
        });
        assert!(!output.success);
    }
}
