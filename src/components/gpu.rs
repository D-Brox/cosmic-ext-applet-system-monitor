use nvml_wrapper::Nvml;
use std::collections::HashMap;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

const NV_VENDOR_ID: u16 = 0x10DE;

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
    // Nvidia. We keep the sysfs `device/` path (for the RTD3 power gate) and the PCI slot
    // (to re-acquire the NVML device lazily), but never hold an NVML handle across ticks so
    // the dGPU is free to runtime-suspend (RTD3).
    PrayAndHope { sysfs_path: PathBuf, pci_slot: String },
    PlugAndPlay { sysfs_path: PathBuf }, // Anything else
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

                        // Find vendor, since for Nvidia we need to use nvml.
                        // For this, we test the vendor file, with the PCI_ID in uevent as backup.
                        // Nvidia is a pain, so driver is probably needed as backup too.
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

/// `false` only when the dGPU is asleep (or going to sleep), so we never resume a
/// suspended Nvidia dGPU just to read stats. `power/runtime_status` is one of
/// `active` / `suspended` / `suspending` / `resuming` / `unsupported`.
fn is_runtime_active(status: &str) -> bool {
    !matches!(status.trim(), "suspended" | "suspending")
}

/// Read the dGPU's runtime power state from sysfs. This is a plain file read and never
/// touches NVML, so it cannot wake the device. A missing/unreadable file (e.g. a desktop
/// without runtime PM) is treated as active so live stats keep working there.
fn nvidia_runtime_active(sysfs_path: &Path) -> bool {
    match std::fs::read_to_string(sysfs_path.join("power/runtime_status")) {
        Ok(status) => is_runtime_active(&status),
        Err(_) => true,
    }
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
        // Don't touch NVML here: opening it at startup would pin/wake the dGPU. Start at
        // zero; the first refresh while the dGPU is active fills in real values.
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
                // RTD3 gate: if the dGPU is suspended, leave it alone (a suspended GPU is
                // idle, so report 0% usage and keep the last known VRAM).
                if !nvidia_runtime_active(sysfs_path) {
                    self.data.usage = 0;
                    return;
                }

                // Acquire an NVML handle, read, then drop it before returning. Holding it
                // across ticks would keep `/dev/nvidia*` open and pin runtime_usage > 0,
                // preventing the dGPU from ever suspending.
                if let Ok(nvml) = Nvml::init() {
                    if let Ok(device) = nvml.device_by_pci_bus_id(pci_slot.clone()) {
                        if let Ok(utilization) = device.utilization_rates() {
                            self.data.usage = u64::from(utilization.gpu);
                        }
                        if let Ok(meminfo) = device.memory_info() {
                            self.data.used_vram = meminfo.total - meminfo.free;
                            self.data.total_vram = meminfo.total;
                        }
                    }
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
    use super::is_runtime_active;

    #[test]
    fn active_states_are_queried() {
        assert!(is_runtime_active("active"));
        assert!(is_runtime_active("active\n"));
        assert!(is_runtime_active("unsupported"));
        assert!(is_runtime_active("resuming"));
    }

    #[test]
    fn suspended_states_are_skipped() {
        assert!(!is_runtime_active("suspended"));
        assert!(!is_runtime_active("suspended\n"));
        assert!(!is_runtime_active("suspending"));
    }
}
