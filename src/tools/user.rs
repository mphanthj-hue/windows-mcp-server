use rmcp::handler::server::wrapper::{Json, Parameters};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct ListUsersArgs {}

#[derive(Debug, Serialize, JsonSchema)]
pub struct UserInfo {
    pub name: String,
    pub full_name: String,
    pub enabled: bool,
    pub last_logon: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ListUsersOutput {
    pub users: Vec<UserInfo>,
    pub total_count: usize,
}

pub fn list_users_inner(_args: ListUsersArgs) -> ListUsersOutput {
    let output = Command::new("powershell")
        .args(["-Command", "Get-LocalUser | Select-Object Name, FullName, Enabled, LastLogon | ConvertTo-Json"])
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute powershell"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let users: Vec<UserInfo> = if stdout.trim().is_empty() {
        Vec::new()
    } else if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&stdout) {
        match parsed {
            serde_json::Value::Array(arr) => arr
                .iter()
                .filter_map(|v| {
                    Some(UserInfo {
                        name: v["Name"].as_str()?.to_string(),
                        full_name: v["FullName"].as_str().unwrap_or("").to_string(),
                        enabled: v["Enabled"].as_bool().unwrap_or(false),
                        last_logon: v["LastLogon"].as_str().map(String::from),
                    })
                })
                .collect(),
            serde_json::Value::Object(obj) => {
                if let Some(name) = obj.get("Name").and_then(|v| v.as_str()) {
                    vec![UserInfo {
                        name: name.to_string(),
                        full_name: obj.get("FullName").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        enabled: obj.get("Enabled").and_then(|v| v.as_bool()).unwrap_or(false),
                        last_logon: obj.get("LastLogon").and_then(|v| v.as_str()).map(String::from),
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

    let total_count = users.len();
    ListUsersOutput {
        users,
        total_count,
    }
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct ListGroupsArgs {}

#[derive(Debug, Serialize, JsonSchema)]
pub struct GroupInfo {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ListGroupsOutput {
    pub groups: Vec<GroupInfo>,
    pub total_count: usize,
}

pub fn list_groups_inner(_args: ListGroupsArgs) -> ListGroupsOutput {
    let output = Command::new("powershell")
        .args(["-Command", "Get-LocalGroup | Select-Object Name, Description | ConvertTo-Json"])
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute powershell"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let groups: Vec<GroupInfo> = if stdout.trim().is_empty() {
        Vec::new()
    } else if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&stdout) {
        match parsed {
            serde_json::Value::Array(arr) => arr
                .iter()
                .filter_map(|v| {
                    Some(GroupInfo {
                        name: v["Name"].as_str()?.to_string(),
                        description: v["Description"].as_str().unwrap_or("").to_string(),
                    })
                })
                .collect(),
            serde_json::Value::Object(obj) => {
                if let Some(name) = obj.get("Name").and_then(|v| v.as_str()) {
                    vec![GroupInfo {
                        name: name.to_string(),
                        description: obj.get("Description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
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

    let total_count = groups.len();
    ListGroupsOutput {
        groups,
        total_count,
    }
}

#[tracing::instrument(skip_all)]
pub fn list_users(Parameters(args): Parameters<ListUsersArgs>) -> Json<ListUsersOutput> {
    Json(list_users_inner(args))
}

#[tracing::instrument(skip_all)]
pub fn list_groups(Parameters(args): Parameters<ListGroupsArgs>) -> Json<ListGroupsOutput> {
    Json(list_groups_inner(args))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn list_users_returns_non_empty() {
        let output = list_users_inner(ListUsersArgs {});
        assert!(output.total_count > 0, "should have at least one user");
    }

    #[test]
    fn list_groups_returns_non_empty() {
        let output = list_groups_inner(ListGroupsArgs {});
        assert!(output.total_count > 0, "should have at least one group");
    }
}
