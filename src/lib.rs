mod log_macros;
pub mod quartile;

use clap::Parser;
use core::fmt::Arguments;
use easy_error::{self, ResultExt};
use quartile::Quartile;
use serde::Deserialize;
use std::{
    error::Error,
    fs::File,
    io::{self, Read, Write},
    path::PathBuf,
};
use svg::{node::element::*, node::*, Document};

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
        let document = self.render_chart(&render_data)?;

        Self::write_svg_file(cli.get_output()?, &document)?;

        Ok(())
    }

    fn read_chart_file(mut reader: Box<dyn Read>) -> Result<ChartData, Box<dyn Error>> {
        let mut content = String::new();

        reader.read_to_string(&mut content)?;

        let chart_data: ChartData = json5::from_str(&content)?;

        Ok(chart_data)
    }

    fn write_svg_file(writer: Box<dyn Write>, document: &Document) -> Result<(), Box<dyn Error>> {
        svg::write(writer, document)?;

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

    fn render_chart(self: &Self, rd: &RenderData) -> Result<Document, Box<dyn Error>> {
        let width = rd.gutter.left
            + ((rd.quartile_tuples.len() as f64) * rd.box_plot_width)
            + rd.gutter.right;
        let height = rd.gutter.top + rd.gutter.bottom + rd.y_axis_height;
        let y_range = ((rd.y_axis_range.1 - rd.y_axis_range.0) / rd.y_axis_interval) as usize;
        let y_scale = rd.y_axis_height / (rd.y_axis_range.1 - rd.y_axis_range.0);
        let scale =
            |n: &f64| -> f64 { height - rd.gutter.bottom - (n - rd.y_axis_range.0) * y_scale };
        let mut document = Document::new()
            .set("xmlns", "http://www.w3.org/2000/svg")
            .set("width", width)
            .set("height", height)
            .set("viewBox", format!("0 0 {} {}", width, height))
            .set("style", "background-color: white;");
        let style = element::Style::new(rd.styles.join("\n"));
        let axis = element::Polyline::new().set("class", "axis").set(
            "points",
            vec![
                (rd.gutter.left, rd.gutter.top),
                (rd.gutter.left, rd.gutter.top + rd.y_axis_height),
                (width - rd.gutter.right, rd.gutter.top + rd.y_axis_height),
            ],
        );
        let mut x_axis_labels = element::Group::new().set("class", "labels");

        for i in 0..rd.quartile_tuples.len() {
            x_axis_labels.append(
                element::Text::new(format!("{}", rd.quartile_tuples[i].0)).set(
                    "transform",
                    format!(
                        "translate({},{}) rotate(45)",
                        rd.gutter.left + (i as f64 * rd.box_plot_width) + rd.box_plot_width / 2.0,
                        height - rd.gutter.bottom + 15.0
                    ),
                ),
            );
        }

        let mut y_axis_labels = element::Group::new().set("class", "labels y-labels");

        for i in 0..=y_range {
            let n = i as f64 * rd.y_axis_interval;

            y_axis_labels.append(
                element::Text::new(format!("{0:.1$}", n + rd.y_axis_range.0, rd.y_axis_dps)).set(
                    "transform",
                    format!(
                        "translate({},{})",
                        rd.gutter.left - 10.0,
                        height - rd.gutter.bottom - f64::floor(n * y_scale) + 5.0
                    ),
                ),
            );
        }

        let mut box_plots = element::Group::new();

        for i in 0..rd.quartile_tuples.len() {
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
            let y_outliers: Vec<f64> = quartile
                .upper_outliers()
                .into_iter()
                .chain(quartile.lower_outliers())
                .collect();
            let mut box_plot = element::Group::new().set("class", "box-plot");

            for outlier in y_outliers.iter() {
                box_plot.append(
                    element::Circle::new()
                        .set("class", "outliers")
                        .set("cx", x)
                        .set(
                            "cy",
                            height - rd.gutter.bottom - (outlier - rd.y_axis_range.0) * y_scale,
                        )
                        .set("r", rd.outlier_radius),
                )
            }

            box_plot.append(
                element::Path::new().set(
                    "d",
                    path::Data::new()
                        // Top whisker
                        .move_to((x - half_whisker_width, y[0]))
                        .line_by((whisker_width, 0.0))
                        .move_by((-half_whisker_width, 0.0))
                        .line_to((x, y[1]))
                        // Box
                        .move_to((x - half_box_width, y[2]))
                        .line_to((x - half_box_width, y[1]))
                        .line_by((box_width, 0.0))
                        .line_to((x + half_box_width, y[2]))
                        .line_by((-box_width, 0.0))
                        .line_to((x - half_box_width, y[3]))
                        .line_by((box_width, 0.0))
                        .line_to((x + half_box_width, y[2]))
                        // Lowel whisker
                        .move_to((x, y[3]))
                        .line_to((x, y[4]))
                        .line_by((-half_whisker_width, 0.0))
                        .line_by((whisker_width, 0.0)),
                ),
            );

            box_plots.append(box_plot);
        }

        let title = element::Text::new(format!("{} ({})", &rd.title, &rd.units))
            .set("class", "title")
            .set("x", width / 2.0)
            .set("y", rd.gutter.top / 2.0);

        document.append(style);
        document.append(axis);
        document.append(x_axis_labels);
        document.append(y_axis_labels);
        document.append(box_plots);
        document.append(title);

        Ok(document)
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
