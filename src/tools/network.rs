use rmcp::handler::server::wrapper::{Json, Parameters};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct NetworkAdaptersArgs {}

#[derive(Debug, Serialize, JsonSchema)]
pub struct NetworkAdapter {
    pub name: String,
    pub description: String,
    pub mac_address: String,
    pub status: String,
    pub speed_mbps: i64,
    pub ipv4_addresses: Vec<String>,
    pub ipv6_addresses: Vec<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct NetworkAdaptersOutput {
    pub adapters: Vec<NetworkAdapter>,
    pub total_count: i32,
}

pub fn network_adapters_inner(_args: NetworkAdaptersArgs) -> NetworkAdaptersOutput {
    let output = Command::new("powershell")
        .args(["-Command", "Get-NetAdapter | Select-Object Name, InterfaceDescription, MacAddress, Status, LinkSpeed, InterfaceIndex | ConvertTo-Json"])
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute powershell"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let adapters: Vec<NetworkAdapter> = if stdout.trim().is_empty() {
        Vec::new()
    } else if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&stdout) {
        let adapter_list = match parsed {
            serde_json::Value::Array(arr) => arr,
            serde_json::Value::Object(obj) => vec![serde_json::Value::Object(obj)],
            _ => return NetworkAdaptersOutput { adapters: Vec::new(), total_count: 0 },
        };

        let ip_output = Command::new("powershell")
            .args(["-Command", "Get-NetIPAddress -AddressFamily IPv4,IPv6 | Select-Object InterfaceAlias, IPAddress | ConvertTo-Json"])
            .output()
            .unwrap_or_else(|_| panic!("Failed to execute powershell"));

        let ip_stdout = String::from_utf8_lossy(&ip_output.stdout);
        let ip_data: Vec<serde_json::Value> = serde_json::from_str(&ip_stdout).unwrap_or_default();

        adapter_list
            .iter()
            .filter_map(|v| {
                let name = v["Name"].as_str()?.to_string();
                let ipv4: Vec<String> = ip_data
                    .iter()
                    .filter(|ip| ip["InterfaceAlias"].as_str() == Some(&name))
                    .filter_map(|ip| ip["IPAddress"].as_str().map(String::from))
                    .filter(|ip| !ip.contains(":"))
                    .collect();
                let ipv6: Vec<String> = ip_data
                    .iter()
                    .filter(|ip| ip["InterfaceAlias"].as_str() == Some(&name))
                    .filter_map(|ip| ip["IPAddress"].as_str().map(String::from))
                    .filter(|ip| ip.contains(":"))
                    .collect();

                let speed_str = v["LinkSpeed"].as_str().unwrap_or("0 Mbps");
                let speed: i64 = speed_str
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse()
                    .unwrap_or(0);

                Some(NetworkAdapter {
                    name,
                    description: v["InterfaceDescription"].as_str().unwrap_or("").to_string(),
                    mac_address: v["MacAddress"].as_str().unwrap_or("").to_string(),
                    status: v["Status"].as_str().unwrap_or("Unknown").to_string(),
                    speed_mbps: speed,
                    ipv4_addresses: ipv4,
                    ipv6_addresses: ipv6,
                })
            })
            .collect()
    } else {
        Vec::new()
    };

    let total_count = adapters.len() as i32;
    NetworkAdaptersOutput {
        adapters,
        total_count,
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PingArgs {
    pub host: String,
    pub count: Option<u32>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct PingOutput {
    pub host: String,
    pub success: bool,
    pub average_ms: Option<f64>,
    pub packets_sent: i32,
    pub packets_received: i32,
}

pub fn ping_inner(args: PingArgs) -> PingOutput {
    let count = args.count.unwrap_or(4);
    let output = Command::new("ping")
        .args(["-n", &count.to_string(), &args.host])
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute ping"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let success = output.status.success();

    let average_ms = stdout
        .lines()
        .find(|l| l.contains("Average"))
        .and_then(|l| {
            l.split('=').last()?.trim().trim_end_matches("ms").trim().parse::<f64>().ok()
        });

    let packets_sent = stdout
        .lines()
        .find(|l| l.contains("Packets: Sent"))
        .and_then(|l| l.split(',').next()?.split('=').last()?.trim().parse().ok())
        .unwrap_or(0);

    let packets_received = stdout
        .lines()
        .find(|l| l.contains("Received"))
        .and_then(|l| l.split('=').last()?.trim().parse().ok())
        .unwrap_or(0);

    PingOutput {
        host: args.host,
        success,
        average_ms,
        packets_sent,
        packets_received,
    }
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct NetworkConnectionsArgs {}

#[derive(Debug, Serialize, JsonSchema)]
pub struct NetworkConnection {
    pub local_address: String,
    pub local_port: i16,
    pub remote_address: String,
    pub remote_port: i16,
    pub state: String,
    pub process_name: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct NetworkConnectionsOutput {
    pub connections: Vec<NetworkConnection>,
    pub total_count: i32,
}

pub fn network_connections_inner(_args: NetworkConnectionsArgs) -> NetworkConnectionsOutput {
    let output = Command::new("powershell")
        .args(["-Command", "Get-NetTCPConnection | Select-Object LocalAddress, LocalPort, RemoteAddress, RemotePort, State, OwningProcess | ConvertTo-Json"])
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute powershell"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let connections: Vec<NetworkConnection> = if stdout.trim().is_empty() {
        Vec::new()
    } else if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&stdout) {
        let conn_list = match parsed {
            serde_json::Value::Array(arr) => arr,
            serde_json::Value::Object(obj) => vec![serde_json::Value::Object(obj)],
            _ => return NetworkConnectionsOutput { connections: Vec::new(), total_count: 0 },
        };

        conn_list
            .iter()
            .filter_map(|v| {
                let pid = v["OwningProcess"].as_u64()?;
                let process_name = Command::new("powershell")
                    .args(["-Command", &format!("(Get-Process -Id {}).ProcessName", pid)])
                    .output()
                    .ok()
                    .and_then(|o| {
                        let name = String::from_utf8_lossy(&o.stdout).trim().to_string();
                        if name.is_empty() || name.contains("Cannot find a process") {
                            None
                        } else {
                            Some(name)
                        }
                    })
                    .unwrap_or_else(|| "Unknown".to_string());

                Some(NetworkConnection {
                    local_address: v["LocalAddress"].as_str().unwrap_or("").to_string(),
                    local_port: v["LocalPort"].as_i64().unwrap_or(0) as i16,
                    remote_address: v["RemoteAddress"].as_str().unwrap_or("").to_string(),
                    remote_port: v["RemotePort"].as_i64().unwrap_or(0) as i16,
                    state: v["State"].as_str().unwrap_or("Unknown").to_string(),
                    process_name,
                })
            })
            .collect()
    } else {
        Vec::new()
    };

    let total_count = connections.len() as i32;
    NetworkConnectionsOutput {
        connections,
        total_count,
    }
}

#[tracing::instrument(skip_all)]
pub fn network_adapters(Parameters(args): Parameters<NetworkAdaptersArgs>) -> Json<NetworkAdaptersOutput> {
    Json(network_adapters_inner(args))
}

#[tracing::instrument(skip_all)]
pub fn ping(Parameters(args): Parameters<PingArgs>) -> Json<PingOutput> {
    Json(ping_inner(args))
}

#[tracing::instrument(skip_all)]
pub fn network_connections(Parameters(args): Parameters<NetworkConnectionsArgs>) -> Json<NetworkConnectionsOutput> {
    Json(network_connections_inner(args))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn network_adapters_returns_non_empty() {
        let output = network_adapters_inner(NetworkAdaptersArgs {});
        assert!(output.total_count > 0, "should have at least one adapter");
    }

    #[test]
    fn ping_localhost_succeeds() {
        let output = ping_inner(PingArgs {
            host: "127.0.0.1".to_string(),
            count: Some(2),
        });
        assert!(output.success, "ping to localhost should succeed");
    }

    #[test]
    fn network_connections_returns_list() {
        let output = network_connections_inner(NetworkConnectionsArgs {});
        assert!(output.connections.len() > 0, "should have connections");
    }
}
