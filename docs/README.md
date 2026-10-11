# Configuring the applet

You can configure the applet editing the files in `~/.config/cosmic/dev.DBrox.CosmicSystemMonitor/v2/`.

The configuration is divided in 3 files:

- [Components](./Components.md): controls what resource monitors are displayed and how they are viewed
- [Sampling](./Sampling.md): controls the speed of the sampling and how many samples are stored for each of the resources monitored
- [Layout](./Layout.md): controls the padding and spacing of the components

These popup-related files are also available:

- `tooltip_enabled` (`true`/`false`): show tooltips when hovering the in-panel charts
- `popup_mode` (`true`/`false`, default `false`): show a single icon in the panel and the charts in a popup when it is clicked
- `popup_expanded` (list, e.g. `["cpu", "gpu0"]`): popup sections that are open; all others start collapsed. Keys: `cpu`, `mem`, `disk`, `net`, `gpu0`, `gpu1`, …, plus `disk-others` for the unmounted drives list
