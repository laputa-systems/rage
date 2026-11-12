use std::{fs, io::Write, path::PathBuf, process::{Command, Stdio}, sync::atomic::{AtomicU64, Ordering}};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let scratch = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join(".work/tmp");
        fs::create_dir_all(&scratch).unwrap();
        let path = scratch.join(format!("rage-io-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_rage"));
        command.current_dir(&self.0);
        command
    }

    fn key(&self) -> String {
        let result = self.command().args(["keygen", "-o", "key"]).output().unwrap();
        assert!(result.status.success(), "{result:?}");
        let result = self.command().args(["keygen", "-y", "key"]).output().unwrap();
        assert!(result.status.success(), "{result:?}");
        String::from_utf8(result.stdout).unwrap().trim().to_owned()
    }

    fn encrypt(&self, plaintext: &[u8]) {
        fs::write(self.0.join("input"), plaintext).unwrap();
        let recipient = self.key();
        let result = self.command().args(["-r", &recipient, "-o", "ciphertext", "input"]).output().unwrap();
        assert!(result.status.success(), "{result:?}");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn hard_link_outputs_preserve_encryption_and_decryption_inputs() {
    let fixture = Fixture::new();
    fixture.encrypt(b"source must survive");
    let recipient = fixture.command().args(["keygen", "-y", "key"]).output().unwrap();
    let recipient = String::from_utf8(recipient.stdout).unwrap();
    for (source, args) in [
        ("input", vec!["-r", recipient.trim(), "-o", "alias", "input"]),
        ("ciphertext", vec!["-d", "-i", "key", "-o", "alias", "ciphertext"]),
    ] {
        let original = fs::read(fixture.0.join(source)).unwrap();
        fs::hard_link(fixture.0.join(source), fixture.0.join("alias")).unwrap();
        let result = fixture.command().args(args).output().unwrap();
        assert!(!result.status.success(), "{result:?}");
        assert_eq!(fs::read(fixture.0.join(source)).unwrap(), original);
        assert_eq!(fs::read(fixture.0.join("alias")).unwrap(), original);
        fs::remove_file(fixture.0.join("alias")).unwrap();
    }
}

#[test]
fn empty_decryption_creates_and_truncates_output() {
    let fixture = Fixture::new();
    fixture.encrypt(b"");
    for existing in [false, true] {
        if existing {
            fs::write(fixture.0.join("output"), b"sentinel").unwrap();
        }
        let result = fixture.command().args(["-d", "-i", "key", "-o", "output", "ciphertext"]).output().unwrap();
        assert!(result.status.success(), "{result:?}");
        assert_eq!(fs::read(fixture.0.join("output")).unwrap(), b"");
    }
}

#[test]
fn invalid_ciphertext_preserves_lazy_output() {
    let fixture = Fixture::new();
    fixture.key();
    fs::write(fixture.0.join("ciphertext"), b"invalid").unwrap();
    for existing in [false, true] {
        if existing {
            fs::write(fixture.0.join("output"), b"sentinel").unwrap();
        }
        let result = fixture.command().args(["-d", "-i", "key", "-o", "output", "ciphertext"]).output().unwrap();
        assert!(!result.status.success());
        if existing {
            assert_eq!(fs::read(fixture.0.join("output")).unwrap(), b"sentinel");
        } else {
            assert!(!fixture.0.join("output").exists());
        }
    }
}

#[test]
fn stdin_identity_waits_for_more_than_a_comment() {
    let fixture = Fixture::new();
    fixture.encrypt(b"plaintext");
    let mut child = fixture.command().args(["-d", "-i", "-", "ciphertext"])
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(b"# comment\n").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));
    assert!(child.try_wait().unwrap().is_none());
    stdin.write_all(&fs::read(fixture.0.join("key")).unwrap()).unwrap();
    drop(stdin);
    let result = child.wait_with_output().unwrap();
    assert!(result.status.success(), "{result:?}");
    assert_eq!(result.stdout, b"plaintext");
}

#[cfg(target_os = "linux")]
#[test]
fn decryption_reports_final_stdout_flush_failure() {
    let fixture = Fixture::new();
    fixture.encrypt(b"no newline");
    let result = fixture.command().args(["-d", "-i", "key", "ciphertext"])
        .stdout(fs::OpenOptions::new().write(true).open("/dev/full").unwrap()).output().unwrap();
    assert!(!result.status.success());
    assert!(!result.stderr.is_empty());
}

#[cfg(unix)]
#[test]
fn output_aliases_created_after_setup_are_rejected() {
    use age::cli_common::file_io::{InputReader, OutputFormat, OutputWriter};
    for symlink in [false, true] {
        let fixture = Fixture::new();
        let source = fixture.0.join("input");
        let destination = fixture.0.join("output");
        fs::write(&source, b"preserve input").unwrap();
        fs::write(&destination, b"old output").unwrap();
        let input = InputReader::new(Some(source.to_str().unwrap().to_owned())).unwrap();
        let mut output = OutputWriter::new(
            Some(destination.to_str().unwrap().to_owned()), true, OutputFormat::Text, 0o600, false,
        ).unwrap();
        output.protect_input(&input).unwrap();
        fs::remove_file(&destination).unwrap();
        if symlink {
            std::os::unix::fs::symlink(&source, &destination).unwrap();
        } else {
            fs::hard_link(&source, &destination).unwrap();
        }
        assert_eq!(output.flush().unwrap_err().kind(), std::io::ErrorKind::InvalidInput);
        assert_eq!(fs::read(&source).unwrap(), b"preserve input");
        assert_eq!(fs::read(&destination).unwrap(), b"preserve input");
    }
}
