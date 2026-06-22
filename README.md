# Windows MCP Server

A comprehensive Rust-based MCP (Model Context Protocol) server providing **27 tools** for complete Windows system management.

## Tools

| Category | Tools | Implementation |
|----------|-------|----------------|
| **Process** | `list_processes`, `kill_process`, `start_process` | sysinfo crate |
| **Service** | `list_services`, `start_service`, `stop_service`, `restart_service` | PowerShell |
| **Disk** | `disk_info`, `cleanup_disk` | PowerShell |
| **Network** | `network_adapters`, `ping_host`, `network_connections` | PowerShell |
| **Event Log** | `read_event_log`, `clear_event_log` | PowerShell |
| **Performance** | `performance` | sysinfo crate |
| **Registry** | `read_registry`, `write_registry` | PowerShell |
| **Startup** | `list_startup` | PowerShell |
| **User** | `list_users`, `list_groups` | PowerShell |
| **Task** | `list_tasks`, `run_task` | PowerShell |
| **Windows Update** | `check_updates`, `install_updates` | COM Interop |
| **Hardware Info** | `hardware_info` | WMI |
| **System** | `system_info`, `ping` | sysinfo crate |

## Quick Start

```bash
# Build
cargo build --release

# Run (stdio transport)
./target/release/windows-mcp-server.exe --transport stdio

# Run (HTTP transport)
./target/release/windows-mcp-server.exe --transport http --bind 127.0.0.1:3000
```

## Integration

### OpenCode

Add to `~/.config/opencode/opencode.json`:

```json
{
  "mcp": {
    "windows-mcp": {
      "type": "local",
      "command": ["C:\\path\\to\\windows-mcp-server.exe"],
      "enabled": true,
      "timeout": 60000
    }
  }
}
```

### Claude Desktop

Add to Claude Desktop config:

```json
{
  "mcpServers": {
    "windows-mcp": {
      "command": "C:\\path\\to\\windows-mcp-server.exe",
      "args": ["--transport", "stdio"]
    }
  }
}
```

## CLI Options

| Flag | Default | Description |
|------|---------|-------------|
| `--transport` | `stdio` | Transport type: `stdio` or `http` |
| `--bind` | `127.0.0.1:8080` | Bind address for HTTP |
| `--log-format` | `pretty` | Log format: `pretty` or `json` |

## Architecture

```
src/
├── main.rs                 # Entry point
├── config.rs               # CLI + Config
├── transport/              # stdio + HTTP
├── server/
│   └── handler.rs          # 27 tool handlers
└── tools/
    ├── process.rs          # Process management
    ├── service.rs          # Service management
    ├── disk.rs             # Disk operations
    ├── network.rs          # Network operations
    ├── eventlog.rs         # Event log
    ├── performance.rs      # Performance metrics
    ├── registry.rs         # Registry operations
    ├── startup.rs          # Startup programs
    ├── user.rs             # User/Group management
    ├── task.rs             # Scheduled tasks
    ├── system.rs           # System info
    ├── windowsupdate.rs    # Windows Update
    └── sysinfo_advanced.rs # Hardware details (WMI)
```

## Development

```bash
cargo test                 # Run 41 tests
cargo clippy --all-targets # Lint
cargo fmt --all            # Format
```

## License

MIT
