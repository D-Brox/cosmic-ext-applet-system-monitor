# System Monitor Cosmic Applet

A highly configurable system resource monitor for the COSMIC DE 

![screenshot of the applet](./res/screenshot.png)

The instructions for configuring are located in the [documentation](./docs/README.md)

## Installing

You can just grab the `.deb`, `.rpm` or tarball from the [releases](https://github.com/D-Brox/cosmic-ext-applet-system-monitor/releases/latest) page.


## Building from source

Clone the repository

```sh
git clone https://github.com/D-Brox/cosmic-ext-applet-system-monitor 
cd cosmic-ext-applet-system-monitor 
```

Install the build dependencies (or equivalent packages in non debian-based distros):

- rustc/cargo
- just
- libxkbcommon-dev


Build and install the project:

```bash
just build-release
sudo just install 
# or
just install-local
```

For alternative packaging methods, use the one of the following recipes:

- `deb`: run `just build-deb` and `sudo just install-deb`
- `rpm`: run `just build-rpm` and `sudo just install-rpm`

For vendoring, use `just vendor` and `just vendor-build`

## Roadmap

Theming:
- [x] Layout
- [x] Custom colors
- [ ] Transparency

Resource monitoring:
- [x] CPU usage (global and per core)
- [x] Memory usage (RAM and Swap)
- [x] Network I/O
- [x] Disk I/O
- [ ] GPU (usage and VRAM) (WIP, testing needed for Nvidia)
- [ ] Thermal sensors

[Component](./docs/Components.md) views 
- [x] Run chart views (percentage and I/O)
- [x] Bar chart views (percentage and CPU cores)
- [ ] Text views
- [ ] Popup (general system info)

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

## Known wgpu issue

There are currently some rendering issues with the `wgpu` libcosmic features in some older hybrid gpus.
If you are affected by this, you can build and install it with this feature disabled, however this may lead to other problems:

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
