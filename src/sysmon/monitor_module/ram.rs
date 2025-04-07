use crate::{
    config::{Ram as RamConfig, SingleView},
    sysmon::{
        monitor_module::{init_data_points, Refresh},
        SourceCollection,
    },
};
use std::marker::PhantomData;
use sysinfo::MemoryRefreshKind;

use super::{Configurable, History, SingleModule};

pub type RamData = History;

impl Refresh for RamModule {
    fn tick(&mut self, source: &mut SourceCollection) {
        source
            .sys
            .refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
        let total_ram = source.sys.total_memory() as f64;
        let used_ram = source.sys.used_memory() as f64;
        let percentage = ((used_ram / total_ram) * 100.0) as i64;
        self.data.push(percentage);
    }
}

impl From<RamConfig> for RamModule {
    fn from(c: RamConfig) -> Self {
        Self {
            data: init_data_points(c.history_size),
            vis: c.vis,
            config: PhantomData,
        }
    }
}

pub type RamModule = SingleModule<RamConfig>;

impl From<RamConfig> for (usize, Box<[SingleView]>) {
    fn from(config: RamConfig) -> Self {
        let RamConfig {
            history_size, vis, ..
        } = config;
        (history_size.into(), vis)
    }
}

impl Configurable for RamModule {
    type Config = RamConfig;

    fn configure(&mut self, config: impl Into<RamConfig>) {
        let RamConfig {
            history_size, vis, ..
        } = config.into();
        self.data.configure(history_size);
        self.vis = vis;
    }
}
