use crate::{
    config::{Ram as RamConfig, SingleView},
    sysmon::{
        monitor_module::{init_data_points, Module, Refresh, SingleData},
        SourceCollection,
    },
};
use std::marker::PhantomData;
use sysinfo::MemoryRefreshKind;

pub type RamData = SingleData;

impl Refresh for RamModule {
    fn tick(&mut self, source: &mut SourceCollection) {
        source
            .sys
            .refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
        let total_ram = source.sys.total_memory() as f64;
        let used_ram = source.sys.used_memory() as f64;
        let percentage = ((used_ram / total_ram) * 100.0) as i64;
        self.data.history.push(percentage);
    }
}

impl From<RamConfig> for RamModule {
    fn from(c: RamConfig) -> Self {
        Self {
            data: init_data_points(c.history_size).into(),
            vis: c.vis,
            color: c.color,
            config: PhantomData,
        }
    }
}

pub type RamModule = Module<RamData, RamConfig, SingleView>;

impl From<RamConfig> for (usize, Box<[SingleView]>, crate::color::Color) {
    fn from(config: RamConfig) -> Self {
        let RamConfig {
            history_size,
            vis,
            color,
            ..
        } = config;
        (history_size.into(), vis, color)
    }
}
