use crate::{
    config::{SingleView, Swap as SwapConfig},
    sysmon::{
        monitor_module::{init_data_points, Module, Refresh, SingleData},
        SourceCollection,
    },
};
use std::marker::PhantomData;
use sysinfo::MemoryRefreshKind;

pub type SwapModule = Module<SingleData, SwapConfig, SingleView>;

impl Refresh for SwapModule {
    fn tick(&mut self, source: &mut SourceCollection) {
        source
            .sys
            .refresh_memory_specifics(MemoryRefreshKind::nothing().with_swap());
        let total_swap = source.sys.total_swap() as f64;
        let used_swap = source.sys.used_swap() as f64;
        let percentage = ((used_swap / total_swap) * 100.0) as i64;

        self.data.history.push(percentage);
    }
}

impl From<SwapConfig> for SwapModule {
    fn from(c: SwapConfig) -> Self {
        Self {
            data: init_data_points(c.history_size).into(),
            vis: c.vis,
            color: c.color,
            config: PhantomData,
        }
    }
}
