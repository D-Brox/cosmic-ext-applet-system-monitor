use crate::{
    config::Swap as SwapConfig,
    sysmon::{
        monitor_module::{init_data_points, Refresh},
        SourceCollection,
    },
};
use std::marker::PhantomData;
use sysinfo::MemoryRefreshKind;

use super::{Configurable, SingleModule};

pub type SwapModule = SingleModule<SwapConfig>;

impl Refresh for SwapModule {
    fn tick(&mut self, source: &mut SourceCollection) {
        source
            .sys
            .refresh_memory_specifics(MemoryRefreshKind::nothing().with_swap());
        let total_swap = source.sys.total_swap() as f64;
        let used_swap = source.sys.used_swap() as f64;
        let percentage = ((used_swap / total_swap) * 100.0) as i64;

        self.data.push(percentage);
    }
}

impl From<SwapConfig> for SwapModule {
    fn from(c: SwapConfig) -> Self {
        Self {
            data: init_data_points(c.history_size).into(),
            vis: c.vis,
            config: PhantomData,
        }
    }
}

impl Configurable for SwapModule {
    type Config = SwapConfig;

    fn configure(&mut self, config: impl Into<SwapConfig>) {
        let SwapConfig {
            history_size, vis, ..
        } = config.into();
        self.data.configure(history_size);
        self.vis = vis;
    }
}
