use box_plot_chart::{error, BoxPlotChartLog, BoxPlotChartTool};
use core::fmt::Arguments;
use yansi::Paint;

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
        eprintln!("{}", Paint::yellow(&format!("warning: {}", args)));
    }
    fn error(self: &Self, args: Arguments) {
        eprintln!("{}", Paint::red(&format!("error: {}", args)));
    }
}

fn main() {
    let logger = BoxPlotChartLogger::new();

    if let Err(error) = BoxPlotChartTool::new(&logger).run(std::env::args_os()) {
        error!(logger, "{}", error);
        std::process::exit(1);
    }
}
