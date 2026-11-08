use std::collections::HashSet;
use std::ffi::OsString;
use std::fmt;

use lexopt::{Arg, Parser};

use crate::schema::OptionSpec;
use crate::fl;

pub(crate) fn options() -> Vec<OptionSpec> {
    crate::schema::keygen_options()
}

#[derive(Debug, Default)]
pub(crate) struct AgeOptions {
    pub(crate) input: Option<String>,
    pub(crate) output: Option<String>,
    pub(crate) convert: bool,
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
                    if !seen.insert(spec.id) {
                        return Err(ParseError(format!("option '-{short}' may only be used once")));
                    }
                    match spec.id {
                        "help" => return Ok(ParseOutcome::Help),
                        "version" => return Ok(ParseOutcome::Version),
                        "convert" => opts.convert = true,
                        "output" => {
                            let value = parser.value().map_err(|e| ParseError(e.to_string()))?;
                            opts.output = Some(to_utf8(value, "OUTPUT")?);
                        }
                        _ => unreachable!(),
                    }
                }
                Arg::Long(long) => {
                    let spec = specs.iter().find(|spec| spec.long == Some(long)).ok_or_else(|| ParseError(format!("unknown option '--{long}'")))?;
                    if !seen.insert(spec.id) {
                        return Err(ParseError(format!("option '--{}' may only be used once", spec.long.unwrap_or(spec.id))));
                    }
                    match spec.id {
                        "help" => return Ok(ParseOutcome::Help),
                        "version" => return Ok(ParseOutcome::Version),
                        "output" => {
                            let value = parser.value().map_err(|e| ParseError(e.to_string()))?;
                            opts.output = Some(to_utf8(value, "OUTPUT")?);
                        }
                        _ => unreachable!(),
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

fn to_utf8(value: OsString, name: &str) -> Result<String, ParseError> {
    value.into_string().map_err(|_| ParseError(format!("{name} must be valid UTF-8")))
}

pub(crate) fn help() -> String {
    let specs = options();
    let mut result = format!("Usage: {}\n\nArguments:\n  [{}]  {}\n\nOptions:\n", usage(), fl!("input"), fl!("help-arg-input"));
    let label_width = specs.iter().map(option_label).map(|label| label.len()).max().unwrap_or_default();
    for spec in specs {
        let label = option_label(&spec);
        result.push_str(&format!("  {label:label_width$}  {}\n", (spec.help)()));
    }
    result.pop();
    result
}

fn option_label(spec: &OptionSpec) -> String {
    spec.help_label()
}

pub(crate) fn usage() -> String {
    crate::schema::keygen_usage()
}

pub(crate) fn version() -> String {
    format!("rage-keygen {} (git {})", env!("CARGO_PKG_VERSION"), env!("RAGE_GIT_SHA"))
}

#[cfg(test)]
mod tests {
    use super::{AgeOptions, ParseOutcome, options};
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
    fn parses_file_values_clusters_and_long_equals() {
        let ParseOutcome::Options(opts) = parse(&["-y", "--output=key.txt", "--", "-identity"]).unwrap() else {
            panic!("expected parsed options")
        };
        assert!(opts.convert);
        assert_eq!(opts.output.as_deref(), Some("key.txt"));
        assert_eq!(opts.input.as_deref(), Some("-identity"));
    }

    #[test]
    fn version_spellings_unknown_flags_and_duplicates_are_checked() {
        assert!(matches!(parse(&["-v"]), Ok(ParseOutcome::Version)));
        assert!(matches!(parse(&["-V"]), Ok(ParseOutcome::Version)));
        assert!(matches!(parse(&["--version"]), Ok(ParseOutcome::Version)));
        assert!(parse(&["--unknown"]).unwrap_err().to_string().contains("--unknown"));
        assert!(parse(&["-o", "one", "-o", "two"]).unwrap_err().to_string().contains("-o"));
        assert!(parse(&["-yy"]).unwrap_err().to_string().contains("-y"));
    }

    #[test]
    fn help_and_usage_are_derived_from_the_schema() {
        assert!(super::help().contains("rage-keygen [OPTIONS] [INPUT]"));
        assert!(super::help().contains("-V, -v, --version"));
        assert!(super::help().contains("-y"));
    }
}
