// SPDX-License-Identifier: GPL-3.0-only

mod bar_chart;
mod chart;
mod cpu_chart;
mod monitor_module;

use crate::sysmon::alignment::Horizontal;
use crate::sysmon::alignment::Vertical;
// mod gpu;
// use gpu::Gpus,
use crate::sysmon::monitor_module::Refresh;
use crate::{
    applet::Message,
    config::{ChartConfig, Config},
};
use cosmic::widget::Text;

use cosmic::iced::alignment;
use cosmic::{applet, Element, Theme};
use monitor_module::{RamModule, SwapModule};
use sysinfo::{Disks, Networks, System};

// use crate::sysmon::cpu_chart::CpuData;

use crate::fl;

pub struct SourceCollection {
    sys: System,
    nets: Networks,
    disks: Disks,
    // gpus: Gpus,
}

pub struct SystemMonitor {
    sources: SourceCollection,

    charts: Box<[UsedChart]>,

    // cpu: CpuModule,
    ram: RamModule,
    swap: SwapModule,
    // net: NetModule,
    // disk: DiskModule,

    // vram: VramModule,
}

impl SystemMonitor {
    pub fn new(config: Config) -> Self {
        let (mut ram, mut swap) = Default::default();
        let charts: Box<[_]> = config
            .charts
            .into_iter()
            .map(|chart_config| match chart_config {
                ChartConfig::Ram(c) => {
                    ram = Some(c);
                    UsedChart::Ram
                }
                ChartConfig::Swap(c) => {
                    swap = Some(c);
                    UsedChart::Swap
                }
            })
            .collect();

        let mut new_self = Self {
            sources: SourceCollection {
                sys: System::new(),
                nets: Networks::new_with_refreshed_list(),
                disks: Disks::new_with_refreshed_list(),
                // gpus: Gpus {},
            },
            charts,
            // cpu: None,
            ram: RamModule::from(ram.unwrap_or_default()),
            swap: SwapModule::from(swap.unwrap_or_default()),
            // net: None,
            // disk: None,
            // vram: None,
        };
        new_self.update_cpu();
        new_self.update_ram();
        new_self.update_swap();
        new_self.update_net();
        new_self.update_disk();
        // new_self.update_vram();
        new_self
    }
}

impl SystemMonitor {
    pub fn update_cpu(&mut self) {
        // todo
    }

    pub fn update_ram(&mut self) {
        self.ram.tick(&mut self.sources);
    }

    pub fn update_swap(&mut self) {
        self.swap.tick(&mut self.sources);
    }

    pub fn update_net(&mut self) {
        // todo
    }

    pub fn update_disk(&mut self) {
        // todo
    }

    // pub fn update_vram(&mut self, _theme: &Theme) {
    //     todo
    // }

    pub fn update_config(&mut self, config: Config, theme: &Theme) {
        self.charts = config
            .charts
            .into_iter()
            .map(|chart| match chart {
                ChartConfig::Ram(c) => {
                    self.ram.configure(c.history_size, c.vis, c.color);
                    UsedChart::Ram
                }
                ChartConfig::Swap(c) => {
                    self.ram.configure(c.history_size, c.vis, c.color);
                    UsedChart::Swap
                }
            })
            .collect();
    }

    pub fn view<'a, 'b: 'a>(&'a self, context: &'b applet::Context) -> Element<'a, Message> {
        let mut vec: Vec<Element<Message>> = Vec::with_capacity(5);

        let spacing = 5;
        for item in &self.charts {
            vec.push(match item {
                UsedChart::Ram => self.ram.view(context, spacing),
                UsedChart::Swap => self.swap.view(context, spacing),
                _ => unimplemented!(),
            });
        }
        if vec.is_empty() {
            vec.push(
                Text::new(fl!("loading"))
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
                    .into(),
            );
        }
        crate::helpers::collection(context, vec, 30, 0.0)
    }
}

#[derive(Clone)]
pub enum UsedChart {
    Cpu,
    Ram,
    Swap,
    Net,
    Disk,
    // Vram,
}
