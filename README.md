# Box Plot Chart Generator

[![coverage](https://shields.io/endpoint?url=https://raw.githubusercontent.com/jlyonsmith/box_plot_chart/main/coverage.json)](https://github.com/jlyonsmith/box_plot_chart/blob/main/coverage.json)
[![Crates.io](https://img.shields.io/crates/v/box_plot_chart.svg)](https://crates.io/crates/box_plot_chart)
[![Docs.rs](https://docs.rs/box_plot_chart/badge.svg)](https://docs.rs/box_plot_chart)

This is a simple box plot generator.  You provide a [JSON5](https://json5.org/) file with data and it generates an SVG file.  You can convert the SVG to PNG or other bitmap formats with the [resvg](https://crates.io/crates/resvg) tool.

Here is an example of the output:

![Example Box Plot](example/example.svg)

Install with `cargo install box_plot_chart`.  Run with `box-plot-chart`.

Features of the tool include:

- Automatic scaling of the Y axis labels
- Shows box, whiskers and outliers

You can understand the box plot composition with the aid of the following graphic:

![Box Plot Components](example/box-plot-components.jpeg)
