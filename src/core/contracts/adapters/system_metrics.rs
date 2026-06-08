use async_trait::async_trait;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CpuInfo {
    pub name: String,
    pub architecture: String,
    pub physical_cores: usize,
    pub logical_cores: usize,
    pub percent: f64,
    pub temperature: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryInfo {
    pub total_gb: f64,
    pub available_gb: f64,
    pub used_gb: f64,
    pub percent: f64,
    pub free_percent: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskInfo {
    pub total_gb: f64,
    pub used_gb: f64,
    pub free_gb: f64,
    pub percent: f64,
    pub free_percent: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkInfo {
    pub ip: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MachineInformation {
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub disk: DiskInfo,
    pub network: NetworkInfo,
    pub uptime: u64,
}

#[async_trait]
pub trait SystemMetricsPort: Send + Sync {
    async fn collect(&self) -> MachineInformation;
}
