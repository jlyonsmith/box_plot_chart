mod log_macros;
mod quartile;

use clap::Parser;
use core::fmt::Arguments;
use hypermelon::{attr::PathCommand::*, build, prelude::*};
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

#[derive(Debug)]
struct RenderData {
    title: String,
    units: String,
    y_axis_height: f64,
    y_axis_range: (f64, f64),
    y_axis_ticks: f64,
    left_gutter: f64,
    bottom_gutter: f64,
    top_gutter: f64,
    box_plot_width: f64,
    outlier_radius: f64,
    styles: Vec<String>,
    quartile_tuples: Vec<(String, Quartile)>,
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

        //println!("{:?}", &render_data);

        let output = self.render_chart(&render_data)?;

        let mut file = fs::File::create(cli.output_file)?;

        file.write_all(output.as_bytes())?;

        Ok(())
    }

    fn read_chart_file(chart_file: &PathBuf) -> Result<ChartData, Box<dyn Error>> {
        let content = fs::read_to_string(chart_file)?;
        let chart_data: ChartData = json5::from_str(&content)?;

        Ok(chart_data)
    }

    fn process_chart_data(self: &Self, cd: &ChartData) -> Result<RenderData, Box<dyn Error>> {
        let mut quartile_tuples: Vec<(String, Quartile)> = vec![];
        let mut y_axis_range: (f64, f64) = (f64::MAX, f64::MIN);
        let y_axis_ticks = 10.0;

        for item_data in cd.data.iter() {
            let quartile = Quartile::new(&item_data.values)?;
            let min_value = quartile.min_value();
            let max_value = quartile.max_value();

            if min_value < y_axis_range.0 {
                y_axis_range.0 = f64::floor(min_value / y_axis_ticks) * y_axis_ticks;
            }

            if max_value > y_axis_range.1 {
                y_axis_range.1 = f64::ceil(max_value / y_axis_ticks) * y_axis_ticks;
            }

            quartile_tuples.push((item_data.key.to_owned(), quartile));
        }

        let top_gutter = 40.0;
        let bottom_gutter = 80.0;
        let left_gutter = 40.0;
        let y_axis_height = 400.0;
        let box_plot_width = 60.0;

        Ok(RenderData {
            title: cd.title.to_owned(),
            units: cd.units.to_owned(),
            y_axis_height,
            y_axis_range,
            y_axis_ticks,
            top_gutter,
            bottom_gutter,
            left_gutter,
            box_plot_width,
            outlier_radius: 2.0,
            styles: vec![
                ".box-plot{fill:none;stroke:rgb(0,0,0);stroke-width:1;}".to_owned(),
                ".outlier{fill:none;stroke:rgb(0,0,0);stroke-width:1;}".to_owned(),
                ".axis{fill:none;stroke:rgb(0,0,0);stroke-width:1;}".to_owned(),
                ".labels{fill:rgb(0,0,0);font-size:10;font-family:Arial}".to_owned(),
                ".y-labels{text-anchor:end;}".to_owned(),
                ".title{font-family:Arial;font-size:12;text-anchor:middle;}".to_owned(),
            ],
            quartile_tuples,
        })
    }

    fn render_chart(self: &Self, rd: &RenderData) -> Result<String, Box<dyn Error>> {
        let width = rd.left_gutter + ((rd.quartile_tuples.len() as f64) * rd.box_plot_width);
        let height = rd.top_gutter + rd.bottom_gutter + rd.y_axis_height;
        let y_range = ((rd.y_axis_range.1 - rd.y_axis_range.0) / rd.y_axis_ticks) as usize;
        let y_scale = rd.y_axis_height / (rd.y_axis_range.1 - rd.y_axis_range.0);
        let scale =
            |n: &f64| -> f64 { height - rd.bottom_gutter - (n - rd.y_axis_range.0) * y_scale };

        let style = build::elem("style").append(build::from_iter(rd.styles.iter()));

        let svg = build::elem("svg").with(attrs!(
            ("xmlns", "http://www.w3.org/2000/svg"),
            ("width", width),
            ("height", height),
            ("viewBox", format_move!("0 0 {} {}", width, height)),
            ("style", "background-color: white;")
        ));

        let axis = build::single("polyline").with(attrs!(
            ("class", "axis"),
            build::points([
                (rd.left_gutter, rd.top_gutter),
                (rd.left_gutter, rd.top_gutter + rd.y_axis_height),
                (width, rd.top_gutter + rd.y_axis_height),
            ])
        ));
        let x_axis_labels = build::elem("g")
            .with(("class", "labels"))
            .append(build::from_iter((0..rd.quartile_tuples.len()).map(|i| {
                build::elem("text")
                    .with(attrs!((
                        "transform",
                        format_move!(
                            "translate({},{}) rotate(45)",
                            rd.left_gutter
                                + (i as f64 * rd.box_plot_width)
                                + rd.box_plot_width / 2.0,
                            height - rd.bottom_gutter + 15.0
                        )
                    )))
                    .append(format_move!("{}", rd.quartile_tuples[i].0))
            })));

        let y_axis_labels =
            build::elem("g")
                .with(("class", "labels y-labels"))
                .append(build::from_iter((0..=y_range).map(|i| {
                    let n = i as f64 * rd.y_axis_ticks;

                    build::elem("text")
                        .with(attrs!((
                            "transform",
                            format_move!(
                                "translate({},{})",
                                rd.left_gutter - 10.0,
                                height - rd.bottom_gutter - f64::floor(n * y_scale) + 5.0
                            )
                        )))
                        .append(format_move!("{}", n + rd.y_axis_range.0))
                })));

        let box_plots = build::from_iter((0..rd.quartile_tuples.len()).map(|i| {
            let quartile = &rd.quartile_tuples[i].1;
            let box_width = rd.box_plot_width / 3.0;
            let half_box_width = box_width / 2.0;
            let whisker_width = rd.box_plot_width / 4.0;
            let half_whisker_width = whisker_width / 2.0;

            let y = vec![
                quartile.max_before_upper_fence(),
                quartile.upper_median(),
                quartile.median(),
                quartile.lower_median(),
                quartile.min_before_lower_fence(),
            ]
            .iter()
            .map(scale)
            .collect::<Vec<f64>>();
            let x = rd.left_gutter + rd.box_plot_width / 2.0 + (i as f64 * rd.box_plot_width);
            let outliers = build::from_closure(move |w| {
                let y_outliers: Vec<f64> = quartile
                    .upper_outliers()
                    .into_iter()
                    .chain(quartile.lower_outliers())
                    .collect();

                w.render(build::from_iter(y_outliers.iter().map(|&v| {
                    build::single("circle").with(attrs!(
                        ("class", "outliers"),
                        ("cx", x),
                        (
                            "cy",
                            height - rd.bottom_gutter - (v - rd.y_axis_range.0) * y_scale
                        ),
                        ("r", rd.outlier_radius)
                    ))
                })))
            });

            build::elem("g")
                .with(attrs!(("class", "box-plot")))
                .append(outliers)
                .append(build::single("path").with(build::path([
                    // Top whisker
                    M(x, y[0]),
                    M_(-half_whisker_width, 0.0),
                    L_(whisker_width, 0.0),
                    M_(-half_whisker_width, 0.0),
                    L(x, y[1]),
                    // Box
                    M(x - half_box_width, y[2]),
                    L(x - half_box_width, y[1]),
                    L_(box_width, 0.0),
                    L(x + half_box_width, y[2]),
                    L_(-box_width, 0.0),
                    L(x - half_box_width, y[3]),
                    L_(box_width, 0.0),
                    L(x + half_box_width, y[2]),
                    // Lowel whisker
                    M(x, y[3]),
                    L(x, y[4]),
                    M_(-half_whisker_width, 0.0),
                    L_(whisker_width, 0.0),
                ])))
        }));

        let title = build::elem("text")
            .with(attrs!(
                ("class", "title"),
                ("x", width / 2.0),
                ("y", rd.top_gutter / 2.0)
            ))
            .append(format_move!("{} ({})", &rd.title, &rd.units));

        // TODO(john): Render the chart title

        let mut output = String::new();
        let all = svg
            .append(style)
            .append(axis)
            .append(x_axis_labels)
            .append(y_axis_labels)
            .append(box_plots)
            .append(title);

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
