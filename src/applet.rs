// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    Application, Apply as _, Element, Theme,
    app::{Core, Task},
    cosmic_config,
    iced::Subscription,
    surface,
    widget::container,
};
use std::time::Duration;
use sysinfo::{
    CpuRefreshKind, Disk, DiskRefreshKind, Disks, MemoryRefreshKind, Networks, RefreshKind, System,
};

use crate::{
    components::gpu::Gpus,
    config::{ComponentConfig, Config, config_subscription},
    history::History,
};

pub const ID: &str = "dev.DBrox.CosmicSystemMonitor";

pub struct SystemMonitorApplet {
    pub core: Core,
    pub config: Config,
    #[allow(dead_code)]
    config_handler: Option<cosmic_config::Config>,

    pub sys: System,
    pub nets: Networks,
    pub disks: Disks,
    pub gpus: Gpus,
    /// percentage global cpu used between refreshes
    pub global_cpu: History<f32>,
    pub ram: History,
    pub swap: History,
    /// amount uploaded between refresh of `sysinfo::Nets`. (DOES NOT STORE RATE)
    pub upload: History,
    /// amount downloaded between refresh of `sysinfo::Nets`. (DOES NOT STORE RATE)
    pub download: History,
    /// amount read between refresh of `sysinfo::Disks`. (DOES NOT STORE RATE)
    pub disk_read: History,
    /// amount written between refresh of `sysinfo::Disks`. (DOES NOT STORE RATE)
    pub disk_write: History,
    /// amount read between refresh of `sysinfo::Disks`. (DOES NOT STORE RATE)
    pub gpu_usage: Vec<History>,
    /// amount written between refresh of `sysinfo::Disks`. (DOES NOT STORE RATE)
    pub vram: Vec<History>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Config(Config),
    TickCpu,
    TickMem,
    TickNet,
    TickDisk,
    TickGpu,
    Surface(surface::Action),
}

#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: Config,
}

impl Application for SystemMonitorApplet {
    type Executor = cosmic::executor::Default;

    type Flags = Flags;

    type Message = Message;

    const APP_ID: &'static str = ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let (mut cpu, mut mem, mut net, mut disk, mut gpu) = Default::default();
        let sampling = &flags.config.sampling;
        for chart_config in &flags.config.components {
            match chart_config {
                ComponentConfig::Cpu(_) => cpu = sampling.cpu.sampling_window,
                ComponentConfig::Mem(_) => mem = sampling.mem.sampling_window,
                ComponentConfig::Net(_) => net = sampling.net.sampling_window,
                ComponentConfig::Disk(_) => disk = sampling.disk.sampling_window,
                ComponentConfig::Gpu(_) => gpu = sampling.gpu.sampling_window,
            }
        }
        let gpus = Gpus::new();
        let app = Self {
            core,
            config: flags.config,
            config_handler: flags.config_handler,

            global_cpu: History::with_capacity(cpu),
            ram: History::with_capacity(mem),
            swap: History::with_capacity(mem),
            upload: History::with_capacity(net),
            download: History::with_capacity(net),
            disk_read: History::with_capacity(disk),
            disk_write: History::with_capacity(disk),
            gpu_usage: vec![History::with_capacity(gpu); gpus.num_gpus()],
            vram: vec![History::with_capacity(gpu); gpus.num_gpus()],

            sys: System::new_with_specifics(
                RefreshKind::nothing()
                    .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
                    .with_memory(MemoryRefreshKind::everything()),
            ),
            nets: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list_specifics(
                DiskRefreshKind::nothing().with_io_usage(),
            ),
            gpus,
        };

        (app, Task::none())
    }

    fn view(&'_ self) -> Element<'_, Message> {
        let item_iter = self.config.components.iter().map(|module| {
            match module {
                ComponentConfig::Cpu(vis) => self.cpu_view(vis),
                ComponentConfig::Mem(vis) => self.mem_view(vis),
                ComponentConfig::Net(vis) => self.net_view(vis),
                ComponentConfig::Disk(vis) => self.disk_view(vis),
                ComponentConfig::Gpu(vis) => self.gpu_view(vis),
            }
            .apply(|elements| {
                self.panel_collection(elements, self.config.layout.inner_spacing, 0.0)
            })
        });

        let items = self.panel_collection(item_iter, self.config.layout.spacing, self.padding());

        self.core.applet.autosize_window(items).into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Config(config) => {
                self.config = config;
                let sampĺing = &self.config.sampling;
                self.global_cpu.resize(sampĺing.cpu.sampling_window);
                self.ram.resize(sampĺing.mem.sampling_window);
                self.swap.resize(sampĺing.mem.sampling_window);
                self.upload.resize(sampĺing.net.sampling_window);
                self.download.resize(sampĺing.net.sampling_window);
                self.disk_read.resize(sampĺing.disk.sampling_window);
                self.disk_write.resize(sampĺing.disk.sampling_window);
                for i in 0..self.gpus.num_gpus() {
                    self.gpu_usage[i].resize(sampĺing.cpu.sampling_window);
                    self.vram[i].resize(sampĺing.cpu.sampling_window);
                }
            }
            Message::TickCpu => {
                self.sys.refresh_cpu_usage();
                self.global_cpu.push(self.sys.global_cpu_usage());
            }
            Message::Surface(a) => {
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(a),
                ));
            }
            Message::TickMem => {
                self.sys.refresh_memory();
                self.ram.push(self.sys.used_memory());
                self.swap.push(self.sys.used_swap());
            }
            Message::TickNet => {
                self.nets.refresh(true);
                let (received, transmitted) =
                    self.nets.iter().fold((0, 0), |(acc_r, acc_t), (_, data)| {
                        (acc_r + data.received(), acc_t + data.transmitted())
                    });
                self.upload.push(transmitted);
                self.download.push(received);
            }
            Message::TickDisk => {
                self.disks
                    .refresh_specifics(true, DiskRefreshKind::nothing().with_io_usage());
                let (read, written) = self
                    .disks
                    .iter()
                    .map(Disk::usage)
                    .fold((0, 0), |(acc_r, acc_w), usage| {
                        (acc_r + usage.read_bytes, acc_w + usage.written_bytes)
                    });
                self.disk_read.push(read);
                self.disk_write.push(written);
            }
            Message::TickGpu => {
                self.gpus.refresh();
                for (idx, data) in self.gpus.data().iter().enumerate() {
                    self.gpu_usage[idx].push(data.usage);
                    self.vram[idx].push(data.used_vram);
                }
            }
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let mut subs = Vec::new();
        let sampling = &self.config.sampling;
        for chart in &self.config.components {
            let tick = {
                match chart {
                    ComponentConfig::Cpu(_) => cosmic::iced::time::every(Duration::from_millis(
                        sampling.cpu.update_interval,
                    ))
                    .map(|_| Message::TickCpu),
                    ComponentConfig::Mem(_) => cosmic::iced::time::every(Duration::from_millis(
                        sampling.mem.update_interval,
                    ))
                    .map(|_| Message::TickMem),
                    ComponentConfig::Net(_) => cosmic::iced::time::every(Duration::from_millis(
                        sampling.net.update_interval,
                    ))
                    .map(|_| Message::TickNet),
                    ComponentConfig::Disk(_) => cosmic::iced::time::every(Duration::from_millis(
                        sampling.disk.update_interval,
                    ))
                    .map(|_| Message::TickDisk),
                    ComponentConfig::Gpu(_) => cosmic::iced::time::every(Duration::from_millis(
                        sampling.gpu.update_interval,
                    ))
                    .map(|_| Message::TickGpu),
                }
            };
            subs.push(tick);
        }

        subs.push(config_subscription());

        Subscription::batch(subs)
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}

pub fn base_background(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(cosmic::iced::Color::from(theme.cosmic().primary.base).into()),
        ..container::Style::default()
    }
}
