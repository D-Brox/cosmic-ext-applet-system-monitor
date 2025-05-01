use lazy_regex::{lazy_regex, Lazy, Regex};
use nvml_wrapper::{error::NvmlError, Device, Nvml};
use std::collections::HashMap;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

static RE_CARDS: Lazy<Regex> = lazy_regex!(r"card(\d+)/device$");
const NV_VENDOR_ID: u16 = 0x10DE;
static NVML: LazyLock<Result<Nvml, NvmlError>> = LazyLock::new(Nvml::init);

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
    PrayAndHope { device: Device<'static> }, // Nvidia
    PlugAndPlay { sysfs_path: PathBuf },     // Anything else
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
                        let _ = RE_CARDS.captures(sysfs_path.to_str().unwrap())?;

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
                                u16::from_str_radix(&content.trim_start_matches("0x"), 16).ok()
                            })
                            .or(uevent.get("PCI_ID").and_then(|id| {
                                id.split_once(":")
                                    .and_then(|p| u16::from_str_radix(p.0, 16).ok())
                            }));
                        let driver = uevent.get("DRIVER").map(|s| s.as_str());

                        if vendor == Some(NV_VENDOR_ID) || driver == Some("nvidia") {
                            let pci_slot = uevent.get("PCI_SLOT_NAME").cloned()?;
                            Gpu::new_nvidia(pci_slot)
                        } else {
                            Gpu::new(sysfs_path)
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or(vec![]);
        Self { inner: gpus }
    }

    pub fn refresh(&mut self) {
        for gpu in &mut self.inner {
            gpu.refresh_usage();
            gpu.refresh_vram();
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

    fn new_nvidia(pci_slot: String) -> Option<Self> {
        let device = NVML.as_ref().ok()?.device_by_pci_bus_id(pci_slot).ok()?;
        let utilization = device.utilization_rates().ok()?;
        let meminfo = device.memory_info().ok()?;

        Some(Self {
            vendor: GpuType::PrayAndHope { device },
            data: GpuData {
                usage: utilization.gpu as u64,
                used_vram: meminfo.total - meminfo.free,
                total_vram: meminfo.total,
            },
        })
    }

    fn refresh_usage(&mut self) {
        match &self.vendor {
            GpuType::PrayAndHope { device } => {
                _ = device
                    .utilization_rates()
                    .map(|utilization| self.data.usage = utilization.gpu as u64)
            }

            GpuType::PlugAndPlay { sysfs_path } => {
                _ = read_syspath(sysfs_path, "gpu_busy_percent")
                    .map(|usage| self.data.usage = usage)
            }
        }
    }

    fn refresh_vram(&mut self) {
        match &self.vendor {
            GpuType::PrayAndHope { device } => {
                _ = device
                    .memory_info()
                    .map(|meminfo| self.data.used_vram = meminfo.total - meminfo.free)
            }

            GpuType::PlugAndPlay { sysfs_path } => {
                _ = read_syspath(sysfs_path, "mem_info_vram_used")
                    .map(|used_vram| self.data.used_vram = used_vram)
            }
        }
    }
}
