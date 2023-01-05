use colored::Colorize;
use core::fmt::Arguments;
use box_plot_chart::{error, BoxPlotChartLog, BoxPlotChartTool};

struct BoxPlotChartLogger;

impl BoxPlotChartLogger {
    fn new() -> BoxPlotChartLogger {
        BoxPlotChartLogger {}
    }
}

impl BoxPlotChartLog for BoxPlotChartLogger {
    fn output(self: &Self, args: Arguments) {
        println!("{}", args);
    }
    fn warning(self: &Self, args: Arguments) {
        eprintln!("{}", format!("warning: {}", args).yellow());
    }
    fn error(self: &Self, args: Arguments) {
        eprintln!("{}", format!("error: {}", args).red());
    }
}

fn main() {
    let logger = BoxPlotChartLogger::new();

    if let Err(error) = BoxPlotChartTool::new(&logger).run(std::env::args_os()) {
        error!(logger, "{}", error);
        std::process::exit(1);
    }
}
