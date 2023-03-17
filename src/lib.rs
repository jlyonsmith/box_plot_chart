mod log_macros;
pub mod quartile;

use clap::Parser;
use core::fmt::Arguments;
use easy_error::{self, ResultExt};
use hypermelon::{attr::PathCommand::*, build, prelude::*};
use quartile::Quartile;
use serde::Deserialize;
use std::{
    error::Error,
    fs::File,
    io::{self, Read, Write},
    path::PathBuf,
};

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
    /// The JSON5 input file
    #[clap(value_name = "INPUT_FILE")]
    input_file: Option<PathBuf>,

    /// The SVG output file
    #[clap(value_name = "OUTPUT_FILE")]
    output_file: Option<PathBuf>,
}

impl Cli {
    fn get_output(&self) -> Result<Box<dyn Write>, Box<dyn Error>> {
        match self.output_file {
            Some(ref path) => File::create(path)
                .context(format!(
                    "Unable to create file '{}'",
                    path.to_string_lossy()
                ))
                .map(|f| Box::new(f) as Box<dyn Write>)
                .map_err(|e| Box::new(e) as Box<dyn Error>),
            None => Ok(Box::new(io::stdout())),
        }
    }

    fn get_input(&self) -> Result<Box<dyn Read>, Box<dyn Error>> {
        match self.input_file {
            Some(ref path) => File::open(path)
                .context(format!("Unable to open file '{}'", path.to_string_lossy()))
                .map(|f| Box::new(f) as Box<dyn Read>)
                .map_err(|e| Box::new(e) as Box<dyn Error>),
            None => Ok(Box::new(io::stdin())),
        }
    }
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
struct Gutter {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

#[derive(Debug)]
struct RenderData {
    title: String,
    units: String,
    y_axis_height: f64,
    y_axis_range: (f64, f64),
    y_axis_interval: f64,
    y_axis_dps: usize,
    gutter: Gutter,
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

        let chart_data = Self::read_chart_file(cli.get_input()?)?;
        let render_data = self.process_chart_data(&chart_data)?;
        let output = self.render_chart(&render_data)?;

        Self::write_svg_file(cli.get_output()?, &output)?;

        Ok(())
    }

    fn read_chart_file(mut reader: Box<dyn Read>) -> Result<ChartData, Box<dyn Error>> {
        let mut content = String::new();

        reader.read_to_string(&mut content)?;

        let chart_data: ChartData = json5::from_str(&content)?;

        Ok(chart_data)
    }

    fn write_svg_file(mut writer: Box<dyn Write>, output: &str) -> Result<(), Box<dyn Error>> {
        write!(writer, "{}", output)?;

        Ok(())
    }

    fn process_chart_data(self: &Self, cd: &ChartData) -> Result<RenderData, Box<dyn Error>> {
        let mut quartile_tuples: Vec<(String, Quartile)> = vec![];
        let mut y_axis_range: (f64, f64) = (f64::MAX, f64::MIN);

        for item_data in cd.data.iter() {
            let quartile = Quartile::new(&item_data.values)?;
            let min_value = quartile.min_value();
            let max_value = quartile.max_value();

            if min_value < y_axis_range.0 {
                y_axis_range.0 = min_value;
            }

            if max_value > y_axis_range.1 {
                y_axis_range.1 = max_value;
            }

            quartile_tuples.push((item_data.key.to_owned(), quartile));
        }

        let y_axis_num_intervals = 20;
        let y_axis_interval = (10.0_f64).powf(((y_axis_range.1 - y_axis_range.0).log10()).ceil())
            / (y_axis_num_intervals as f64);
        let dps = y_axis_interval.log10();
        let y_axis_dps = if dps < 0.0 {
            dps.abs().ceil() as usize
        } else {
            0
        };

        y_axis_range = (
            f64::floor(y_axis_range.0 / y_axis_interval) * y_axis_interval,
            f64::ceil(y_axis_range.1 / y_axis_interval) * y_axis_interval,
        );

        let gutter = Gutter {
            top: 40.0,
            bottom: 80.0,
            left: 80.0,
            right: 80.0,
        };
        let y_axis_height = 400.0;
        let box_plot_width = 60.0;

        Ok(RenderData {
            title: cd.title.to_owned(),
            units: cd.units.to_owned(),
            y_axis_height,
            y_axis_range,
            y_axis_interval,
            y_axis_dps,
            gutter,
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
        let width = rd.gutter.left
            + ((rd.quartile_tuples.len() as f64) * rd.box_plot_width)
            + rd.gutter.right;
        let height = rd.gutter.top + rd.gutter.bottom + rd.y_axis_height;
        let y_range = ((rd.y_axis_range.1 - rd.y_axis_range.0) / rd.y_axis_interval) as usize;
        let y_scale = rd.y_axis_height / (rd.y_axis_range.1 - rd.y_axis_range.0);
        let scale =
            |n: &f64| -> f64 { height - rd.gutter.bottom - (n - rd.y_axis_range.0) * y_scale };

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
                (rd.gutter.left, rd.gutter.top),
                (rd.gutter.left, rd.gutter.top + rd.y_axis_height),
                (width - rd.gutter.right, rd.gutter.top + rd.y_axis_height),
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
                            rd.gutter.left
                                + (i as f64 * rd.box_plot_width)
                                + rd.box_plot_width / 2.0,
                            height - rd.gutter.bottom + 15.0
                        )
                    )))
                    .append(format_move!("{}", rd.quartile_tuples[i].0))
            })));

        let y_axis_labels =
            build::elem("g")
                .with(("class", "labels y-labels"))
                .append(build::from_iter((0..=y_range).map(|i| {
                    let n = i as f64 * rd.y_axis_interval;

                    build::elem("text")
                        .with(attrs!((
                            "transform",
                            format_move!(
                                "translate({},{})",
                                rd.gutter.left - 10.0,
                                height - rd.gutter.bottom - f64::floor(n * y_scale) + 5.0
                            )
                        )))
                        .append(format_move!(
                            "{0:.1$}",
                            n + rd.y_axis_range.0,
                            rd.y_axis_dps
                        ))
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
            let x = rd.gutter.left + rd.box_plot_width / 2.0 + (i as f64 * rd.box_plot_width);
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
                            height - rd.gutter.bottom - (v - rd.y_axis_range.0) * y_scale
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
                    M(x - half_whisker_width, y[0]),
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
                ("y", rd.gutter.top / 2.0)
            ))
            .append(format_move!("{} ({})", &rd.title, &rd.units));

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
