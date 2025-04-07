use std::marker::PhantomData;

use super::{init_data_points, Configurable, DoubleData, DoubleModule};
use crate::config::Network as NetConfig;

pub type NetModule = DoubleModule<NetConfig>;

impl From<NetConfig> for NetModule {
    fn from(config: NetConfig) -> Self {
        let NetConfig {
            history_size: samples,
            vis: visualization,
            ..
        } = config;

        let history = init_data_points(samples);
        let data = DoubleData {
            history1: history.clone(),
            history2: history,
        };

        Self {
            data,
            vis: visualization,
            config: PhantomData,
        }
    }
}

impl Configurable for NetModule {
    type Config = NetConfig;

    fn configure(&mut self, config: impl Into<Self::Config>) {
        let NetConfig {
            history_size, vis, ..
        } = config.into();
        self.data.configure(history_size);
        self.vis = vis;
    }
}
