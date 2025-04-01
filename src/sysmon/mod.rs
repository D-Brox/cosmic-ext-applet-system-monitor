// SPDX-License-Identifier: GPL-3.0-only
use crate::sysmon::monitor_item::MonitorItem;
mod bar_chart;
mod chart;
mod cpu_chart;
mod monitor_item;
// mod gpu;
// use gpu::Gpus,
use crate::sysmon::alignment::Horizontal;
use crate::sysmon::alignment::Vertical;
use crate::{
    applet::Message,
    config::{ChartConfig, Config},
};
use chart::{DoubleData, SingleData};
use cosmic::widget::Text;

use cosmic::iced::alignment;
use cosmic::{applet, Element, Theme};
use sysinfo::{Disks, MemoryRefreshKind, Networks, System};

use crate::sysmon::cpu_chart::CpuData;

use crate::fl;

pub struct SystemMonitor {
    sys: System,
    nets: Networks,
    disks: Disks,
    // gpus: Gpus,
    // panel_items: Vec<Rc<dyn MonitorItem>>,
    charts: Vec<UsedChart>,

    cpu: Option<CpuData>,
    ram: Option<SingleData>,
    swap: Option<SingleData>,
    net: Option<DoubleData>,
    disk: Option<DoubleData>,
    // vram: Option<SingleData>,
}

impl SystemMonitor {
    pub fn new(config: Config, theme: &Theme) -> Self {
        let mut new_self = Self {
            sys: System::new(),
            nets: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),
            // gpus: Gpus {},
            charts: vec![],
            cpu: None,
            ram: None,
            swap: None,
            net: None,
            disk: None,
            // vram: None,
        };
        new_self.update_config(config, theme);
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
        if let Some(cpu) = &mut self.cpu {
            self.sys.refresh_cpu_usage();
            let cpu_data = self.sys.global_cpu_usage() as i64;

            cpu.update_latest_per_core(self.sys.cpus());
            cpu.push_data(cpu_data);
        }
    }

    pub fn update_ram(&mut self) {
        if let Some(ram) = &mut self.ram {
            self.sys
                .refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
            let total_ram = self.sys.total_memory() as f64;
            let used_ram = self.sys.used_memory() as f64;
            let ram_data = ((used_ram / total_ram) * 100.0) as i64;

            ram.push_data(ram_data);
        }
    }
    pub fn update_swap(&mut self) {
        if let Some(swap) = &mut self.swap {
            self.sys
                .refresh_memory_specifics(MemoryRefreshKind::nothing().with_swap());
            let total_swap = self.sys.total_swap() as f64;
            let used_swap = self.sys.used_swap() as f64;
            let ram_swap = ((used_swap / total_swap) * 100.0) as i64;

            swap.push_data(ram_swap);
        }
    }

    pub fn update_net(&mut self) {
        if let Some(net) = &mut self.net {
            self.nets.refresh(true);
            let mut upload = 0;
            let mut download = 0;

            for (_, data) in &self.nets {
                upload += data.transmitted();
                download += data.received();
            }

            net.push_data(upload, download);
        }
    }

    pub fn update_disk(&mut self) {
        if let Some(disk) = &mut self.disk {
            self.disks.refresh(true);
            let mut write = 0;
            let mut read = 0;

            for disk in &self.disks {
                let usage = disk.usage();
                write += usage.written_bytes;
                read += usage.read_bytes;
            }

            disk.push_data(write, read);
        }
    }

    // pub fn update_vram(&mut self, _theme: &Theme) {
    //     if let Some(vram) = self.vram {}
    // }

    pub fn update_config(&mut self, config: Config, theme: &Theme) {
        // todo can be radically simplified to utilize MonitorItem methods
        let mut charts = Vec::new();
        for chart in config.charts {
            match chart {
                ChartConfig::CPU(c) => {
                    charts.push(UsedChart::Cpu);
                    if let Some(cpu) = &mut self.cpu {
                        cpu.update_colors(c.color.as_cosmic_color(theme).into());
                        cpu.resize_queue(c.samples);
                    } else {
                        self.cpu = Some(CpuData::new((c, self.sys.cpus().len()), theme));
                    }
                }
                ChartConfig::RAM(c) => {
                    charts.push(UsedChart::Ram);
                    if let Some(ram) = &mut self.ram {
                        ram.update_colors(c.color, theme);
                        ram.resize_queue(c.samples);
                        ram.update_aspect_ratio(c.aspect_ratio);
                    } else {
                        self.ram = Some(SingleData::new(c, theme));
                    }
                }
                ChartConfig::Swap(c) => {
                    charts.push(UsedChart::Swap);
                    if let Some(swap) = &mut self.swap {
                        swap.update_colors(c.color, theme);
                        swap.resize_queue(c.samples);
                        swap.update_aspect_ratio(c.aspect_ratio);
                    } else {
                        self.swap = Some(SingleData::new(c, theme));
                    }
                }
                ChartConfig::Net(c) => {
                    charts.push(UsedChart::Net);
                    if let Some(net) = &mut self.net {
                        net.update_colors(c.color_up, c.color_down, theme);
                        net.resize_queue(c.samples);
                        net.update_aspect_ratio(c.aspect_ratio);
                    } else {
                        self.net = Some(DoubleData::new(c.into(), theme));
                    }
                }
                ChartConfig::Disk(c) => {
                    charts.push(UsedChart::Disk);
                    if let Some(disk) = &mut self.disk {
                        disk.update_colors(c.color_write, c.color_read, theme);
                        disk.resize_queue(c.samples);
                        disk.update_aspect_ratio(c.aspect_ratio);
                    } else {
                        self.disk = Some(DoubleData::new(c.into(), theme));
                    }
                }
                ChartConfig::VRAM(_) => (),
                // ChartConfig::VRAM(c) => {
                //     charts.push(UsedChart::Vram);
                // if let Some(vram) = &mut self.vram {
                //         vram.update_colors(c.color.clone(), theme);
                //         vram.resize_queue(c.samples);
                //     } else {
                //         self.vram = Some(SingleChart::new(
                //             c.color.clone(),
                //             c.size,
                //             c.samples,
                //             theme,
                //         ));
                //     }
                // },
            }
        }
        self.charts = charts;
    }

    pub fn view<'a>(&'a self, context: &'a applet::Context) -> Element<'a, Message> {
        let mut vec: Vec<Element<Message>> = Vec::with_capacity(5);

        let spacing = 5;
        for item in &self.charts {
            vec.push(match item {
                UsedChart::Cpu => self.cpu.as_ref().unwrap().view(context, spacing),
                UsedChart::Ram => self.ram.as_ref().unwrap().view(context, spacing),
                UsedChart::Swap => self.swap.as_ref().unwrap().view(context, spacing),
                UsedChart::Net => self.net.as_ref().unwrap().view(context, spacing),
                UsedChart::Disk => self.disk.as_ref().unwrap().view(context, spacing),
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
        crate::helpers::collection(context, vec, 30)
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
