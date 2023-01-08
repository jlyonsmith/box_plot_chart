mod log_macros;
mod quartile;

use chrono::NaiveDate;
use clap::Parser;
use core::fmt::Arguments;
use hypermelon::build;
use hypermelon::prelude::*;
use hypermelon::render;
use hypermelon::tools::WriteWrap;
use quartile::Quartile;
use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub trait BoxPlotChartLog {
    fn output(self: &Self, args: Arguments);
    fn warning(self: &Self, args: Arguments);
    fn error(self: &Self, args: Arguments);
}

pub struct BoxPlotChartTool<'a> {
    log: &'a dyn BoxPlotChartLog,
}

#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Cli {
    /// Specify the JSON data file
    #[clap(value_name = "INPUT_FILE")]
    input_file: PathBuf,

    #[clap(value_name = "OUTPUT_FILE")]
    output_file: PathBuf,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ChartData {
    pub title: String,
    pub units: String,
    pub data: Vec<ItemData>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ItemData {
    pub key: String,
    pub values: Vec<f64>,
}

struct RenderData {
    chart_width: f64,
    chart_height: f64,
    y_axis_width: f64,
    x_axis_height: f64,
    box_plot_width: f64,
    box_width: f64,
    outlier_radius: f64,
    styles: Vec<String>,
    quartiles: Vec<(String, Quartile)>,
}

impl<'a> BoxPlotChartTool<'a> {
    pub fn new(log: &'a dyn BoxPlotChartLog) -> BoxPlotChartTool {
        BoxPlotChartTool { log }
    }

    pub fn run(
        self: &mut Self,
        args: impl IntoIterator<Item = std::ffi::OsString>,
    ) -> Result<(), Box<dyn Error>> {
        let cli = match Cli::try_parse_from(args) {
            Ok(m) => m,
            Err(err) => {
                output!(self.log, "{}", err.to_string());
                return Ok(());
            }
        };

        let chart_data = Self::read_chart_file(&cli.input_file)?;
        let render_data = self.process_chart_data(&chart_data)?;
        let output = self.render_chart(&render_data)?;

        let mut file = fs::File::create(cli.output_file)?;

        file.write_all(output.as_bytes());

        Ok(())
    }

    fn read_chart_file(chart_file: &PathBuf) -> Result<ChartData, Box<dyn Error>> {
        let content = fs::read_to_string(chart_file)?;
        let chart_data: ChartData = json5::from_str(&content)?;

        Ok(chart_data)
    }

    fn process_chart_data(
        self: &Self,
        chart_data: &ChartData,
    ) -> Result<RenderData, Box<dyn Error>> {
        // TODO(john): Calculate the min/max of the y-axis range

        fn quartile_tuple(item_data: &ItemData) -> Result<(String, Quartile), Box<dyn Error>> {
            Ok((item_data.key.to_owned(), Quartile::new(&item_data.values)?))
        }

        let quartiles = chart_data
            .data
            .iter()
            .map(quartile_tuple)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let x_axis_height = 20.0;
        let y_axis_width = 20.0;
        let box_plot_width = 20.0;
        let chart_width = y_axis_width + ((quartiles.len() as f64) * box_plot_width);
        let chart_height = x_axis_height + 100.0;

        Ok(RenderData {
            chart_width,
            chart_height,
            x_axis_height,
            y_axis_width,
            box_plot_width,
            box_width: 5.0,
            outlier_radius: 0.0,
            styles: vec![
                ".whisker-style { stroke: rgb(0,0,0); stroke-width: 1; }".to_owned(),
                ".box-style { stroke: rgb(0,0,0); stroke-width: 1; }".to_owned(),
                ".outlier-style {}".to_owned(),
                ".axis-style { stroke: rgb(0,0,0); stroke-width: 1; }".to_owned(),
            ],
            quartiles: quartiles,
        })
    }

    fn render_chart(self: &Self, render_data: &RenderData) -> Result<String, Box<dyn Error>> {
        let style = build::elem("style").append(build::from_iter(render_data.styles.iter()));

        let svg = build::elem("svg").with(attrs!(
            ("xmlns", "http://www.w3.org/2000/svg"),
            ("width", render_data.chart_width),
            ("height", render_data.chart_height),
            (
                "viewBox",
                format_move!(
                    "0 0 {} {}",
                    render_data.chart_width,
                    render_data.chart_height
                )
            ),
            ("style", "background-color: white;")
        ));

        // TODO(john): Render the x-axis
        let y_axis = build::single("line").with(attrs!(
            ("class", "axis-style"),
            ("x1", render_data.y_axis_width),
            ("y1", 0.0),
            ("x2", render_data.y_axis_width),
            ("y2", render_data.chart_height - render_data.x_axis_height)
        ));
        // TODO(john): Add the y-axis labels
        let x_axis = build::single("line").with(attrs!(
            ("class", "axis-style"),
            ("x1", render_data.y_axis_width),
            ("y1", render_data.chart_height - render_data.x_axis_height),
            ("x2", render_data.chart_width),
            ("y2", render_data.chart_height - render_data.x_axis_height)
        ));
        // TODO(john): Add x-axis labels
        // TODO(john): Add y-axis lines
        // TODO(john): Render one box plot
        // TODO(john): Render all the box plots
        // TODO(john): Render the chart title

        let mut output = String::new();
        let all = svg.append(style).append(y_axis).append(x_axis);

        hypermelon::render(all, &mut output)?;

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_test() {
        struct TestLogger;

        impl TestLogger {
            fn new() -> TestLogger {
                TestLogger {}
            }
        }

        impl BoxPlotChartLog for TestLogger {
            fn output(self: &Self, _args: Arguments) {}
            fn warning(self: &Self, _args: Arguments) {}
            fn error(self: &Self, _args: Arguments) {}
        }

        let logger = TestLogger::new();
        let mut tool = BoxPlotChartTool::new(&logger);
        let args: Vec<std::ffi::OsString> = vec!["".into(), "--help".into()];

        tool.run(args).unwrap();
    }
}
