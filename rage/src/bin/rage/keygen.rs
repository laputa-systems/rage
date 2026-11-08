use age::{cli_common::file_io, secrecy::ExposeSecret};
use std::ffi::OsString;
use std::io::{self, Write};

#[path = "../rage-keygen/cli.rs"]
pub(crate) mod cli;

#[path = "../rage-keygen/error.rs"]
pub(crate) mod error;

pub(crate) enum RunError {
    Parse(cli::ParseError),
    Error(error::Error),
}

pub(crate) fn run(mut args: Vec<OsString>) -> Result<(), RunError> {
    if !args.is_empty() {
        args.remove(0);
    }
    let opts = match cli::AgeOptions::parse(args).map_err(RunError::Parse)? {
        cli::ParseOutcome::Help => {
            println!("{}", cli::help());
            return Ok(());
        }
        cli::ParseOutcome::Version => {
            println!("{}", cli::version());
            return Ok(());
        }
        cli::ParseOutcome::Options(opts) => opts,
    };

    let output = file_io::OutputWriter::new(
        opts.output,
        false,
        file_io::OutputFormat::Text,
        0o600,
        false,
    )
    .map_err(error::Error::FailedToOpenOutput)
    .map_err(RunError::Error)?;

    if opts.convert {
        convert(opts.input, output).map_err(RunError::Error)
    } else {
        generate(output).map_err(error::Error::FailedToWriteOutput).map_err(RunError::Error)
    }
}

fn generate(mut output: file_io::OutputWriter) -> io::Result<()> {
    let sk = age::x25519::Identity::generate();
    let pk = sk.to_public();

    writeln!(
        output,
        "# {}: {}",
        crate::fl!("identity-file-created"),
        chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    )?;
    writeln!(output, "# {}: {}", crate::fl!("identity-file-pubkey"), pk)?;
    writeln!(output, "{}", sk.to_string().expose_secret())?;

    if !output.is_terminal() {
        eprintln!("{}: {}", crate::fl!("tty-pubkey"), pk);
    }

    Ok(())
}

fn convert(filename: Option<String>, output: file_io::OutputWriter) -> Result<(), error::Error> {
    let file = age::IdentityFile::from_input_reader(
        file_io::InputReader::new(filename).map_err(error::Error::FailedToOpenInput)?,
    )
    .map_err(error::Error::FailedToReadInput)?;

    file.write_recipients_file(output)
        .map_err(error::Error::IdentityFileConvert)?;

    Ok(())
}
