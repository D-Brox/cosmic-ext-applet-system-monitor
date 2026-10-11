// SPDX-License-Identifier: GPL-3.0-only

//! Physical drives and how much of them is used by mounted filesystems.

use std::collections::{HashMap, HashSet};
use std::fs::{read_dir, read_link, read_to_string};
use std::path::Path;
use sysinfo::Disks;

const SYS_BLOCK: &str = "/sys/class/block";

#[derive(Clone, Debug)]
pub struct Drive {
    /// Kernel name, e.g. `nvme0n1`
    pub name: String,
    /// Model reported by the drive, falls back to `name`
    pub model: String,
    pub size: u64,
    /// Space used by mounted filesystems on this drive, `None` if nothing is mounted
    pub used: Option<u64>,
}

/// Lists physical drives and adds up the used space of the filesystems mounted on them.
pub fn drives(disks: &Disks) -> Vec<Drive> {
    let mut used: HashMap<String, u64> = HashMap::new();
    // The same filesystem can show up at many mount points (e.g. inside a flatpak sandbox)
    let mut seen = HashSet::new();
    for disk in disks.list() {
        let Some(block) = block_name(&disk.name().to_string_lossy()) else {
            continue;
        };
        if !seen.insert(block.clone()) {
            continue;
        }
        let fs_used = disk.total_space().saturating_sub(disk.available_space());
        for parent in physical_parents(&block) {
            *used.entry(parent).or_default() += fs_used;
        }
    }

    let Ok(entries) = read_dir("/sys/block") else {
        return Vec::new();
    };
    let mut drives: Vec<Drive> = entries
        .filter_map(|entry| {
            let name = entry.ok()?.file_name().to_string_lossy().into_owned();
            let path = Path::new("/sys/block").join(&name);
            // Skip virtual devices: device mapper, loop, zram, ram disks
            if path.join("dm").exists()
                || ["loop", "zram", "ram"].iter().any(|p| name.starts_with(p))
            {
                return None;
            }
            let sectors: u64 = read_to_string(path.join("size")).ok()?.trim().parse().ok()?;
            if sectors == 0 {
                return None;
            }
            let model = read_to_string(path.join("device/model"))
                .map(|m| m.trim().to_string())
                .ok()
                .filter(|m| !m.is_empty())
                .unwrap_or_else(|| name.clone());
            Some(Drive {
                used: used.get(&name).copied(),
                model,
                size: sectors * 512,
                name,
            })
        })
        .collect();
    drives.sort_by(|a, b| a.name.cmp(&b.name));
    drives
}

/// Maps a device path from the mount table (`/dev/nvme0n1p2`, `/dev/mapper/data-root`)
/// to its name in `/sys/class/block` (`nvme0n1p2`, `dm-1`).
fn block_name(device: &str) -> Option<String> {
    let device = device.strip_prefix("/dev/")?;
    if let Some(mapper) = device.strip_prefix("mapper/") {
        return read_dir("/sys/block").ok()?.find_map(|entry| {
            let entry = entry.ok()?;
            let name = read_to_string(entry.path().join("dm/name")).ok()?;
            (name.trim() == mapper).then(|| entry.file_name().to_string_lossy().into_owned())
        });
    }
    Path::new(SYS_BLOCK)
        .join(device)
        .exists()
        .then(|| device.to_string())
}

/// Follows device mapper slaves and partitions down to the physical drives.
fn physical_parents(block: &str) -> Vec<String> {
    let path = Path::new(SYS_BLOCK).join(block);
    let slaves: Vec<String> = read_dir(path.join("slaves"))
        .map(|entries| {
            entries
                .filter_map(|e| Some(e.ok()?.file_name().to_string_lossy().into_owned()))
                .collect()
        })
        .unwrap_or_default();
    if !slaves.is_empty() {
        return slaves.iter().flat_map(|s| physical_parents(s)).collect();
    }
    if path.join("partition").exists() {
        // /sys/class/block/nvme0n1p2 -> .../nvme0n1/nvme0n1p2
        if let Some(parent) = read_link(&path)
            .ok()
            .and_then(|target| target.parent()?.file_name().map(|n| n.to_string_lossy().into_owned()))
        {
            return vec![parent];
        }
    }
    vec![block.to_string()]
}
