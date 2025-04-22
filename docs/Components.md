# Components

The components config is a list of monitored resources, and can contain duplicates.
Each component contains a list of views, i.e., how the component is displayed.

There are currently 5 kinds of components:

- `Cpu`: monitors cpu global and per-core usage
- `Mem`: monitors RAM and Swap usage
- `Net`: monitors network upload/download
- `Disk`: monitors disk read/write
- `Gpu`: monitors GPU usage and VRAM usage

There are 2 types of views, each with their own config:

- Run charts: shows a graph the [samples](./Sampling.md) stored in memory.
- Bar charts: shows a bar with height relative to the current sample.

## Run charts

For the components that monitor 2 values (`Mem`,`Net`,`Disk`,`Gpu`), run charts can be drawn with a single value of with both values. 
The `aspect_ratio` field corresponds to the height and width ratio, while the color fields are explained in the [colors section](#colors).

```ron
[
    RunChart(
        color_back: accent_pink,
        color_front: accent_orange,
        aspect_ratio: 1.5,
    ),
    RunChartBack(
        color: accent_pink,
        aspect_ratio: 1.5,
    ),
    RunChartFront(
        color: accent_orange,
        aspect_ratio: 1.5,
    )
]
```

The following aliases can be used to help configuring:

| Component | `RunChartBack`     | `color_back`     | `RunChartFront`  | `color_front`  |
|-----------|--------------------|------------------|------------------|----------------|
| `Mem`     | `RunChartRam`      | `color_ram`      | `RunChartSwap`   | `color_swap`   |
| `Net`     | `RunChartDownload` | `color_download` | `RunChartUpload` | `color_upload` |
| `Disk`    | `RunChartRead`     | `color_read`     | `RunChartWrite`  | `color_write`  |
| `Gpu`     | `RunChartUsage`    | `color_usage`    | `RunChartVram`   | `color_vram`   |

## Bar charts

For the components that monitor 2 values (`Mem`,`Gpu`), bar charts can be drawn with a single value of with both values. 
The `aspect_ratio` field corresponds to the height and width ratio per bar, while the color fields are explained in the [colors section](#colors).
The `spacing` field corresponds to the spacing between bars in the `BarChart` view.

```ron
[
    BarChart(
        color_left: accent_green,
        color_right: accent_purple,
        spacing: 2.5,
        aspect_ratio: 0.5,
    ),
    BarChartLeft(
        color: accent_green,
        aspect_ratio: 0.5,
    ),
    BarChartRight(
        color: accent_purple,
        aspect_ratio: 0.5,
    ),
]
```

The following aliases can be used to help configuring:

| Component | `BarChartLeft`  | `color_back`  | `BarChartLeft` | `color_front` |
|-----------|-----------------|---------------|----------------|---------------|
| `Mem`     | `BarChartRam`   | `color_ram`   | `BarChartSwap` | `color_swap`  |
| `Gpu`     | `BarChartUsage` | `color_usage` | `BarChartVram` | `color_vram`  |

## Cpu views

The `Cpu` component monitors global usage, which can be displayed as a run chart or bar chart, or per-core usage, which can be displayed only as a bar chart.
The `color` and `aspect_ratio` work the same way as defined in [Run charts](#run-charts) and [Bar charts](#bar-charts).

```ron
[
    RunChart(
        color: accent_purple,
        aspect_ratio: 0.5,
    ),
    BarGlobal(
        color: accent_green,
        aspect_ratio: 0.5,
    ),
    BarCores(
        color: accent_green,
        spacing: 2.5,
        aspect_ratio: 0.5,
    )
]
```

## Colors

You can use colors defined in [CosmicPaletteInner](https://pop-os.github.io/libcosmic/cosmic/cosmic_theme/struct.CosmicPaletteInner.html), as well colors defined in the following format:

```ron
(
    red: 0.0,
    green: 0.0,
    blue: 255.0,
    alpha: 1.0,
)
```

## Example

```ron
[
    Cpu([
        RunChart(
            color: accent_blue,
            aspect_ratio: 1.5,
        ),
        BarGlobal(
            color: accent_blue,
            aspect_ratio: 0.5,
        ),
    ]),
    Mem([
        RunChart(
            color_back: accent_green,
            color_front: accent_purple,
            aspect_ratio: 1.5,
        ),
        BarChart(
            color_left: accent_green,
            color_right: accent_purple,
            spacing: 2.5,
            aspect_ratio: 0.5,
        ),
    ]),
    Disk([
        RunChart(
            color_back: accent_pink,
            color_front: accent_orange,
            aspect_ratio: 1.5,
        ),
    ]),
    Net([
        RunChart(
            color_back: accent_red,
            color_front: accent_yellow,
            aspect_ratio: 1.5,
        ),
    ]),
]
```