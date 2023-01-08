mod log_macros;
mod quartile;

use chrono::NaiveDate;
use clap::Parser;
use core::fmt::Arguments;
use hypermelon::build;
use hypermelon::prelude::*;
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
    pub values: Vec<f32>,
}

struct RenderData {
    chart_width: f32,
    chart_height: f32,
    y_axis_width: f32,
    x_axis_width: f32,
    box_plot_width: f32,
    box_width: f32,
    outlier_radius: f32,
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
        // TODO(john): Generate all the Quartiles from the data
        // TODO(john): Calculate the chart width
        // TODO(john): Calculate the min/max of the y-axis range

        let render_data = RenderData {
            chart_width: 100.0,
            chart_height: 100.0,
            x_axis_width: 20.0,
            y_axis_width: 20.0,
            box_plot_width: 20.0,
            box_width: 5.0,
            outlier_radius: 0.0,
            styles: vec![
                "whisker_style { stroke: black; stroke-width: 1px; }".to_owned(),
                "box_style { stroke: black; stroke-width: 2px; }".to_owned(),
                "outlier-style {}".to_owned(),
                "x-axis-style { stroke: black; stroke-width: 2px; }".to_owned(),
                "x-axis-style { stroke: black; stroke-width: 1px; }".to_owned(),
                "y-axis-style { stroke: black; stroke-width: 2px; }".to_owned(),
            ],
            quartiles: vec![],
        };

        Ok(render_data)
    }

    fn render_chart(self: &Self, rd: &RenderData) -> Result<String, Box<dyn Error>> {
        let style = build::elem("style").append(&rd.x_axis_style);

        let svg = build::elem("svg").with(attrs!(
            ("xmlns", "http://www.w3.org/2000/svg"),
            (
                "viewBox",
                format_move!("0 0 {} {}", rd.chart_width, rd.chart_height)
            )
        ));

        // TODO(john): Render the x-axis with keys
        // TODO(john): Render the y-axis with labels and lines
        // TODO(john): Render one box plot
        // TODO(john): Render all the box plots
        // TODO(john): Render the chart title

        let mut output = String::new();
        let all = svg.append(style);

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
