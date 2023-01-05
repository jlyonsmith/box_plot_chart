mod log_macros;
mod quartile;

use clap::Parser;
use core::fmt::Arguments;
use quartile::Quartile;
use std::error::Error;
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

impl<'a> BoxPlotChartTool<'a> {
    pub fn new(log: &'a dyn BoxPlotChartLog) -> BoxPlotChartTool {
        BoxPlotChartTool { log }
    }

    pub fn run(
        self: &mut Self,
        args: impl IntoIterator<Item = std::ffi::OsString>,
    ) -> Result<(), Box<dyn Error>> {
        let _cli = match Cli::try_parse_from(args) {
            Ok(m) => m,
            Err(err) => {
                output!(self.log, "{}", err.to_string());
                return Ok(());
            }
        };

        let quartile = Quartile::new(&[
            5.0, 6.0, 48.0, 52.0, 57.0, 61.0, 64.0, 72.0, 76.0, 77.0, 81.0, 85.0, 88.0,
        ]);

        println!("{:?}", quartile);

        Ok(())
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
