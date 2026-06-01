use std::io::{self, Write};
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    match pao::run(std::env::args_os()).await {
        Ok(report) => {
            print!("{}", report.stdout);
            ExitCode::SUCCESS
        }
        Err(error) => {
            let _ = writeln!(io::stderr(), "{}", error.render());
            ExitCode::from(error.exit_code())
        }
    }
}
