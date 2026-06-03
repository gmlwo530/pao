pub mod ai_client;
pub mod approval;
pub mod change;
pub mod cli;
pub mod command;
pub mod command_log;
pub mod config;
pub mod error;
pub mod git;
pub mod task;
pub mod validation;
pub mod workspace;

#[cfg(test)]
mod test_support;

use std::ffi::OsString;
use std::path::PathBuf;

use clap::{error::ErrorKind, CommandFactory, Parser};

use crate::cli::Cli;
use crate::command::{execute, CommandReport};
use crate::error::{ErrorCode, PaoError};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone)]
pub struct RuntimeEnv {
    pub cwd: PathBuf,
    pub home: Option<PathBuf>,
    pub config_home: Option<PathBuf>,
}

impl RuntimeEnv {
    pub fn from_process() -> Result<Self, PaoError> {
        let cwd = std::env::current_dir().map_err(PaoError::io)?;
        let home = std::env::var_os("HOME").map(PathBuf::from);
        let config_home = std::env::var_os("PAO_CONFIG_HOME").map(PathBuf::from);

        Ok(Self {
            cwd,
            home,
            config_home,
        })
    }
}

pub async fn run<I>(args: I) -> Result<CommandReport, PaoError>
where
    I: IntoIterator<Item = OsString>,
{
    let runtime = RuntimeEnv::from_process()?;

    run_with_env(args, &runtime).await
}

pub async fn run_with_env<I>(args: I, runtime: &RuntimeEnv) -> Result<CommandReport, PaoError>
where
    I: IntoIterator<Item = OsString>,
{
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(error) => {
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) {
                let mut command = Cli::command();
                return Ok(CommandReport::stdout(
                    error.format(&mut command).to_string(),
                ));
            }

            return Err(PaoError::new(ErrorCode::CliUsage, error.to_string()));
        }
    };

    execute(cli, runtime).await
}
