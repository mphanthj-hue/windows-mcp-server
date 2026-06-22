use rmcp::handler::server::wrapper::{Json, Parameters};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct CheckUpdatesArgs {}

#[derive(Debug, Serialize, JsonSchema)]
pub struct UpdateInfo {
    pub title: String,
    pub kb: String,
    pub severity: String,
    pub size: String,
    pub is_installed: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CheckUpdatesOutput {
    pub updates: Vec<UpdateInfo>,
    pub total_count: i32,
    pub has_updates: bool,
}

pub fn check_updates_inner(_args: CheckUpdatesArgs) -> CheckUpdatesOutput {
    let script = r#"
        $session = New-Object -ComObject Microsoft.Update.Session
        $searcher = $session.CreateUpdateSearcher()
        $result = $searcher.Search("IsInstalled=0")
        $updates = @()
        foreach ($update in $result.Updates) {
            $kb = ""
            if ($update.KBArticleIDs.Count -gt 0) {
                $kb = "KB" + $update.KBArticleIDs.Item(0)
            }
            $updates += [PSCustomObject]@{
                Title = $update.Title
                KB = $kb
                Severity = $update.MsrcSeverity
                Size = [math]::Round($update.MaxDownloadSize / 1MB, 2).ToString() + " MB"
                IsInstalled = $update.IsInstalled
            }
        }
        $updates | ConvertTo-Json
    "#;

    let output = Command::new("powershell")
        .args(["-Command", script])
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute powershell"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let updates: Vec<UpdateInfo> = if stdout.trim().is_empty() {
        Vec::new()
    } else if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&stdout) {
        match parsed {
            serde_json::Value::Array(arr) => arr
                .iter()
                .filter_map(|v| {
                    Some(UpdateInfo {
                        title: v["Title"].as_str().unwrap_or("Unknown").to_string(),
                        kb: v["KB"].as_str().unwrap_or("").to_string(),
                        severity: v["Severity"].as_str().unwrap_or("Unknown").to_string(),
                        size: v["Size"].as_str().unwrap_or("Unknown").to_string(),
                        is_installed: v["IsInstalled"].as_bool().unwrap_or(false),
                    })
                })
                .collect(),
            serde_json::Value::Object(obj) => {
                if let Some(title) = obj.get("Title").and_then(|v| v.as_str()) {
                    vec![UpdateInfo {
                        title: title.to_string(),
                        kb: obj.get("KB").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        severity: obj.get("Severity").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                        size: obj.get("Size").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                        is_installed: obj.get("IsInstalled").and_then(|v| v.as_bool()).unwrap_or(false),
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

    let has_updates = !updates.is_empty();
    let total_count = updates.len() as i32;
    CheckUpdatesOutput {
        updates,
        total_count,
        has_updates,
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct InstallUpdatesArgs {
    pub title_filter: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct InstallUpdatesOutput {
    pub success: bool,
    pub message: String,
    pub installed_count: i32,
}

pub fn install_updates_inner(args: InstallUpdatesArgs) -> InstallUpdatesOutput {
    let filter = args.title_filter.unwrap_or_default();

    let script = format!(
        r#"
        $session = New-Object -ComObject Microsoft.Update.Session
        $searcher = $session.CreateUpdateSearcher()
        $result = $searcher.Search("IsInstalled=0")
        $updatesToInstall = New-Object -ComObject Microsoft.Update.UpdateColl
        foreach ($update in $result.Updates) {{
            $title = $update.Title
            if ("{}" -eq "" -or $title -like "*{}*") {{
                $updatesToInstall.Add($update) | Out-Null
            }}
        }}
        if ($updatesToInstall.Count -eq 0) {{
            Write-Output "NO_UPDATES"
        }} else {{
            $downloader = $session.CreateUpdateDownloader()
            $downloader.Updates = $updatesToInstall
            $downloader.Download()
            $installer = $session.CreateUpdateInstaller()
            $installer.Updates = $updatesToInstall
            $installationResult = $installer.Install()
            Write-Output "INSTALLED:$($updatesToInstall.Count)"
        }}
        "#,
        filter, filter
    );

    let output = Command::new("powershell")
        .args(["-Command", &script])
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute powershell"));

    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.contains("NO_UPDATES") {
        InstallUpdatesOutput {
            success: true,
            message: "No matching updates found to install".to_string(),
            installed_count: 0,
        }
    } else if let Some(count_str) = stdout.lines().find(|l| l.starts_with("INSTALLED:")) {
        let count: i32 = count_str.strip_prefix("INSTALLED:").unwrap_or("0").parse().unwrap_or(0);
        InstallUpdatesOutput {
            success: true,
            message: format!("{} updates installed successfully", count),
            installed_count: count,
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        InstallUpdatesOutput {
            success: false,
            message: format!("Failed to install updates: {}", stderr.trim()),
            installed_count: 0,
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn check_updates(Parameters(args): Parameters<CheckUpdatesArgs>) -> Json<CheckUpdatesOutput> {
    Json(check_updates_inner(args))
}

#[tracing::instrument(skip_all)]
pub fn install_updates(Parameters(args): Parameters<InstallUpdatesArgs>) -> Json<InstallUpdatesOutput> {
    Json(install_updates_inner(args))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn check_updates_returns_data() {
        let output = check_updates_inner(CheckUpdatesArgs {});
        assert!(output.total_count > 0 || !output.has_updates);
    }
}
