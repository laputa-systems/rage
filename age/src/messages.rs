#[doc(hidden)]
#[macro_export]
macro_rules! fl {
        ("cli-passphrase-confirm") => { ::std::string::String::from("Confirm passphrase") };
        ("cli-passphrase-desc") => { ::std::string::String::from("Type passphrase (leave empty to autogenerate a secure one)") };
        ("cli-passphrase-prompt") => { ::std::string::String::from("Passphrase") };
        ("cli-secret-input-mismatch") => { ::std::string::String::from("Inputs do not match") };
        ("cli-secret-input-required") => { ::std::string::String::from("Input is required") };
        ("cli-truncated-tty") => { ::std::string::String::from("truncated; use a pipe, a redirect, or -o/--output to decrypt the entire file") };
        ("encrypted-passphrase-prompt", $($args:tt)*) => {
            ::std::format!("Type passphrase for encrypted identity '{filename}'", $($args)*)
        };
        ("encrypted-warn-no-match", $($args:tt)*) => {
            ::std::format!("Warning: encrypted identity file '{filename}' didn't match file's recipients", $($args)*)
        };
        ("err-decryption-failed") => { ::std::string::String::from("Decryption failed") };
        ("err-deny-binary-output") => { ::std::string::String::from("refusing to output binary to the terminal.") };
        ("err-deny-overwrite-file", $($args:tt)*) => {
            ::std::format!("refusing to overwrite existing file '{filename}'.", $($args)*)
        };
        ("err-detected-binary") => { ::std::string::String::from("detected unprintable data; refusing to output to the terminal.") };
        ("err-excessive-work") => { ::std::string::String::from("Excessive work parameter for passphrase.") };
        ("err-failed-to-write-output", $($args:tt)*) => {
            ::std::format!("Failed to write to output: {err}", $($args)*)
        };
        ("err-header-invalid") => { ::std::string::String::from("Header is invalid") };
        ("err-header-mac-invalid") => { ::std::string::String::from("Header MAC is invalid") };
        ("err-identity-file-contains-plugin", $($args:tt)*) => {
            ::std::format!("Identity file '{filename}' contains identities for 'age-plugin-{plugin_name}'.", $($args)*)
        };
        ("err-incompatible-recipients-oneway", $($args:tt)*) => {
            ::std::format!("Cannot encrypt to a recipient with labels '{labels}' alongside a recipient with no labels", $($args)*)
        };
        ("err-incompatible-recipients-twoway", $($args:tt)*) => {
            ::std::format!("Cannot encrypt to a recipient with labels '{left}' alongside a recipient with labels '{right}'", $($args)*)
        };
        ("err-invalid-filename", $($args:tt)*) => {
            ::std::format!("invalid filename '{filename}'.", $($args)*)
        };
        ("err-invalid-plugin-name", $($args:tt)*) => {
            ::std::format!("Invalid plugin name '{name}'.", $($args)*)
        };
        ("err-invalid-recipient-labels", $($args:tt)*) => {
            ::std::format!("The first recipient requires one or more invalid labels: '{labels}'", $($args)*)
        };
        ("err-key-decryption") => { ::std::string::String::from("Failed to decrypt an encrypted key") };
        ("err-missing-directory", $($args:tt)*) => {
            ::std::format!("directory '{path}' does not exist.", $($args)*)
        };
        ("err-missing-plugin", $($args:tt)*) => {
            ::std::format!("Could not find '{plugin_name}' on the PATH.", $($args)*)
        };
        ("err-missing-recipients") => { ::std::string::String::from("Missing recipients.") };
        ("err-mixed-recipient-passphrase") => { ::std::string::String::from("scrypt::Recipient can't be used with other recipients.") };
        ("err-no-identities-in-file", $($args:tt)*) => {
            ::std::format!("No identities found in file '{filename}'.", $($args)*)
        };
        ("err-no-identities-in-stdin") => { ::std::string::String::from("No identities found in standard input.") };
        ("err-no-matching-keys") => { ::std::string::String::from("No matching keys found") };
        ("err-plugin-died", $($args:tt)*) => {
            ::std::format!("'{plugin_name}' unexpectedly died.", $($args)*)
        };
        ("err-plugin-identity", $($args:tt)*) => {
            ::std::format!("'{plugin_name}' couldn't use an identity: {message}", $($args)*)
        };
        ("err-plugin-multiple") => { ::std::string::String::from("Plugin returned multiple errors:") };
        ("err-plugin-recipient", $($args:tt)*) => {
            ::std::format!("'{plugin_name}' couldn't use recipient {recipient}: {message}", $($args)*)
        };
        ("err-read-identity-encrypted-without-passphrase", $($args:tt)*) => {
            ::std::format!("Identity file '{filename}' is encrypted with age but not with a passphrase.", $($args)*)
        };
        ("err-read-identity-not-found", $($args:tt)*) => {
            ::std::format!("Identity file not found: {filename}", $($args)*)
        };
        ("err-read-invalid-recipient", $($args:tt)*) => {
            ::std::format!("Invalid recipient '{recipient}'.", $($args)*)
        };
        ("err-read-invalid-recipients-file", $($args:tt)*) => {
            ::std::format!("Recipients file '{filename}' contains non-recipient data on line {line_number}.", $($args)*)
        };
        ("err-read-missing-recipients-file", $($args:tt)*) => {
            ::std::format!("Recipients file not found: {filename}", $($args)*)
        };
        ("err-read-multiple-stdin") => { ::std::string::String::from("Standard input can't be used for multiple purposes.") };
        ("err-read-rsa-modulus-too-large", $($args:tt)*) => {
            ::std::format!("RSA Modulus Too Large\n---------------------\nOpenSSH supports various RSA modulus sizes, but rage only supports public\nkeys of at most {max_size} bits, to prevent a Denial of Service (DoS) condition\nwhen encrypting to untrusted public keys.", $($args)*)
        };
        ("err-read-rsa-modulus-too-small") => { ::std::string::String::from("RSA key size is too small.") };
        ("err-stream-last-chunk-empty") => { ::std::string::String::from("Last STREAM chunk is empty. Please report this, and/or try an older rage version.") };
        ("err-unknown-format") => { ::std::string::String::from("Unknown age format.") };
        ("plugin-waiting-on-binary", $($args:tt)*) => {
            ::std::format!("Waiting for {binary_name}...", $($args)*)
        };
        ("rec-deny-binary-output") => { ::std::string::String::from("Did you mean to use -a/--armor? Force with '-o -'.") };
        ("rec-detected-binary") => { ::std::string::String::from("Force with '-o -'.") };
        ("rec-excessive-work", $($args:tt)*) => {
            ::std::format!("Decryption would take around {duration} seconds.", $($args)*)
        };
        ("rec-identity-file-contains-plugin", $($args:tt)*) => {
            ::std::format!("Try using 'age-plugin-{plugin_name}' to convert this identity to a recipient.", $($args)*)
        };
        ("rec-missing-plugin") => { ::std::string::String::from("Have you installed the plugin?") };
        ("rec-plugin-died-1", $($args:tt)*) => {
            ::std::format!("If you are developing a plugin, run with {env_var} for more information.", $($args)*)
        };
        ("rec-plugin-died-2") => { ::std::string::String::from("Warning: this prints private encryption key material to standard error.") };
        ("rec-unknown-format") => { ::std::string::String::from("Have you tried upgrading to the latest version?") };
        ("ssh-insecure-key-format", $($args:tt)*) => {
            ::std::format!("Insecure Encrypted Key Format\n-----------------------------\nPrior to OpenSSH version 7.8, if a password was set when generating a new\nDSA, ECDSA, or RSA key, ssh-keygen would encrypt the key using the encrypted\nPEM format. This encryption format is insecure and should no longer be used.\n\nYou can migrate your key to the encrypted SSH private key format (which has\nbeen supported by OpenSSH since version 6.5, released in January 2014) by\nchanging its passphrase with the following command:\n\n    {change_passphrase}\n\nIf you are using an OpenSSH version between 6.5 and 7.7 (such as the default\nOpenSSH provided on Ubuntu 18.04 LTS), you can use the following command to\nforce keys to be generated using the new format:\n\n    {gen_new}", $($args)*)
        };
        ("ssh-passphrase-prompt", $($args:tt)*) => {
            ::std::format!("Type passphrase for OpenSSH key '{filename}'", $($args)*)
        };
        ("ssh-unsupported-cipher", $($args:tt)*) => {
            ::std::format!("Unsupported Cipher for Encrypted SSH Key\n----------------------------------------\nOpenSSH internally supports several different ciphers for encrypted keys,\nbut it has only ever directly generated a few of them. rage supports all\nciphers that ssh-keygen might generate, and is being updated on a\ncase-by-case basis with support for non-standard ciphers. Your key uses a\ncurrently-unsupported cipher ({cipher}).\n\nIf you would like support for this key type, please open an issue here:\n\n{new_issue}", $($args)*)
        };
        ("ssh-unsupported-key", $($args:tt)*) => {
            ::std::format!("Unsupported SSH key: {name}", $($args)*)
        };
        ("ssh-unsupported-key-type", $($args:tt)*) => {
            ::std::format!("Unsupported SSH Key Type\n------------------------\nOpenSSH supports various different key types, but rage only supports a\nsubset of these for backwards compatibility, specifically the 'ssh-rsa'\nand 'ssh-ed25519' key types. This SSH key uses the unsupported key type\n'{key_type}'.", $($args)*)
        };
        ("ssh-unsupported-security-key", $($args:tt)*) => {
            ::std::format!("Unsupported SSH Hardware Authenticator\n--------------------------------------\nOpenSSH version 8.2p1 added support for FIDO/U2F hardware authenticators,\nincluding hardware security keys such as YubiKeys. rage does not work with\nthese SSH key types, because their protocol does not support encryption.\nThis SSH key uses the incompatible type '{key_type}'.\n\nIf you have a compatible hardware security key, you should use this plugin:\n\n{age_plugin_yubikey_url}\n\nA hardware security key used with both OpenSSH and this plugin will have a\nseparate SSH public key and age encryption recipient, because the plugin\nimplements the PIV protocol.", $($args)*)
        };
}

/// age-localized version of the write! macro.
#[doc(hidden)]
#[macro_export]
macro_rules! wfl {
    ($f:ident, $message_id:tt) => {
        write!($f, "{}", $crate::fl!($message_id))
    };

    ($f:ident, $message_id:tt, $($args:tt)*) => {
        write!($f, "{}", $crate::fl!($message_id, $($args)*))
    };
}

/// age-localized version of the writeln! macro.
#[doc(hidden)]
#[macro_export]
macro_rules! wlnfl {
    ($f:ident, $message_id:tt) => {
        writeln!($f, "{}", $crate::fl!($message_id))
    };

    ($f:ident, $message_id:tt, $($args:tt)*) => {
        writeln!($f, "{}", $crate::fl!($message_id, $($args)*))
    };
}
