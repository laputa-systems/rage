use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use flate2::{Compression, write::GzEncoder};

#[path = "src/bin/rage/messages.rs"]
mod messages;
#[allow(dead_code)]
mod schema {
    include!("src/bin/shared/cli_schema.rs");
}
fn git_sha() -> String {
    println!("cargo:rerun-if-env-changed=RAGE_GIT_SHA");
    let sha = match env::var("RAGE_GIT_SHA") {
        Ok(value) if !value.is_empty() => value,
        _ => {
            let git_dir = Command::new("git")
                .args(["rev-parse", "--git-dir"])
                .output()
                .expect("git is required to build without RAGE_GIT_SHA");
            assert!(git_dir.status.success(), "set RAGE_GIT_SHA when building from a source archive");
            let git_dir = String::from_utf8(git_dir.stdout).expect("git directory must be UTF-8");
            let git_dir = PathBuf::from(git_dir.trim());
            println!("cargo:rerun-if-changed={}", git_dir.join("HEAD").display());
            let head_ref = Command::new("git")
                .args(["symbolic-ref", "--quiet", "HEAD"])
                .output()
                .expect("git must be available");
            if head_ref.status.success() {
                let head_ref = String::from_utf8(head_ref.stdout).expect("git ref must be UTF-8");
                let ref_path = Command::new("git")
                    .args(["rev-parse", "--git-path", head_ref.trim()])
                    .output()
                    .expect("git must be available");
                assert!(ref_path.status.success(), "git ref path must be available");
                let ref_path = String::from_utf8(ref_path.stdout).expect("git ref path must be UTF-8");
                println!("cargo:rerun-if-changed={}", ref_path.trim());
            }
            let output = Command::new("git")
                .args(["rev-parse", "--short=12", "HEAD"])
                .output()
                .expect("git must be available");
            assert!(output.status.success(), "set RAGE_GIT_SHA when building from a source archive");
            String::from_utf8(output.stdout).expect("git SHA must be UTF-8").trim().to_owned()
        }
    };
    assert!(!sha.is_empty() && sha != "unknown" && sha != "dirty");
    assert!(sha.bytes().all(|byte| byte.is_ascii_hexdigit()), "RAGE_GIT_SHA must be hexadecimal");
    sha
}

fn main() -> io::Result<()> {
    let sha = git_sha();
    println!("cargo:rustc-env=RAGE_GIT_SHA={sha}");
    let Some(out_dir) = env::var_os("OUT_DIR") else {
        return Ok(());
    };
    let target_dir = PathBuf::from(out_dir)
        .ancestors()
        .nth(3)
        .expect("OUT_DIR must be an absolute Cargo target directory")
        .to_path_buf();
    generate_completions(&target_dir.join("completions"))?;
    generate_manpages(&target_dir.join("manpages/man1"), &sha)
}

fn generate_completions(out_dir: &Path) -> io::Result<()> {
    fs::create_dir_all(out_dir)?;
    generate_completion_set(out_dir, "rage", &schema::rage_options())?;
    generate_completion_set(out_dir, "rage-keygen", &schema::keygen_options())?;
    Ok(())
}

fn generate_completion_set(out_dir: &Path, name: &str, options: &[schema::OptionSpec]) -> io::Result<()> {
    write(out_dir.join(format!("{name}.bash")), bash_completion(name, options))?;
    Ok(())
}

fn bash_completion(name: &str, options: &[schema::OptionSpec]) -> String {
    let candidates = options.iter().flat_map(|option| {
        option.labels()
    }).collect::<Vec<_>>().join(" ");
    let file_options = options.iter().filter(|option| option.complete_files).collect::<Vec<_>>();
    let file_flags = file_options.iter().flat_map(|option| option.labels()).collect::<Vec<_>>().join("|");
    let file_equals = file_options.iter().filter_map(|option| option.long.map(|long| format!("--{long}=*"))).collect::<Vec<_>>().join("|");
    let value_flags = options.iter().filter(|option| option.value_name.is_some() && !option.complete_files).flat_map(|option| option.labels()).collect::<Vec<_>>().join("|");
    let value_case = if value_flags.is_empty() { String::new() } else { format!("    {value_flags}) COMPREPLY=(); return ;;\n") };
    format!("_{name}() {{\n  local cur=\"${{COMP_WORDS[COMP_CWORD]}}\"\n  local value prefix candidate\n  case \"${{COMP_WORDS[COMP_CWORD-1]}}\" in\n    {file_flags}) COMPREPLY=(); while IFS= read -r candidate; do COMPREPLY+=(\"$candidate\"); done < <(compgen -f -- \"$cur\"); return ;;\n{value_case}  esac\n  case \"$cur\" in\n    {file_equals}) prefix=\"${{cur%%=*}}=\"; value=\"${{cur#*=}}\"; COMPREPLY=(); while IFS= read -r candidate; do COMPREPLY+=(\"$prefix$candidate\"); done < <(compgen -f -- \"$value\"); return ;;\n    -*) COMPREPLY=( $(compgen -W '{candidates}' -- \"$cur\") ); return ;;\n  esac\n  COMPREPLY=(); while IFS= read -r candidate; do COMPREPLY+=(\"$candidate\"); done < <(compgen -f -- \"$cur\")\n}}\ncomplete -F _{name} {name}\n")
}

fn generate_manpages(out_dir: &Path, sha: &str) -> io::Result<()> {
    fs::create_dir_all(out_dir)?;
    let pages = [
        ("rage", schema::rage_options().iter().map(|option| (option.help_label(), option.long_help.unwrap_or(option.help)())).collect::<Vec<_>>(), fl!("man-rage-about"), fl!("man-rage-description"), rage_details(), rage_examples()),
        ("rage-keygen", schema::keygen_options().iter().map(|option| (option.help_label(), option.long_help.unwrap_or(option.help)())).collect::<Vec<_>>(), fl!("man-keygen-about"), fl!("man-keygen-description"), String::new(), keygen_examples()),
    ];
    for (name, options, about, description, details, examples) in pages {
        let synopsis = if name == "rage" { schema::rage_usage_named("rage") } else { schema::keygen_usage() };
        let mut text = format!(".TH {} 1\n.SH NAME\n{} \\- {}\n.SH SYNOPSIS\n.B {}\n{}\n.SH DESCRIPTION\n{}\n.SH OPTIONS\n", name.to_uppercase(), name, roff(&about), name, roff(&synopsis), roff(&description));
        for (spelling, help) in options {
            text.push_str(&format!(".TP\n.B {}\n{}\n", roff(&spelling), roff(&help)));
        }
        if !details.is_empty() {
            text.push_str(&details);
        }
        text.push_str(&examples);
        text.push_str(&format!(".SH SEE ALSO\n{}\n.SH VERSION\n{} {} (git {})\n.SH AUTHORS\n{}\n", if name == "rage" { "rage-keygen(1)" } else { "rage(1)" }, name, env!("CARGO_PKG_VERSION"), sha, env!("CARGO_PKG_AUTHORS")));
        let file = fs::File::create(out_dir.join(format!("{name}.1.gz")))?;
        let mut encoder = GzEncoder::new(file, Compression::best());
        encoder.write_all(text.as_bytes())?;
        encoder.finish()?;
    }
    Ok(())
}

fn rage_details() -> String {
    format!(".SH RECIPIENTS AND IDENTITIES\n{}\n.SS {}\n{}\n.SS {}\n{}\n.SS {}\n{}\n.PP\n{}\n", roff(&fl!("man-rage-recipients-and-identities")), fl!("man-rage-native-x25519-keys-heading"), roff(&fl!("man-rage-native-x25519-keys", example_age_recipient = "age1gde3ncmahlqd9gg50tanl99r960llztrhfapnmx853s4tjum03uqfssgdh", example_age_identity = "AGE-SECRET-KEY-1KTYK6RVLN5TAPE7VF6FQQSKZ9HWWCDSKUGXXNUQDWZ7XXT5YK5LSF3UTKQ")), fl!("man-rage-ssh-keys-heading"), roff(&fl!("man-rage-ssh-keys", example_ssh_rsa = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDULTit0KUehbi[...]GU4BtElAbzh8=", example_ssh_ed25519 = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIH9pO5pz22JZEas[...]l1uZc31FGYMXa")), fl!("man-rage-plugins-heading"), roff(&fl!("man-rage-plugins")), roff(&schema::rage_after_help_content("rage-keygen")))
}

fn rage_examples() -> String {
    render_examples(vec![
        (fl!("man-rage-example-single"), vec![("rage-keygen -o key.txt".into(), vec![format!("{}: age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p", fl!("tty-pubkey"))]), ("tar cvz ~/data | rage -r age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p > data.tar.gz.age".into(), vec![]), ("rage -d -o data.tar.gz -i key.txt data.tar.gz.age".into(), vec![])]),
        (fl!("man-rage-example-enc-multiple", input = "example.jpg", output = "example.jpg.age"), vec![("rage -o example.jpg.age -r age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p -r age1lggyhqrw2nlhcxprm67z43rta597azn8gknawjehu9d9dl0jq3yqqvfafg example.jpg".into(), vec![])]),
        (fl!("man-rage-example-enc-list"), vec![("cat > recipients.txt".into(), vec!["# Alice".into(), "age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p".into(), "# Bob".into(), "age1lggyhqrw2nlhcxprm67z43rta597azn8gknawjehu9d9dl0jq3yqqvfafg".into()]), ("rage -R recipients.txt example.jpg > example.jpg.age".into(), vec![])]),
        (fl!("man-rage-example-password"), vec![("rage -p secrets.txt > secrets.txt.age".into(), vec![fl!("autogenerated-passphrase"), "    release-response-step-brand-wrap-ankle-pair-unusual-sword-train".into()]), ("rage -d secrets.txt.age > secrets.txt".into(), vec![format!("{}:", fl!("type-passphrase"))])]),
        (fl!("man-rage-example-identity-passphrase"), vec![("rage -p <(rage-keygen) > key.age".into(), vec![format!("{}: age1yhm4gctwfmrpz87tdslm550wrx6m79y9f2hdzt0lndjnehwj0ukqrjpyx5", fl!("tty-pubkey")), fl!("autogenerated-passphrase"), "    hip-roast-boring-snake-mention-east-wasp-honey-input-actress".into()]), ("rage -r age1yhm4gctwfmrpz87tdslm550wrx6m79y9f2hdzt0lndjnehwj0ukqrjpyx5 secrets.txt > secrets.txt.age".into(), vec![]), ("rage -d -i key.age secrets.txt.age > secrets.txt".into(), vec![format!("{}:", fl!("type-passphrase"))])]),
        (fl!("man-rage-example-ssh"), vec![("rage -R ~/.ssh/id_ed25519.pub example.jpg > example.jpg.age".into(), vec![]), ("rage -d -i ~/.ssh/id_ed25519 example.jpg.age > example.jpg".into(), vec![])]),
        (fl!("man-rage-example-yubikey"), vec![("age-plugin-yubikey".into(), vec![format!("# {}", fl!("man-rage-example-yubikey-setup"))]), ("rage -r age1yubikey1qwt50d05nh5vutpdzmlg5wn80xq5negm4uj9ghv0snvdd3yysf5yw3rhl3t secrets.txt > secrets.txt.age".into(), vec![]), ("rage -d -i age-yubikey-identity-388178f3.txt secrets.txt.age".into(), vec![])]),
        (fl!("man-rage-example-enc-github"), vec![("curl https://github.com/benjojo.keys | rage -R - example.jpg > example.jpg.age".into(), vec![])]),
    ])
}

fn keygen_examples() -> String {
    render_examples(vec![
        (fl!("man-keygen-example-stdout"), vec![("rage-keygen".into(), vec![format!("# {}: 2021-01-02T15:30:45+01:00", fl!("identity-file-created")), format!("# {}: age1lvyvwawkr0mcnnnncaghunadrqkmuf9e6507x9y920xxpp866cnql7dp2z", fl!("identity-file-pubkey")), "AGE-SECRET-KEY-1N9JEPW6DWJ0ZQUDX63F5A03GX8QUW7PXDE39N8UYF82VZ9PC8UFS3M7XA9".into()])]),
        (fl!("man-keygen-example-file", filename = "key.txt"), vec![("rage-keygen -o key.txt".into(), vec![format!("{}: age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p", fl!("tty-pubkey"))])]),
        (fl!("man-keygen-example-convert"), vec![("rage-keygen -y key.txt".into(), vec!["age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p".into()])]),
    ])
}

/// Example groups: a heading and its commands, each with its description lines.
type Examples = Vec<(String, Vec<(String, Vec<String>)>)>;

fn render_examples(examples: Examples) -> String {
    let mut output = format!(".SH {}\n", fl!("man-examples-heading"));
    for (heading, commands) in examples {
        output.push_str(&format!(".TP\n.B {}\n", roff(&heading)));
        for (command, lines) in commands {
            output.push_str(&format!(".nf\n.B $ {}\n", roff(&command)));
            for line in lines {
                output.push_str(&format!("{}\n", roff(&line)));
            }
            output.push_str(".fi\n");
        }
    }
    output
}

fn roff(value: &str) -> String {
    value.replace('\\', "\\\\").replace('-', "\\-").replace('\n', "\n.PP\n")
}

fn write(path: PathBuf, text: String) -> io::Result<()> {
    fs::write(path, text)
}
