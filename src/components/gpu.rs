use nvml_wrapper::Nvml;
use std::collections::HashMap;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;

const NV_VENDOR_ID: u16 = 0x10DE;
static SELF_EXE: LazyLock<Option<PathBuf>> = LazyLock::new(|| std::env::current_exe().ok());

pub struct Gpus {
    inner: Vec<Gpu>,
}

#[derive(Clone, Copy)]
pub struct GpuData {
    pub usage: u64,
    pub used_vram: u64,
    pub total_vram: u64,
}

struct Gpu {
    vendor: GpuType,
    data: GpuData,
}

enum GpuType {
    PrayAndHope {
        sysfs_path: PathBuf,
        pci_slot: String,
    }, // Nvidia
    PlugAndPlay {
        sysfs_path: PathBuf,
    }, // Anything else
}

impl Gpus {
    pub fn new() -> Self {
        let gpus = read_dir("/sys/class/drm")
            .map(|dir_entries| {
                dir_entries
                    .filter_map(|dir_entry| {
                        // If at any point this fails, we just skip the entry

                        // Check if it's a card or a display output
                        let entry = dir_entry.ok()?;
                        let sysfs_path = entry.path().join("device");
                        match_card_device(sysfs_path.to_str()?)?;

                        // Next get the uevent info of the card if it exists
                        let device_uevent_path = sysfs_path.join("uevent");
                        let uevent = std::fs::read_to_string(device_uevent_path)
                            .map(|content| {
                                content
                                    .lines()
                                    .map(|line| {
                                        line.split_once('=')
                                            .map(|(a, b)| (a.to_string(), b.to_string()))
                                            .expect("Malformed uevent line")
                                    })
                                    .collect::<HashMap<_, _>>()
                            })
                            .ok()?;

                        // Find vendor and driver, since for Nvidia we may need to use nvml.
                        // For this, we test the vendor file, with the PCI_ID in uevent as backup.
                        let device_vendor_path = sysfs_path.join("vendor");
                        let vendor = std::fs::read_to_string(device_vendor_path)
                            .ok()
                            .and_then(|content| {
                                u16::from_str_radix(content.trim_start_matches("0x"), 16).ok()
                            })
                            .or(uevent.get("PCI_ID").and_then(|id| {
                                id.split_once(':')
                                    .and_then(|p| u16::from_str_radix(p.0, 16).ok())
                            }));
                        let driver = uevent.get("DRIVER").map(String::as_str);

                        if vendor == Some(NV_VENDOR_ID) || driver == Some("nvidia") {
                            let pci_slot = uevent.get("PCI_SLOT_NAME").cloned()?;
                            Gpu::new_nvidia(sysfs_path, pci_slot)
                        } else {
                            Gpu::new(sysfs_path)
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Self { inner: gpus }
    }

    pub fn refresh(&mut self) {
        for gpu in &mut self.inner {
            gpu.refresh();
        }
    }

    pub fn num_gpus(&self) -> usize {
        self.inner.len()
    }
    pub fn data(&self) -> Vec<GpuData> {
        self.inner.iter().map(|gpu| gpu.data).collect()
    }
}

fn read_syspath(sysfs_path: &Path, file: &str) -> Option<u64> {
    std::fs::read_to_string(sysfs_path.join(file))
        .ok()
        .and_then(|s| s.trim_end().parse().ok())
}

fn match_card_device(s: &str) -> Option<()> {
    let before_device = s.strip_suffix("/device")?;
    let start_card = before_device.rfind("card")?;
    let digits = &before_device[start_card + 4..]; // slice after "card"

    if !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) {
        Some(())
    } else {
        None
    }
}

// Read the dGPU's runtime power state from sysfs to avoid waking the device.
fn nvidia_runtime_active(sysfs_path: &Path) -> bool {
    match std::fs::read_to_string(sysfs_path.join("power/runtime_status")) {
        Ok(status) => !matches!(status.trim(), "suspended" | "suspending"),
        Err(_) => true,
    }
}

fn read_nvidia_stats(pci_slot: &str) -> Option<GpuData> {
    let output = Command::new(SELF_EXE.as_deref()?)
        .args(["--nvidia-stats", pci_slot])
        .output()
        .ok()?;

    output.status.success().then_some(
        std::str::from_utf8(&output.stdout)
            .ok()
            .and_then(parse_nvidia_stats)?,
    )
}

pub fn query_nvidia_stats(pci_slot: &str) -> Option<GpuData> {
    let nvml = Nvml::init().ok()?;
    let device = nvml.device_by_pci_bus_id(pci_slot).ok()?;
    let utilization = device.utilization_rates().ok()?;
    let memory = device.memory_info().ok()?;

    Some(GpuData {
        usage: u64::from(utilization.gpu),
        used_vram: memory.total - memory.free,
        total_vram: memory.total,
    })
}

pub fn nvidia_stats_helper(pci_slot: Option<String>) {
    if let Some(pci_slot) = pci_slot {
        if let Some(data) = query_nvidia_stats(&pci_slot) {
            println!("{},{},{}", data.usage, data.used_vram, data.total_vram);
        } else {
            eprintln!("failed to read NVIDIA stats for {pci_slot}");
            std::process::exit(1);
        }
    } else {
        eprintln!("--nvidia-stats requires a PCI bus id");
        std::process::exit(2);
    }
}

fn parse_nvidia_stats(output: &str) -> Option<GpuData> {
    let mut values = output.lines().next()?.split(',').map(str::trim);
    let usage = values.next()?.parse().ok()?;
    let used_vram = values.next()?.parse().ok()?;
    let total_vram = values.next()?.parse().ok()?;
    values.next().is_none().then_some(GpuData {
        usage,
        used_vram,
        total_vram,
    })
}

impl Gpu {
    fn new(sysfs_path: PathBuf) -> Option<Self> {
        Some(Self {
            data: GpuData {
                usage: read_syspath(&sysfs_path, "gpu_busy_percent")?,
                used_vram: read_syspath(&sysfs_path, "mem_info_vram_used")?,
                total_vram: read_syspath(&sysfs_path, "mem_info_vram_total")?,
            },
            vendor: GpuType::PlugAndPlay { sysfs_path },
        })
    }

    fn new_nvidia(sysfs_path: PathBuf, pci_slot: String) -> Option<Self> {
        Some(Self {
            vendor: GpuType::PrayAndHope {
                sysfs_path,
                pci_slot,
            },
            data: GpuData {
                usage: 0,
                used_vram: 0,
                total_vram: 0,
            },
        })
    }

    fn refresh(&mut self) {
        match &self.vendor {
            GpuType::PrayAndHope {
                sysfs_path,
                pci_slot,
            } => {
                if !nvidia_runtime_active(sysfs_path) {
                    self.data.usage = 0;
                    return;
                }

                // Run NVML in a short-lived helper, to prevent fd leakage and RTD3 issues.
                // Holding a NVLM handle in the main process across ticks would keep `/dev/nvidia*` open
                // and pin runtime_usage > 0, preventing the dGPU from ever suspending.
                if let Some(data) = read_nvidia_stats(pci_slot) {
                    self.data = data;
                } else {
                    self.data.usage = 0;
                    self.data.used_vram = 0;
                }
            }

            GpuType::PlugAndPlay { sysfs_path } => {
                _ = read_syspath(sysfs_path, "gpu_busy_percent")
                    .map(|usage| self.data.usage = usage);
                _ = read_syspath(sysfs_path, "mem_info_vram_used")
                    .map(|used_vram| self.data.used_vram = used_vram);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nvidia_helper_statistics() {
        let stats = parse_nvidia_stats("19, 709, 12227\n").expect("valid helper output");

        assert_eq!(stats.usage, 19);
        assert_eq!(stats.used_vram, 709);
        assert_eq!(stats.total_vram, 12_227);
    }

    #[test]
    fn rejects_incomplete_or_non_numeric_nvidia_helper_statistics() {
        assert!(parse_nvidia_stats("19, 709\n").is_none());
        assert!(parse_nvidia_stats("N/A, 709, 12227\n").is_none());
        assert!(parse_nvidia_stats("19, 709, 12227, 1\n").is_none());
    }
}
