# System Monitor Cosmic Applet

A highly configurable system resource monitor for the COSMIC DE 

![screenshot of the applet](./res/screenshot.png)


## Dependencies

- libfontconfig-dev
- libxkbcommon-dev

Or equivalent packages in non-debian based distros.

## Install

Clone the repo and run the commands corresponding to your distro:

```sh
git clone https://github.com/D-Brox/cosmic-ext-applet-system-monitor 
cd cosmic-ext-applet-system-monitor 

# Debian based distros
just build-deb
sudo just install-deb

# RPM based distros
just build-rpm
sudo just install-rpm

# Arch based distros
just install-aur ${aur_helper}

# For other distros
just build-release
# Global install (root)
sudo just install
# or local install (user)
just install-local
```

## Roadmap

- [x] CPU usage
- [x] Memory usage (RAM and swap)
- [x] Network chart (upload/download)
- [x] Disk chart (write/read)
- [ ] GPU VRAM chart (help needed)
- [x] Displayed charts config
- [x] Sampling configs
- [x] Chart theming
- [x] Vertical charts (for left/right panels)
- [ ] Popup (general system info)

## Configuring

You can configure the charts displayed by editing `~/.config/cosmic/dev.DBrox.CosmicSystemMonitor/v1/charts`. Only charts in this config will be displayed. `VRAM` will be ignored until it is implemented.

The fields `update_interval`, `samples` and `size` are the sampling time in milliseconds, the total number of samples displayed and the size relative to the panel height (top/bottom panels), respectively.

You can use colors defined in [CosmicPaletteInner](https://pop-os.github.io/libcosmic/cosmic/cosmic_theme/struct.CosmicPaletteInner.html), as well as `rgb("")` with a hexcode.

Example config where the CPU, RAM, Swap, Net and Disk charts are displayed, in this order:
```ron
[
    CPU((
        update_interval: 1000,
        samples: 60,
        size: 1.5,
        color: accent_blue,
    )),
    RAM((
        update_interval: 2000,
        samples: 30,
        size: 1.5,
        color: accent_green,
    )),
    Swap((
        update_interval: 5000,
        samples: 12,
        size: 1.5,
        color: accent_purple,
    )),
     Net((
         update_interval: 1000,
         samples: 60,
         size: 1.5,
         color_up: accent_yellow,
         color_down: accent_red,
     )),
     Disk((
         update_interval: 2000,
         samples: 30,
         color_read: accent_orange,
         color_write: accent_pink,
         size: 1.5,
     )),
    // VRAM((
    //     update_interval: 1000,
    //     samples: 60,
    //     color: accent_indigo,
    //     size: 1.5,
    // )),
]
```

## Contributing

Contributions are welcome

To build and install the debug build

```sh
just build-debug && sudo just debug=1 install
```

## Special Thanks

- [paradoxxxzero](https://github.com/paradoxxxzero) for their [GNOME Shell system monitor extension](https://github.com/paradoxxxzero/gnome-shell-system-monitor-applet), the inspiration for this applet
- [edfloreshz](https://github.com/edfloreshz) for the [template for COSMIC applets](https://github.com/edfloreshz/cosmic-applet-template), which taught me the logic behind an applet
- [aschiavon91](https://github.com/aschiavon91) for their initial work at a [system status applet](https://github.com/aschiavon91/cosmic-applet-sys-status/), which was used as a reference implementation
- [Joylei](https://github.com/Joylei) for implementing an [Iced backend for `plotters`](https://github.com/Joylei/plotters-iced), used at the core of this applet

## Known wgpu issue

There are currently some rendering issues with the `wgpu` libcosmic features in some older hybrid gpus.
If you are affected by this, you can build and install it with this feature disabled:

```sh
just build-no-wgpu

# Debin based
command -v cargo-deb || cargo install cargo-deb
cargo deb
sudo just install-deb

# RPM based
strip -s target/release/cosmic-ext-applet-system-monitor
cargo generate-rpm
sudo just install-rpm

# Other distros
sudo just install
```