use rmcp::handler::server::wrapper::{Json, Parameters};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct HardwareInfoArgs {}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CpuDetail {
    pub name: String,
    pub cores: i32,
    pub logical_processors: i32,
    pub max_clock_mhz: i32,
    pub current_clock_mhz: i32,
    pub l2_cache_kb: i32,
    pub l3_cache_kb: i32,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct MemoryDetail {
    pub total_gb: f64,
    pub slots_used: i32,
    pub form_factor: String,
    pub speed_mhz: i32,
    pub manufacturer: String,
    pub part_number: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct BiosInfo {
    pub manufacturer: String,
    pub version: String,
    pub release_date: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct MotherboardInfo {
    pub manufacturer: String,
    pub product: String,
    pub serial_number: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct GpuInfo {
    pub name: String,
    pub adapter_ram_gb: f64,
    pub driver_version: String,
    pub video_processor: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct StorageDetail {
    pub model: String,
    pub size_gb: f64,
    pub media_type: String,
    pub interface_type: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct HardwareInfoOutput {
    pub cpu: Vec<CpuDetail>,
    pub memory: Vec<MemoryDetail>,
    pub total_memory_gb: f64,
    pub bios: Option<BiosInfo>,
    pub motherboard: Option<MotherboardInfo>,
    pub gpus: Vec<GpuInfo>,
    pub storage: Vec<StorageDetail>,
}

pub fn hardware_info_inner(_args: HardwareInfoArgs) -> HardwareInfoOutput {
    let cpu_script = r#"
        Get-CimInstance Win32_Processor | Select-Object Name, NumberOfCores, NumberOfLogicalProcessors, MaxClockSpeed, CurrentClockSpeed, L2CacheSize, L3CacheSize | ConvertTo-Json
    "#;
    let cpu_output = run_ps(cpu_script);
    let cpu: Vec<CpuDetail> = parse_json_array(&cpu_output)
        .iter()
        .filter_map(|v| {
            Some(CpuDetail {
                name: v["Name"].as_str().unwrap_or("Unknown").to_string(),
                cores: v["NumberOfCores"].as_i64().unwrap_or(0) as i32,
                logical_processors: v["NumberOfLogicalProcessors"].as_i64().unwrap_or(0) as i32,
                max_clock_mhz: v["MaxClockSpeed"].as_i64().unwrap_or(0) as i32,
                current_clock_mhz: v["CurrentClockSpeed"].as_i64().unwrap_or(0) as i32,
                l2_cache_kb: v["L2CacheSize"].as_i64().unwrap_or(0) as i32,
                l3_cache_kb: v["L3CacheSize"].as_i64().unwrap_or(0) as i32,
            })
        })
        .collect();

    let mem_script = r#"
        Get-CimInstance Win32_PhysicalMemory | Select-Object Capacity, FormFactor, Speed, Manufacturer, PartNumber, DeviceLocator | ConvertTo-Json
    "#;
    let mem_output = run_ps(mem_script);
    let memory: Vec<MemoryDetail> = parse_json_array(&mem_output)
        .iter()
        .filter_map(|v| {
            let capacity = v["Capacity"].as_u64().unwrap_or(0);
            let size_gb = (capacity as f64 / 1024.0 / 1024.0 / 1024.0 * 100.0).round() / 100.0;
            let form_factor = match v["FormFactor"].as_u64().unwrap_or(0) {
                8 => "DIMM",
                12 => "SODIMM",
                _ => "Unknown",
            };
            Some(MemoryDetail {
                total_gb: size_gb,
                slots_used: 1,
                form_factor: form_factor.to_string(),
                speed_mhz: v["Speed"].as_i64().unwrap_or(0) as i32,
                manufacturer: v["Manufacturer"].as_str().unwrap_or("Unknown").to_string(),
                part_number: v["PartNumber"].as_str().unwrap_or("Unknown").to_string(),
            })
        })
        .collect();

    let total_memory_gb = memory.iter().map(|m| m.total_gb).sum::<f64>() * 100.0 / 100.0;

    let bios_script = r#"
        Get-CimInstance Win32_BIOS | Select-Object Manufacturer, SMBIOSBIOSVersion, ReleaseDate | ConvertTo-Json
    "#;
    let bios_output = run_ps(bios_script);
    let bios = parse_json_object(&bios_output).and_then(|v| {
        Some(BiosInfo {
            manufacturer: v["Manufacturer"].as_str().unwrap_or("Unknown").to_string(),
            version: v["SMBIOSBIOSVersion"].as_str().unwrap_or("Unknown").to_string(),
            release_date: v["ReleaseDate"].as_str().unwrap_or("Unknown").to_string(),
        })
    });

    let mobo_script = r#"
        Get-CimInstance Win32_BaseBoard | Select-Object Manufacturer, Product, SerialNumber | ConvertTo-Json
    "#;
    let mobo_output = run_ps(mobo_script);
    let motherboard = parse_json_object(&mobo_output).and_then(|v| {
        Some(MotherboardInfo {
            manufacturer: v["Manufacturer"].as_str().unwrap_or("Unknown").to_string(),
            product: v["Product"].as_str().unwrap_or("Unknown").to_string(),
            serial_number: v["SerialNumber"].as_str().unwrap_or("Unknown").to_string(),
        })
    });

    let gpu_script = r#"
        Get-CimInstance Win32_VideoController | Select-Object Name, AdapterRAM, DriverVersion, VideoProcessor | ConvertTo-Json
    "#;
    let gpu_output = run_ps(gpu_script);
    let gpus: Vec<GpuInfo> = parse_json_array(&gpu_output)
        .iter()
        .filter_map(|v| {
            let adapter_ram = v["AdapterRAM"].as_u64().unwrap_or(0);
            let ram_gb = (adapter_ram as f64 / 1024.0 / 1024.0 / 1024.0 * 100.0).round() / 100.0;
            Some(GpuInfo {
                name: v["Name"].as_str().unwrap_or("Unknown").to_string(),
                adapter_ram_gb: ram_gb,
                driver_version: v["DriverVersion"].as_str().unwrap_or("Unknown").to_string(),
                video_processor: v["VideoProcessor"].as_str().unwrap_or("Unknown").to_string(),
            })
        })
        .collect();

    let storage_script = r#"
        Get-PhysicalDisk | Select-Object FriendlyName, Size, MediaType, BusType | ConvertTo-Json
    "#;
    let storage_output = run_ps(storage_script);
    let storage: Vec<StorageDetail> = parse_json_array(&storage_output)
        .iter()
        .filter_map(|v| {
            let size = v["Size"].as_u64().unwrap_or(0);
            let size_gb = (size as f64 / 1024.0 / 1024.0 / 1024.0 * 100.0).round() / 100.0;
            Some(StorageDetail {
                model: v["FriendlyName"].as_str().unwrap_or("Unknown").to_string(),
                size_gb,
                media_type: v["MediaType"].as_str().unwrap_or("Unknown").to_string(),
                interface_type: v["BusType"].as_str().unwrap_or("Unknown").to_string(),
            })
        })
        .collect();

    HardwareInfoOutput {
        cpu,
        memory,
        total_memory_gb,
        bios,
        motherboard,
        gpus,
        storage,
    }
}

fn run_ps(script: &str) -> String {
    let output = Command::new("powershell")
        .args(["-Command", script])
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute powershell"));
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn parse_json_array(json: &str) -> Vec<serde_json::Value> {
    if json.trim().is_empty() {
        return Vec::new();
    }
    match serde_json::from_str::<serde_json::Value>(json) {
        Ok(serde_json::Value::Array(arr)) => arr,
        Ok(serde_json::Value::Object(obj)) => vec![serde_json::Value::Object(obj)],
        _ => Vec::new(),
    }
}

fn parse_json_object(json: &str) -> Option<serde_json::Map<String, serde_json::Value>> {
    if json.trim().is_empty() {
        return None;
    }
    match serde_json::from_str::<serde_json::Value>(json) {
        Ok(serde_json::Value::Object(obj)) => Some(obj),
        _ => None,
    }
}

#[tracing::instrument(skip_all)]
pub fn hardware_info(Parameters(args): Parameters<HardwareInfoArgs>) -> Json<HardwareInfoOutput> {
    Json(hardware_info_inner(args))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn hardware_info_returns_cpu() {
        let output = hardware_info_inner(HardwareInfoArgs {});
        assert!(!output.cpu.is_empty(), "should have at least one CPU");
    }

    #[test]
    fn hardware_info_returns_memory() {
        let output = hardware_info_inner(HardwareInfoArgs {});
        assert!(output.total_memory_gb > 0.0, "should have positive memory");
    }

    #[test]
    fn hardware_info_returns_bios() {
        let output = hardware_info_inner(HardwareInfoArgs {});
        assert!(output.bios.is_some(), "should have BIOS info");
    }

    #[test]
    fn hardware_info_returns_gpu() {
        let output = hardware_info_inner(HardwareInfoArgs {});
        assert!(!output.gpus.is_empty(), "should have at least one GPU");
    }
}
