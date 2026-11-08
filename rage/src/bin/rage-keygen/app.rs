use std::ffi::OsString;
use std::process::ExitCode;

use crate::keygen;

pub(crate) fn run<I, T>(args: I) -> ExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    env_logger::builder()
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Off)
        .parse_default_env()
        .init();
    match keygen::run(args.into_iter().map(Into::into).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(keygen::RunError::Parse(error)) => {
            eprintln!("error: {error}");
            eprintln!("Usage: {}", keygen::cli::usage());
            ExitCode::from(2)
        }
        Err(keygen::RunError::Error(error)) => {
            eprintln!("Error: {error:?}");
            ExitCode::FAILURE
        }
    }
}

