//! `ServerHandler` — the rmcp service implementation that owns the
//! template's tool set.
//!
//! Adding a new tool: drop a file in `src/tools/`, declare it in
//! `tools/mod.rs`, then add a `#[tool]` method here that delegates to
//! it. The router macro picks it up automatically — no other wiring.

use rmcp::{
    ServerHandler,
    handler::server::{
        router::tool::ToolRouter,
        wrapper::{Json, Parameters},
    },
    model::{Implementation, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};

use crate::tools::disk::{
    CleanupDiskArgs, CleanupDiskOutput, DiskInfoArgs, DiskInfoOutput,
    cleanup_disk_inner, disk_info_inner,
};
use crate::tools::eventlog::{
    ClearEventLogArgs, ClearEventLogOutput, ReadEventLogArgs, ReadEventLogOutput,
    clear_event_log_inner, read_event_log_inner,
};
use crate::tools::network::{
    NetworkAdaptersArgs, NetworkAdaptersOutput, NetworkConnectionsArgs, NetworkConnectionsOutput,
    PingArgs as NetPingArgs, PingOutput as NetPingOutput,
    network_adapters_inner, network_connections_inner, ping_inner as net_ping_inner,
};
use crate::tools::performance::{PerformanceArgs, PerformanceOutput, performance_inner};
use crate::tools::ping::{PingArgs, PingOutput, ping_inner};
use crate::tools::process::{
    KillProcessArgs, KillProcessOutput, ListProcessesArgs, ListProcessesOutput,
    StartProcessArgs, StartProcessOutput, kill_process_inner, list_processes_inner,
    start_process_inner,
};
use crate::tools::registry::{
    ReadRegistryArgs, ReadRegistryOutput, WriteRegistryArgs, WriteRegistryOutput,
    read_registry_inner, write_registry_inner,
};
use crate::tools::service::{
    ListServicesArgs, ListServicesOutput, ServiceActionArgs, ServiceActionOutput,
    list_services_inner, restart_service_inner, start_service_inner, stop_service_inner,
};
use crate::tools::startup::{ListStartupArgs, ListStartupOutput, list_startup_inner};
use crate::tools::sysinfo_advanced::{HardwareInfoArgs, HardwareInfoOutput, hardware_info_inner};
use crate::tools::system::{SystemInfoArgs, SystemInfoOutput, system_info_inner};
use crate::tools::task::{
    ListTasksArgs, ListTasksOutput, RunTaskArgs, RunTaskOutput,
    list_tasks_inner, run_task_inner,
};
use crate::tools::user::{
    ListGroupsArgs, ListGroupsOutput, ListUsersArgs, ListUsersOutput,
    list_groups_inner, list_users_inner,
};
use crate::tools::windowsupdate::{
    CheckUpdatesArgs, CheckUpdatesOutput, InstallUpdatesArgs, InstallUpdatesOutput,
    check_updates_inner, install_updates_inner,
};

#[derive(Clone, Default)]
pub struct McpServerHandler {
    // Read by the `#[tool_handler]` macro's generated impl; the lint
    // pass can't see through that, hence the allow.
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl McpServerHandler {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    /// Smoke-test echo tool. Returns `{ "echo": message ?? "pong" }`.
    #[tool(
        name = "ping",
        description = "Smoke-test tool that echoes the optional `message` argument, or returns \"pong\" if none was provided."
    )]
    #[tracing::instrument(skip_all)]
    pub fn ping(&self, Parameters(args): Parameters<PingArgs>) -> Json<PingOutput> {
        Json(ping_inner(args))
    }

    /// Get Windows system information
    #[tool(
        name = "system_info",
        description = "Get Windows system information including hostname, OS version, memory, and CPU count."
    )]
    #[tracing::instrument(skip_all)]
    pub fn system_info(&self, Parameters(args): Parameters<SystemInfoArgs>) -> Json<SystemInfoOutput> {
        match system_info_inner(args) {
            Ok(output) => Json(output),
            Err(e) => {
                tracing::error!("system_info failed: {}", e);
                Json(SystemInfoOutput {
                    hostname: "error".to_string(),
                    os_version: "error".to_string(),
                    total_memory_mb: 0,
                    cpu_count: 0,
                })
            }
        }
    }

    #[tool(
        name = "list_processes",
        description = "List all running processes with PID, name, status, memory usage, and CPU usage."
    )]
    #[tracing::instrument(skip_all)]
    pub fn list_processes(&self, Parameters(args): Parameters<ListProcessesArgs>) -> Json<ListProcessesOutput> {
        Json(list_processes_inner(args))
    }

    #[tool(
        name = "kill_process",
        description = "Kill a process by PID or name. Use pid for specific process, name to kill all processes with that name."
    )]
    #[tracing::instrument(skip_all)]
    pub fn kill_process(&self, Parameters(args): Parameters<KillProcessArgs>) -> Json<KillProcessOutput> {
        Json(kill_process_inner(args))
    }

    #[tool(
        name = "start_process",
        description = "Start a new process with optional arguments."
    )]
    #[tracing::instrument(skip_all)]
    pub fn start_process(&self, Parameters(args): Parameters<StartProcessArgs>) -> Json<StartProcessOutput> {
        Json(start_process_inner(args))
    }

    #[tool(
        name = "list_services",
        description = "List all Windows services with name, display name, status, and startup type."
    )]
    #[tracing::instrument(skip_all)]
    pub fn list_services(&self, Parameters(args): Parameters<ListServicesArgs>) -> Json<ListServicesOutput> {
        Json(list_services_inner(args))
    }

    #[tool(
        name = "start_service",
        description = "Start a Windows service by name."
    )]
    #[tracing::instrument(skip_all)]
    pub fn start_service(&self, Parameters(args): Parameters<ServiceActionArgs>) -> Json<ServiceActionOutput> {
        Json(start_service_inner(args))
    }

    #[tool(
        name = "stop_service",
        description = "Stop a Windows service by name."
    )]
    #[tracing::instrument(skip_all)]
    pub fn stop_service(&self, Parameters(args): Parameters<ServiceActionArgs>) -> Json<ServiceActionOutput> {
        Json(stop_service_inner(args))
    }

    #[tool(
        name = "restart_service",
        description = "Restart a Windows service by name."
    )]
    #[tracing::instrument(skip_all)]
    pub fn restart_service(&self, Parameters(args): Parameters<ServiceActionArgs>) -> Json<ServiceActionOutput> {
        Json(restart_service_inner(args))
    }

    #[tool(
        name = "disk_info",
        description = "Get information about all disk drives including total size, used space, free space, and usage percentage."
    )]
    #[tracing::instrument(skip_all)]
    pub fn disk_info(&self, Parameters(args): Parameters<DiskInfoArgs>) -> Json<DiskInfoOutput> {
        Json(disk_info_inner(args))
    }

    #[tool(
        name = "cleanup_disk",
        description = "Clean up temporary files and recycle bin on a specific drive. Returns freed space in GB."
    )]
    #[tracing::instrument(skip_all)]
    pub fn cleanup_disk(&self, Parameters(args): Parameters<CleanupDiskArgs>) -> Json<CleanupDiskOutput> {
        Json(cleanup_disk_inner(args))
    }

    #[tool(
        name = "network_adapters",
        description = "List all network adapters with name, MAC address, status, speed, and IP addresses."
    )]
    #[tracing::instrument(skip_all)]
    pub fn network_adapters(&self, Parameters(args): Parameters<NetworkAdaptersArgs>) -> Json<NetworkAdaptersOutput> {
        Json(network_adapters_inner(args))
    }

    #[tool(
        name = "ping_host",
        description = "Ping a host and return average latency, packets sent/received."
    )]
    #[tracing::instrument(skip_all)]
    pub fn ping_host(&self, Parameters(args): Parameters<NetPingArgs>) -> Json<NetPingOutput> {
        Json(net_ping_inner(args))
    }

    #[tool(
        name = "network_connections",
        description = "List all active TCP connections with local/remote addresses, ports, state, and process name."
    )]
    #[tracing::instrument(skip_all)]
    pub fn network_connections(&self, Parameters(args): Parameters<NetworkConnectionsArgs>) -> Json<NetworkConnectionsOutput> {
        Json(network_connections_inner(args))
    }

    #[tool(
        name = "read_event_log",
        description = "Read Windows event log entries (System, Application, Security). Filter by log name, level, and max entries."
    )]
    #[tracing::instrument(skip_all)]
    pub fn read_event_log(&self, Parameters(args): Parameters<ReadEventLogArgs>) -> Json<ReadEventLogOutput> {
        Json(read_event_log_inner(args))
    }

    #[tool(
        name = "clear_event_log",
        description = "Clear a Windows event log. Requires admin privileges."
    )]
    #[tracing::instrument(skip_all)]
    pub fn clear_event_log(&self, Parameters(args): Parameters<ClearEventLogArgs>) -> Json<ClearEventLogOutput> {
        Json(clear_event_log_inner(args))
    }

    #[tool(
        name = "performance",
        description = "Get system performance metrics: CPU usage, memory usage, and top 10 processes by memory."
    )]
    #[tracing::instrument(skip_all)]
    pub fn performance(&self, Parameters(args): Parameters<PerformanceArgs>) -> Json<PerformanceOutput> {
        Json(performance_inner(args))
    }

    #[tool(
        name = "read_registry",
        description = "Read registry key values and subkeys. Path format: HKEY_LOCAL_MACHINE\\SOFTWARE\\..."
    )]
    #[tracing::instrument(skip_all)]
    pub fn read_registry(&self, Parameters(args): Parameters<ReadRegistryArgs>) -> Json<ReadRegistryOutput> {
        Json(read_registry_inner(args))
    }

    #[tool(
        name = "write_registry",
        description = "Write a value to a registry key. Requires admin for HKLM."
    )]
    #[tracing::instrument(skip_all)]
    pub fn write_registry(&self, Parameters(args): Parameters<WriteRegistryArgs>) -> Json<WriteRegistryOutput> {
        Json(write_registry_inner(args))
    }

    #[tool(
        name = "list_startup",
        description = "List all startup programs configured on the system."
    )]
    #[tracing::instrument(skip_all)]
    pub fn list_startup(&self, Parameters(args): Parameters<ListStartupArgs>) -> Json<ListStartupOutput> {
        Json(list_startup_inner(args))
    }

    #[tool(
        name = "list_users",
        description = "List all local user accounts with name, full name, enabled status, and last logon."
    )]
    #[tracing::instrument(skip_all)]
    pub fn list_users(&self, Parameters(args): Parameters<ListUsersArgs>) -> Json<ListUsersOutput> {
        Json(list_users_inner(args))
    }

    #[tool(
        name = "list_groups",
        description = "List all local groups with name and description."
    )]
    #[tracing::instrument(skip_all)]
    pub fn list_groups(&self, Parameters(args): Parameters<ListGroupsArgs>) -> Json<ListGroupsOutput> {
        Json(list_groups_inner(args))
    }

    #[tool(
        name = "list_tasks",
        description = "List non-Microsoft scheduled tasks with name, state, next run, and last run."
    )]
    #[tracing::instrument(skip_all)]
    pub fn list_tasks(&self, Parameters(args): Parameters<ListTasksArgs>) -> Json<ListTasksOutput> {
        Json(list_tasks_inner(args))
    }

    #[tool(
        name = "run_task",
        description = "Run a scheduled task immediately by name."
    )]
    #[tracing::instrument(skip_all)]
    pub fn run_task(&self, Parameters(args): Parameters<RunTaskArgs>) -> Json<RunTaskOutput> {
        Json(run_task_inner(args))
    }

    #[tool(
        name = "check_updates",
        description = "Check for available Windows updates with title, KB, severity, and size."
    )]
    #[tracing::instrument(skip_all)]
    pub fn check_updates(&self, Parameters(args): Parameters<CheckUpdatesArgs>) -> Json<CheckUpdatesOutput> {
        Json(check_updates_inner(args))
    }

    #[tool(
        name = "install_updates",
        description = "Download and install available Windows updates. Optionally filter by title."
    )]
    #[tracing::instrument(skip_all)]
    pub fn install_updates(&self, Parameters(args): Parameters<InstallUpdatesArgs>) -> Json<InstallUpdatesOutput> {
        Json(install_updates_inner(args))
    }

    #[tool(
        name = "hardware_info",
        description = "Get detailed hardware information: CPU, memory, BIOS, motherboard, GPU, and storage."
    )]
    #[tracing::instrument(skip_all)]
    pub fn hardware_info(&self, Parameters(args): Parameters<HardwareInfoArgs>) -> Json<HardwareInfoOutput> {
        Json(hardware_info_inner(args))
    }
}

#[tool_handler]
impl ServerHandler for McpServerHandler {
    fn get_info(&self) -> ServerInfo {
        // `ServerInfo` and `Implementation` are `#[non_exhaustive]`, so
        // we start from `Default::default()` and assign individual fields
        // — struct expressions (even with `..base`) aren't allowed for
        // these from outside `rmcp`.
        let mut server_info_struct = Implementation::default();
        server_info_struct.name = env!("CARGO_PKG_NAME").to_string();
        server_info_struct.version = env!("CARGO_PKG_VERSION").to_string();

        let mut info = ServerInfo::default();
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.server_info = server_info_struct;
        info.instructions = Some(
            "Windows Management MCP Server. Provides tools for process management, \
             system information, and more. Built with Rust for high performance."
                .into(),
        );
        info
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn advertises_ping_tool() {
        let handler = McpServerHandler::new();
        let tools = handler.tool_router.list_all();

        let ping = tools
            .iter()
            .find(|t| t.name == "ping")
            .expect("ping tool must be advertised");

        let desc = ping
            .description
            .as_ref()
            .expect("ping tool must have a description");
        assert!(!desc.is_empty(), "ping description must be non-empty");
    }

    #[test]
    fn ping_route_is_registered() {
        let handler = McpServerHandler::new();
        assert!(handler.tool_router.has_route("ping"));
    }
}
