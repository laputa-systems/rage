#[derive(Clone, Copy)]
pub(crate) struct OptionSpec {
    pub(crate) id: &'static str,
    pub(crate) short: Option<char>,
    pub(crate) short_aliases: &'static [char],
    pub(crate) long: Option<&'static str>,
    pub(crate) value_name: Option<&'static str>,
    pub(crate) help: fn() -> String,
    pub(crate) long_help: Option<fn() -> String>,
    pub(crate) repeatable: bool,
    pub(crate) complete_files: bool,
}

impl OptionSpec {
    pub(crate) fn has_short(&self, short: char) -> bool {
        self.short == Some(short) || self.short_aliases.contains(&short)
    }

    pub(crate) fn labels(&self) -> Vec<String> {
        self.short
            .into_iter()
            .chain(self.short_aliases.iter().copied())
            .map(|short| format!("-{short}"))
            .chain(self.long.map(|long| format!("--{long}")))
            .collect()
    }

    pub(crate) fn help_label(&self) -> String {
        let mut label = self.labels().join(", ");
        if self.short.is_none() {
            label.insert_str(0, "    ");
        }
        if let Some(value_name) = self.value_name {
            label.push_str(&format!(" <{value_name}>"));
        }
        label
    }
}

pub(crate) fn rage_options() -> Vec<OptionSpec> {
    vec![
        OptionSpec { id: "help", short: Some('h'), short_aliases: &[], long: Some("help"), value_name: None, help: || crate::fl!("help-flag-help"), long_help: None, repeatable: false, complete_files: false },
        OptionSpec { id: "version", short: Some('V'), short_aliases: &['v'], long: Some("version"), value_name: None, help: || crate::fl!("help-flag-version"), long_help: None, repeatable: false, complete_files: false },
        OptionSpec { id: "encrypt", short: Some('e'), short_aliases: &[], long: Some("encrypt"), value_name: None, help: || crate::fl!("help-flag-encrypt"), long_help: Some(|| crate::fl!("man-rage-flag-encrypt")), repeatable: false, complete_files: false },
        OptionSpec { id: "decrypt", short: Some('d'), short_aliases: &[], long: Some("decrypt"), value_name: None, help: || crate::fl!("help-flag-decrypt"), long_help: Some(|| crate::fl!("man-rage-flag-decrypt")), repeatable: false, complete_files: false },
        OptionSpec { id: "passphrase", short: Some('p'), short_aliases: &[], long: Some("passphrase"), value_name: None, help: || crate::fl!("help-flag-passphrase"), long_help: Some(|| crate::fl!("man-rage-flag-passphrase")), repeatable: false, complete_files: false },
        OptionSpec { id: "max-work-factor", short: None, short_aliases: &[], long: Some("max-work-factor"), value_name: Some("WF"), help: || crate::fl!("help-flag-max-work-factor"), long_help: None, repeatable: false, complete_files: false },
        OptionSpec { id: "armor", short: Some('a'), short_aliases: &[], long: Some("armor"), value_name: None, help: || crate::fl!("help-flag-armor"), long_help: Some(|| crate::fl!("man-rage-flag-armor")), repeatable: false, complete_files: false },
        OptionSpec { id: "recipient", short: Some('r'), short_aliases: &[], long: Some("recipient"), value_name: Some("RECIPIENT"), help: || crate::fl!("help-flag-recipient"), long_help: Some(|| crate::fl!("man-rage-flag-recipient")), repeatable: true, complete_files: false },
        OptionSpec { id: "recipients-file", short: Some('R'), short_aliases: &[], long: Some("recipients-file"), value_name: Some("PATH"), help: || crate::fl!("help-flag-recipients-file"), long_help: Some(|| crate::fl!("man-rage-flag-recipients-file")), repeatable: true, complete_files: true },
        OptionSpec { id: "identity", short: Some('i'), short_aliases: &[], long: Some("identity"), value_name: Some("IDENTITY"), help: || crate::fl!("help-flag-identity"), long_help: Some(|| crate::fl!("man-rage-flag-identity-decrypt")), repeatable: true, complete_files: true },
        OptionSpec { id: "plugin-name", short: Some('j'), short_aliases: &[], long: None, value_name: Some("PLUGIN-NAME"), help: || crate::fl!("help-flag-plugin-name"), long_help: Some(|| crate::fl!("man-rage-flag-plugin-decrypt")), repeatable: false, complete_files: false },
        OptionSpec { id: "output", short: Some('o'), short_aliases: &[], long: Some("output"), value_name: Some("OUTPUT"), help: || crate::fl!("help-flag-output"), long_help: Some(|| crate::fl!("man-rage-flag-output")), repeatable: false, complete_files: true },
    ]
}

pub(crate) fn keygen_options() -> Vec<OptionSpec> {
    vec![
        OptionSpec { id: "help", short: Some('h'), short_aliases: &[], long: Some("help"), value_name: None, help: || crate::fl!("help-flag-help"), long_help: None, repeatable: false, complete_files: false },
        OptionSpec { id: "version", short: Some('V'), short_aliases: &['v'], long: Some("version"), value_name: None, help: || crate::fl!("help-flag-version"), long_help: None, repeatable: false, complete_files: false },
        OptionSpec { id: "output", short: Some('o'), short_aliases: &[], long: Some("output"), value_name: Some("OUTPUT"), help: || crate::fl!("keygen-help-flag-output"), long_help: Some(|| crate::fl!("man-keygen-flag-output")), repeatable: false, complete_files: true },
        OptionSpec { id: "convert", short: Some('y'), short_aliases: &[], long: None, value_name: None, help: || crate::fl!("keygen-help-flag-convert"), long_help: Some(|| crate::fl!("man-keygen-flag-convert")), repeatable: false, complete_files: false },
    ]
}

pub(crate) fn rage_usage_named(binary_name: &str) -> String {
    let specs = rage_options();
    let short = |id: &str| format!("-{}", specs.iter().find(|spec| spec.id == id).expect("usage option must exist").short.expect("usage option must have a short spelling"));
    let long = |id: &str| format!("--{}", specs.iter().find(|spec| spec.id == id).and_then(|spec| spec.long).expect("usage option must have a long spelling"));
    let value = |id: &str| specs.iter().find(|spec| spec.id == id).and_then(|spec| spec.value_name).expect("usage option must have a value name").to_owned();
    let input = crate::fl!("input");
    let identity = crate::fl!("identity");
    let output = crate::fl!("output");
    let recipients = short("recipient");
    let recipients_value = value("recipient");
    let recipient_files = short("recipients-file");
    let recipient_files_value = value("recipients-file");
    let identities = short("identity");
    let armor = short("armor");
    let output_flag = short("output");
    let encrypt = long("encrypt");
    let decrypt = long("decrypt");
    let passphrase = long("passphrase");
    format!("{binary_name} [{encrypt}] ({recipients} {recipients_value} | {recipient_files} {recipient_files_value})... [{identities} {identity}] [{armor}] [{output_flag} {output}] [{input}]\n       {binary_name} [{encrypt}] {passphrase} [{armor}] [{output_flag} {output}] [{input}]\n       {binary_name} {decrypt} [{identities} {identity}] [{output_flag} {output}] [{input}]")
}

pub(crate) fn keygen_usage() -> String {
    format!("rage-keygen [OPTIONS] [{}]", crate::fl!("input"))
}

pub(crate) fn rage_after_help_content(keygen_name: &str) -> String {
    crate::fl!("rage-after-help-content", keygen_name = keygen_name, example_age_pubkey = "\"age1...\"", example_ssh_pubkey = "\"ssh-ed25519 AAAA...\", \"ssh-rsa AAAA...\"")
}
