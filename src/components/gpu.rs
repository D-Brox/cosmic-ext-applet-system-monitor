use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::{error::NvmlError, Device, Nvml};
use regex::Regex;
use std::collections::HashMap;
use std::fs::{read_dir, read_to_string};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

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
    /// Optional GPU temperature in Celsius. Added for temperature monitoring.
    pub temperature: Option<f32>,
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
        let re_cards = Regex::new(r"card(\d+)/device$").unwrap();
        let gpus = read_dir("/sys/class/drm")
            .map(|dir_entries| {
                dir_entries
                    .filter_map(|dir_entry| {
                        // If at any point this fails, we just skip the entry

                        // Check if it's a card or a display output
                        let entry = dir_entry.ok()?;
                        let sysfs_path = entry.path().join("device");
                        let _ = re_cards.captures(sysfs_path.to_str().unwrap())?;

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
                            Gpu::new_nvidia(pci_slot)
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
            gpu.refresh_usage();
            gpu.refresh_vram();
            gpu.refresh_temperature();
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

/// Finds GPU temperature from HWMON sysfs for non-NVIDIA GPUs. New function.
/// Matches GPU device to an HWMON entry and reads `tempX_input` files.
/// NOT TESTED, but ***SHOULD** work.
#[allow(clippy::cast_precision_loss)]
fn get_hwmon_temperature(device_sysfs_path: &Path) -> Option<f32> {
    // Resolve the actual device path, as device_sysfs_path is often a symlink (e.g. /sys/class/drm/card0/device)
    let target_device_path = std::fs::canonicalize(device_sysfs_path).ok()?;

    for entry in read_dir("/sys/class/hwmon").ok()?.filter_map(Result::ok) {
        let hwmon_path = entry.path();
        if !hwmon_path.is_dir()
            || !hwmon_path
                .file_name()?
                .to_string_lossy()
                .starts_with("hwmon")
        {
            continue;
        }

        let hwmon_device_symlink = hwmon_path.join("device");
        // The device symlink inside an hwmon dir is relative; resolve its absolute path for comparison.
        if let Ok(linked_path) = std::fs::read_link(&hwmon_device_symlink) {
            if let Ok(resolved_hwmon_device_path) = hwmon_path.join(linked_path).canonicalize() {
                if resolved_hwmon_device_path == target_device_path {
                    // Found the hwmon directory for this GPU.
                    for i in 1..=5 {
                        // Check common tempN_input files
                        let temp_input_path = hwmon_path.join(format!("temp{i}_input"));
                        if let Ok(temp_str) = read_to_string(temp_input_path) {
                            if let Ok(temp_milli_c) = temp_str.trim().parse::<i32>() {
                                return Some(temp_milli_c as f32 / 1000.0); // Convert millidegrees
                            }
                        }
                    }
                    return None; // No matching temp input found for this hwmon
                }
            }
        }
    }
    None
}

impl Gpu {
    fn new(sysfs_path: PathBuf) -> Option<Self> {
        let initial_usage = read_syspath(&sysfs_path, "gpu_busy_percent");
        let initial_used_vram = read_syspath(&sysfs_path, "mem_info_vram_used");
        let initial_total_vram = read_syspath(&sysfs_path, "mem_info_vram_total");
        // Fetch initial temperature for non-NVIDIA GPUs.
        let initial_temperature = get_hwmon_temperature(&sysfs_path);

        Some(Self {
            data: GpuData {
                usage: initial_usage?,
                used_vram: initial_used_vram?,
                total_vram: initial_total_vram?,
                temperature: initial_temperature, // Store initial temperature.
            },
            vendor: GpuType::PlugAndPlay { sysfs_path },
        })
    }

    #[allow(clippy::cast_precision_loss)]
    fn new_nvidia(pci_slot: String) -> Option<Self> {
        let device = NVML.as_ref().ok()?.device_by_pci_bus_id(pci_slot).ok()?;
        let utilization = device.utilization_rates().ok()?;
        let meminfo = device.memory_info().ok()?;
        // Fetch initial temperature for NVIDIA GPUs via NVML.
        let temp = device.temperature(TemperatureSensor::Gpu).ok();

        Some(Self {
            vendor: GpuType::PrayAndHope { device },
            data: GpuData {
                usage: u64::from(utilization.gpu),
                used_vram: meminfo.total - meminfo.free,
                total_vram: meminfo.total,
                temperature: temp.map(|t| t as f32), // Store initial temperature.
            },
        })
    }

    fn refresh_usage(&mut self) {
        match &self.vendor {
            GpuType::PrayAndHope { device } => {
                _ = device
                    .utilization_rates()
                    .map(|utilization| self.data.usage = u64::from(utilization.gpu));
            }

            GpuType::PlugAndPlay { sysfs_path } => {
                _ = read_syspath(sysfs_path, "gpu_busy_percent")
                    .map(|usage| self.data.usage = usage);
            }
        }
    }

    fn refresh_vram(&mut self) {
        match &self.vendor {
            GpuType::PrayAndHope { device } => {
                _ = device
                    .memory_info()
                    .map(|meminfo| self.data.used_vram = meminfo.total - meminfo.free);
            }

            GpuType::PlugAndPlay { sysfs_path } => {
                _ = read_syspath(sysfs_path, "mem_info_vram_used")
                    .map(|used_vram| self.data.used_vram = used_vram);
            }
        }
    }

    /// Refreshes GPU temperature (NVIDIA via NVML, others via HWMON). Added for temp monitoring.
    #[allow(clippy::cast_precision_loss)]
    fn refresh_temperature(&mut self) {
        match &self.vendor {
            GpuType::PrayAndHope { device } => {
                if let Ok(temp) = device.temperature(TemperatureSensor::Gpu) {
                    self.data.temperature = Some(temp as f32);
                } else {
                    self.data.temperature = None;
                }
            }
            GpuType::PlugAndPlay { sysfs_path } => {
                self.data.temperature = get_hwmon_temperature(sysfs_path);
            }
        }
    }
}
