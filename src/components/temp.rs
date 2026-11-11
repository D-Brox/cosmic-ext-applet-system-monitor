// SPDX-License-Identifier: GPL-3.0-only

use std::fs::{read_dir, read_to_string};
use std::path::{Path, PathBuf};

const HWMON: &str = "/sys/class/hwmon";

const CPU_DRIVERS: &[&str] = &["k10temp", "zenpower", "coretemp", "cpu_thermal"];
const CPU_LABELS: &[&str] = &["Tdie", "Tctl", "Package id 0"];

pub fn read_celsius(path: &Path) -> Option<f32> {
    let millidegrees: i64 = read_to_string(path).ok()?.trim().parse().ok()?;
    #[allow(clippy::cast_precision_loss)]
    Some(millidegrees as f32 / 1000.0)
}

// hwmon numbers change between boots, so match on the driver name
pub fn cpu_sensor() -> Option<PathBuf> {
    let hwmons: Vec<(String, PathBuf)> = read_dir(HWMON)
        .ok()?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let name = read_to_string(path.join("name")).ok()?.trim().to_string();
            Some((name, path))
        })
        .collect();
    CPU_DRIVERS.iter().find_map(|driver| {
        let (_, path) = hwmons.iter().find(|(name, _)| name == driver)?;
        labeled_input(path).or_else(|| Some(path.join("temp1_input")).filter(|p| p.exists()))
    })
}

fn labeled_input(hwmon: &Path) -> Option<PathBuf> {
    let labels: Vec<(String, PathBuf)> = read_dir(hwmon)
        .ok()?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let file = path.file_name()?.to_str()?;
            let input = hwmon.join(file.strip_suffix("_label")?.to_string() + "_input");
            Some((read_to_string(&path).ok()?.trim().to_string(), input))
        })
        .collect();
    CPU_LABELS.iter().find_map(|wanted| {
        labels
            .iter()
            .find(|(label, _)| label == wanted)
            .map(|(_, input)| input.clone())
    })
}

// NVMe drives have `device/hwmon3`, most other devices `device/hwmon/hwmon3`
pub fn device_temp(device: &Path) -> Option<f32> {
    let find = |dir: &Path| {
        read_dir(dir).ok()?.find_map(|entry| {
            let entry = entry.ok()?;
            entry
                .file_name()
                .to_str()?
                .starts_with("hwmon")
                .then(|| read_celsius(&entry.path().join("temp1_input")))
                .flatten()
        })
    };
    find(&device.join("hwmon")).or_else(|| find(device))
}

pub fn format_celsius(celsius: f32) -> String {
    format!("{celsius:.0} °C")
}
