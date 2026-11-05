use age::{cli_common::file_io, secrecy::ExposeSecret};
use clap::Parser;
use std::ffi::OsString;
use std::io::{self, Write};

#[path = "../rage-keygen/cli.rs"]
mod cli;

#[path = "../rage-keygen/error.rs"]
pub(crate) mod error;

pub(crate) fn run(args: Vec<OsString>) -> Result<(), error::Error> {
    let _ = env_logger::builder()
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Off)
        .parse_default_env()
        .try_init();

    let opts = cli::AgeOptions::parse_from(args);

    let output = file_io::OutputWriter::new(
        opts.output,
        false,
        file_io::OutputFormat::Text,
        0o600,
        false,
    )
    .map_err(error::Error::FailedToOpenOutput)?;

    if opts.convert {
        convert(opts.input, output)
    } else {
        generate(output).map_err(error::Error::FailedToWriteOutput)
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
