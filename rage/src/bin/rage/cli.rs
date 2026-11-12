use std::collections::HashSet;
use std::ffi::OsString;
use std::fmt;
use std::path::Path;

use lexopt::{Arg, Parser};

use crate::schema::OptionSpec;
use crate::fl;

pub(crate) fn options() -> Vec<OptionSpec> {
    crate::schema::rage_options()
}

#[derive(Debug, Default)]
pub(crate) struct AgeOptions {
    pub(crate) input: Option<String>,
    pub(crate) encrypt: bool,
    pub(crate) decrypt: bool,
    pub(crate) passphrase: bool,
    pub(crate) max_work_factor: Option<u8>,
    pub(crate) armor: bool,
    pub(crate) recipient: Vec<String>,
    pub(crate) recipients_file: Vec<String>,
    pub(crate) identity: Vec<String>,
    pub(crate) plugin_name: Option<String>,
    pub(crate) output: Option<String>,
}

#[derive(Debug)]
pub(crate) enum ParseOutcome {
    Help,
    Version,
    Options(AgeOptions),
}

#[derive(Debug)]
pub(crate) struct ParseError(String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ParseError {}

impl AgeOptions {
    pub(crate) fn parse<I, T>(args: I) -> Result<ParseOutcome, ParseError>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString>,
    {
        let mut parser = Parser::from_args(args);
        let mut opts = Self::default();
        let specs = options();
        let mut seen = HashSet::new();
        while let Some(arg) = parser.next().map_err(|e| ParseError(e.to_string()))? {
            match arg {
                Arg::Short(short) => {
                    let spec = specs.iter().find(|spec| spec.has_short(short)).ok_or_else(|| ParseError(format!("unknown option '-{short}'")))?;
                    if !spec.repeatable && !seen.insert(spec.id) {
                        return Err(ParseError(format!("option '-{short}' may only be used once")));
                    }
                    if let Some(value_name) = spec.value_name {
                        let value = parser.value().map_err(|e| ParseError(e.to_string()))?;
                        set_value(&mut opts, spec.id, value, value_name)?;
                    } else {
                        match spec.id {
                            "help" => return Ok(ParseOutcome::Help),
                            "version" => return Ok(ParseOutcome::Version),
                            "encrypt" => opts.encrypt = true,
                            "decrypt" => opts.decrypt = true,
                            "passphrase" => opts.passphrase = true,
                            "armor" => opts.armor = true,
                            _ => unreachable!(),
                        }
                    }
                }
                Arg::Long(long) => {
                    let spec = specs.iter().find(|spec| spec.long == Some(long)).ok_or_else(|| ParseError(format!("unknown option '--{long}'")))?;
                    if !spec.repeatable && !seen.insert(spec.id) {
                        return Err(ParseError(format!("option '--{}' may only be used once", spec.long.unwrap_or(spec.id))));
                    }
                    if let Some(value_name) = spec.value_name {
                        let value = parser.value().map_err(|e| ParseError(e.to_string()))?;
                        set_value(&mut opts, spec.id, value, value_name)?;
                    } else {
                        match spec.id {
                            "help" => return Ok(ParseOutcome::Help),
                            "version" => return Ok(ParseOutcome::Version),
                            "encrypt" => opts.encrypt = true,
                            "decrypt" => opts.decrypt = true,
                            "passphrase" => opts.passphrase = true,
                            "armor" => opts.armor = true,
                            _ => unreachable!(),
                        }
                    }
                }
                Arg::Value(value) => {
                    if opts.input.is_some() {
                        return Err(ParseError("unexpected positional argument".into()));
                    }
                    opts.input = Some(to_utf8(value, "INPUT")?);
                }
            }
        }
        Ok(ParseOutcome::Options(opts))
    }
}

fn set_value(opts: &mut AgeOptions, id: &str, value: OsString, value_name: &str) -> Result<(), ParseError> {
    let value = to_utf8(value, value_name)?;
    match id {
        "max-work-factor" => {
            opts.max_work_factor = Some(value.parse().map_err(|_| ParseError(format!("invalid value for '--max-work-factor': {value}")))?);
        }
        "recipient" => opts.recipient.push(value),
        "recipients-file" => opts.recipients_file.push(value),
        "identity" => opts.identity.push(value),
        "plugin-name" => opts.plugin_name = Some(value),
        "output" => opts.output = Some(value),
        _ => unreachable!(),
    }
    Ok(())
}

fn to_utf8(value: OsString, name: &str) -> Result<String, ParseError> {
    value.into_string().map_err(|_| ParseError(format!("{name} must be valid UTF-8")))
}

pub(crate) fn binary_name(suffix: Option<&str>) -> String {
    if let Some(arg) = std::env::args_os().next() {
        let path = Path::new(&arg);
        if let Some(suffix) = suffix {
            let stem = path.file_stem().expect("is not directory").to_string_lossy();
            let extension = path.extension().map(|extension| format!(".{}", extension.to_string_lossy())).unwrap_or_default();
            format!("{stem}{suffix}{extension}")
        } else {
            path.file_name().expect("is not directory").to_string_lossy().to_string()
        }
    } else {
        format!("rage{}", suffix.unwrap_or_default())
    }
}

pub(crate) fn usage() -> String {
    crate::schema::rage_usage_named(&binary_name(None))
}

pub(crate) fn help() -> String {
    let program = binary_name(None);
    let keygen_name = binary_name(Some("-keygen"));
    let mut result = format!("Usage: {}\n\nArguments:\n  [{}]  {}\n\nOptions:\n", usage(), fl!("input"), fl!("help-arg-input"));
    let specs = options();
    let label_width = specs.iter().map(option_label).map(|label| label.len()).max().unwrap_or_default();
    for spec in specs {
        let label = option_label(&spec);
        result.push_str(&format!("  {label:label_width$}  {}\n", (spec.help)()));
    }
    result.push_str(&format!("\nCommand:\n  {program} keygen [OPTIONS] [INPUT]\n    {}\n    Inherits --help and --version.\n", fl!("man-keygen-about")));
    for spec in crate::schema::keygen_options().into_iter().filter(|spec| spec.id != "help" && spec.id != "version") {
        let label = option_label(&spec);
        result.push_str(&format!("    {label:label_width$}  {}\n", (spec.help)()));
    }
    let example_a = format!("$ {keygen_name} -o key.txt");
    let example_a_output = "age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p";
    let example_b = format!("$ tar cvz ~/data | {program} -r {example_a_output} > data.tar.gz.age");
    let example_c = format!("$ {program} -d -i key.txt -o data.tar.gz data.tar.gz.age");
    result.push_str(&format!("\n{}\n\n{}", after_help_content(&keygen_name), fl!("rage-after-help-example", example_a = example_a, example_a_output = example_a_output, example_b = example_b, example_c = example_c)));
    result
}

fn option_label(spec: &OptionSpec) -> String {
    spec.help_label()
}

pub(crate) fn version() -> String {
    format!("{} {} (git {})", binary_name(None), env!("CARGO_PKG_VERSION"), env!("RAGE_GIT_SHA"))
}

pub(crate) fn after_help_content(keygen_name: &str) -> String {
    crate::schema::rage_after_help_content(keygen_name)
}

#[cfg(test)]
mod tests {
    use super::{AgeOptions, OptionSpec, ParseOutcome, options};
    use std::collections::HashSet;

    fn parse(args: &[&str]) -> Result<ParseOutcome, super::ParseError> {
        AgeOptions::parse(args.iter().copied())
    }

    #[test]
    fn schema_spellings_are_unique_and_described() {
        let specs = options();
        let mut short = HashSet::new();
        let mut long = HashSet::new();
        for spec in &specs {
            assert!(!(spec.help)().is_empty());
            assert!(spec.short.into_iter().chain(spec.short_aliases.iter().copied()).all(|value| short.insert(value)));
            if let Some(value) = spec.long {
                assert!(long.insert(value));
            }
        }
    }

    #[test]
    fn parses_clusters_equals_repeated_values_and_opaque_positionals() {
        let ParseOutcome::Options(opts) = parse(&["-ea", "--max-work-factor=12", "--recipient=first", "-r", "second", "--", "-input"]).unwrap() else {
            panic!("expected parsed options")
        };
        assert!(opts.encrypt && opts.armor);
        assert_eq!(opts.max_work_factor, Some(12));
        assert_eq!(opts.recipient, ["first", "second"]);
        assert_eq!(opts.input.as_deref(), Some("-input"));
    }

    #[test]
    fn supports_all_version_spellings_and_reports_parse_errors() {
        assert!(matches!(parse(&["-v"]), Ok(ParseOutcome::Version)));
        assert!(matches!(parse(&["-V"]), Ok(ParseOutcome::Version)));
        assert!(matches!(parse(&["--version"]), Ok(ParseOutcome::Version)));
        assert!(parse(&["--unknown"]).unwrap_err().to_string().contains("--unknown"));
        assert!(parse(&["--encrypt", "--encrypt"]).unwrap_err().to_string().contains("encrypt"));
    }

    #[test]
    fn rejects_non_utf8_values_at_string_consumers() {
        use std::os::unix::ffi::OsStringExt;
        let args = [std::ffi::OsString::from("--output"), std::ffi::OsString::from_vec(vec![0xff])];
        assert!(AgeOptions::parse(args).unwrap_err().to_string().contains("OUTPUT must be valid UTF-8"));
    }

    #[test]
    fn metadata_spelling_formatter_includes_all_aliases() {
        let version: OptionSpec = options().into_iter().find(|spec| spec.id == "version").unwrap();
        assert_eq!(version.help_label(), "-V, -v, --version");
        assert!(super::help().contains("--max-work-factor <WF>"));
        assert!(super::help().contains("-j <PLUGIN-NAME>"));
    }
}
