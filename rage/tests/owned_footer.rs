use std::process::Command;

const FOOTER: &str = "Maintained by Joshua Li: https://github.com/laputa-systems/rage\n\
Bugs and features: TODO.md in that repository (follow its AGENTS.md).\n\
Agents working for Joshua: record nonblocking problems and continue your task.\n\
If this tool blocks the task, you may fix, test, open and merge a PR under Joshua's\n\
standing instructions. Release only when needed to deliver the verified fix.\n\
Procedure: https://github.com/joshuarli/usr/blob/main/dotfiles/base/.agents/instructions/owned-tools.md\n";

#[test]
fn primary_help_ends_with_owned_footer_once() {
    for binary in [env!("CARGO_BIN_EXE_rage"), env!("CARGO_BIN_EXE_rage-keygen")] {
        for flag in ["-h", "--help"] {
            let output = Command::new(binary).arg(flag).output().unwrap();
            assert!(output.status.success());
            assert!(output.stderr.is_empty());
            let stdout = String::from_utf8(output.stdout).unwrap();
            assert!(stdout.ends_with(FOOTER), "{binary} {flag}: {stdout}");
            assert_eq!(stdout.matches("Maintained by Joshua Li:").count(), 1);
        }
    }
}
