use rmcp::handler::server::wrapper::{Json, Parameters};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadRegistryArgs {
    pub path: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct RegistryValue {
    pub name: String,
    pub value: String,
    pub value_type: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ReadRegistryOutput {
    pub key: String,
    pub values: Vec<RegistryValue>,
    pub subkeys: Vec<String>,
}

pub fn read_registry_inner(args: ReadRegistryArgs) -> ReadRegistryOutput {
    let normalized_path = args.path.replace('/', "\\");
    let script = format!(
        "Get-ItemProperty -Path 'Registry::{}' -ErrorAction SilentlyContinue | Select-Object -Property * -ExcludeProperty PS* | ConvertTo-Json",
        normalized_path
    );

    let output = Command::new("powershell")
        .args(["-Command", &script])
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute powershell"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let values: Vec<RegistryValue> = if stdout.trim().is_empty() {
        Vec::new()
    } else if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&stdout) {
        match &parsed {
            serde_json::Value::Object(obj) => obj
                .iter()
                .map(|(k, v)| RegistryValue {
                    name: k.clone(),
                    value: v.as_str().unwrap_or(&v.to_string()).to_string(),
                    value_type: match v {
                        serde_json::Value::String(_) => "REG_SZ".to_string(),
                        serde_json::Value::Number(n) => {
                            if n.is_i64() { "REG_DWORD".to_string() } else { "REG_QWORD".to_string() }
                        }
                        serde_json::Value::Bool(_) => "REG_DWORD".to_string(),
                        _ => "REG_SZ".to_string(),
                    },
                })
                .collect(),
            _ => Vec::new(),
        }
    } else {
        Vec::new()
    };

    let subkey_script = format!(
        "Get-ChildItem -Path 'Registry::{}' -ErrorAction SilentlyContinue | ForEach-Object {{ $_.PSChildName }}",
        normalized_path
    );

    let subkey_output = Command::new("powershell")
        .args(["-Command", &subkey_script])
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute powershell"));

    let subkey_stdout = String::from_utf8_lossy(&subkey_output.stdout);
    let subkeys: Vec<String> = subkey_stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_string())
        .collect();

    ReadRegistryOutput {
        key: normalized_path,
        values,
        subkeys,
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WriteRegistryArgs {
    pub path: String,
    pub name: String,
    pub value: String,
    #[serde(default = "default_value_type")]
    pub value_type: String,
}

fn default_value_type() -> String {
    "String".to_string()
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct WriteRegistryOutput {
    pub success: bool,
    pub message: String,
}

pub fn write_registry_inner(args: WriteRegistryArgs) -> WriteRegistryOutput {
    let normalized_path = args.path.replace('/', "\\");
    let ps_value_type = match args.value_type.to_lowercase().as_str() {
        "dword" => "Dword",
        "qword" => "Qword",
        "expandstring" => "ExpandString",
        "multistring" => "MultiString",
        "binary" => "Binary",
        _ => "String",
    };

    let script = format!(
        "New-ItemProperty -Path 'Registry::{}' -Name '{}' -Value '{}' -PropertyType '{}' -Force | Out-Null; Write-Output 'OK'",
        normalized_path, args.name, args.value, ps_value_type
    );

    let output = Command::new("powershell")
        .args(["-Command", &script])
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute powershell"));

    if output.status.success() {
        WriteRegistryOutput {
            success: true,
            message: format!("Written value '{}' to {}", args.name, normalized_path),
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        WriteRegistryOutput {
            success: false,
            message: format!("Failed to write registry: {}", stderr.trim()),
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn read_registry(Parameters(args): Parameters<ReadRegistryArgs>) -> Json<ReadRegistryOutput> {
    Json(read_registry_inner(args))
}

#[tracing::instrument(skip_all)]
pub fn write_registry(Parameters(args): Parameters<WriteRegistryArgs>) -> Json<WriteRegistryOutput> {
    Json(write_registry_inner(args))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn read_registry_hklm_returns_data() {
        let output = read_registry_inner(ReadRegistryArgs {
            path: "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion".to_string(),
        });
        assert!(!output.values.is_empty(), "should have registry values");
    }
}
