use actix_web::rt::task;
use async_trait::async_trait;
use log::error;
use std::thread;
use std::time::Duration;
use sysinfo::{Components, Disks, System};

use crate::core::contracts::adapters::system_metrics::{
    CpuInfo, DiskInfo, MachineInformation, MemoryInfo, NetworkInfo, SystemMetricsPort,
};

#[derive(Clone, Default)]
pub struct SysinfoSystemMetrics;

impl SysinfoSystemMetrics {
    pub fn new() -> Self {
        Self
    }

    fn bytes_to_gb(bytes: u64) -> f64 {
        (bytes as f64) / (1024.0 * 1024.0 * 1024.0)
    }

    fn round_two(value: f64) -> f64 {
        (value * 100.0).round() / 100.0
    }

    fn cpu_info(sys: &System) -> CpuInfo {
        let percent = sys
            .cpus()
            .first()
            .map(|cpu| cpu.cpu_usage() as f64)
            .unwrap_or(0.0);
        let name = sys
            .cpus()
            .first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        CpuInfo {
            name,
            architecture: std::env::consts::ARCH.to_string(),
            physical_cores: num_cpus::get_physical(),
            logical_cores: num_cpus::get(),
            percent: Self::round_two(percent),
            temperature: Self::cpu_temperature(),
        }
    }

    fn cpu_temperature() -> String {
        let components = Components::new_with_refreshed_list();
        for component in components.iter() {
            let label = component.label();
            if (label.to_lowercase().contains("cpu") || label.contains("Tctl"))
                && let Some(temp) = component.temperature()
            {
                return format!("{:.2}", temp);
            }
        }
        "Not available".to_string()
    }

    fn memory_info(sys: &System) -> MemoryInfo {
        let total = sys.total_memory();
        let available = sys.available_memory();
        let used = total.saturating_sub(available);

        let (percent, free_percent) = if total > 0 {
            (
                (used as f64 / total as f64) * 100.0,
                (available as f64 / total as f64) * 100.0,
            )
        } else {
            (0.0, 0.0)
        };

        MemoryInfo {
            total_gb: Self::round_two(Self::bytes_to_gb(total)),
            available_gb: Self::round_two(Self::bytes_to_gb(available)),
            used_gb: Self::round_two(Self::bytes_to_gb(used)),
            percent: Self::round_two(percent),
            free_percent: Self::round_two(free_percent),
        }
    }

    fn disk_info() -> DiskInfo {
        let disks = Disks::new_with_refreshed_list();
        let Some(disk) = disks.first() else {
            return DiskInfo {
                total_gb: 0.0,
                used_gb: 0.0,
                free_gb: 0.0,
                percent: 0.0,
                free_percent: 0.0,
            };
        };

        let total = disk.total_space();
        let free = disk.available_space();
        let used = total.saturating_sub(free);

        let (percent, free_percent) = if total > 0 {
            (
                (used as f64 / total as f64) * 100.0,
                (free as f64 / total as f64) * 100.0,
            )
        } else {
            (0.0, 0.0)
        };

        DiskInfo {
            total_gb: Self::round_two(Self::bytes_to_gb(total)),
            used_gb: Self::round_two(Self::bytes_to_gb(used)),
            free_gb: Self::round_two(Self::bytes_to_gb(free)),
            percent: Self::round_two(percent),
            free_percent: Self::round_two(free_percent),
        }
    }

    async fn external_ip() -> Result<String, reqwest::Error> {
        let response = reqwest::get("https://api.ipify.org?format=json").await?;
        let json: serde_json::Value = response.json().await?;
        Ok(json["ip"]
            .as_str()
            .unwrap_or("Unable to obtain IP")
            .to_string())
    }

    fn uptime_ms() -> u64 {
        System::uptime().saturating_mul(1000)
    }
}

#[async_trait]
impl SystemMetricsPort for SysinfoSystemMetrics {
    async fn collect(&self) -> MachineInformation {
        let sys = task::spawn_blocking(|| {
            let mut sys = System::new_all();
            sys.refresh_all();
            thread::sleep(Duration::from_millis(500));
            sys.refresh_cpu_usage();
            sys
        })
        .await
        .unwrap_or_else(|_| System::new_all());

        let ip = match Self::external_ip().await {
            Ok(value) => value,
            Err(err) => {
                error!("Error getting external IP: {:?}", err);
                "Unable to obtain IP".to_string()
            }
        };

        MachineInformation {
            cpu: Self::cpu_info(&sys),
            memory: Self::memory_info(&sys),
            disk: Self::disk_info(),
            network: NetworkInfo { ip },
            uptime: Self::uptime_ms(),
        }
    }
}
