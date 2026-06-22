use rmcp::handler::server::wrapper::{Json, Parameters};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct ListStartupArgs {}

#[derive(Debug, Serialize, JsonSchema)]
pub struct StartupEntry {
    pub name: String,
    pub command: String,
    pub location: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ListStartupOutput {
    pub entries: Vec<StartupEntry>,
    pub total_count: i32,
}

pub fn list_startup_inner(_args: ListStartupArgs) -> ListStartupOutput {
    let script = r#"
        $startup = @()
        $startup += Get-CimInstance Win32_StartupCommand -ErrorAction SilentlyContinue | ForEach-Object {
            [PSCustomObject]@{ Name = $_.Name; Command = $_.Command; Location = $_.Location }
        }
        $startup | ConvertTo-Json
    "#;

    let output = Command::new("powershell")
        .args(["-Command", script])
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute powershell"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let entries: Vec<StartupEntry> = if stdout.trim().is_empty() {
        Vec::new()
    } else if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&stdout) {
        match parsed {
            serde_json::Value::Array(arr) => arr
                .iter()
                .filter_map(|v| {
                    Some(StartupEntry {
                        name: v["Name"].as_str()?.to_string(),
                        command: v["Command"].as_str().unwrap_or("").to_string(),
                        location: v["Location"].as_str().unwrap_or("").to_string(),
                    })
                })
                .collect(),
            serde_json::Value::Object(obj) => {
                if let Some(name) = obj.get("Name").and_then(|v| v.as_str()) {
                    vec![StartupEntry {
                        name: name.to_string(),
                        command: obj.get("Command").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        location: obj.get("Location").and_then(|v| v.as_str()).unwrap_or("").to_string(),
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

    let total_count = entries.len() as i32;
    ListStartupOutput {
        entries,
        total_count,
    }
}

#[tracing::instrument(skip_all)]
pub fn list_startup(Parameters(args): Parameters<ListStartupArgs>) -> Json<ListStartupOutput> {
    Json(list_startup_inner(args))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn list_startup_returns_list() {
        let output = list_startup_inner(ListStartupArgs {});
        assert!(output.total_count > 0, "should have startup entries");
    }
}
