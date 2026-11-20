use std::fs;
use std::io::Write;
use std::os::unix::fs::symlink;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::time::Duration;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join(".work/tmp");
        fs::create_dir_all(&root).unwrap();
        let path = root.join(format!("rage-plugin-{}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.0) {
            if std::thread::panicking() {
                eprintln!("failed to remove plugin fixture: {error}");
            } else {
                panic!("failed to remove plugin fixture: {error}");
            }
        }
    }
}

struct ProcessGroup(Option<u32>);

impl Drop for ProcessGroup {
    fn drop(&mut self) {
        if let Some(pid) = self.0 {
            let _ = Command::new("/bin/kill").args(["-KILL", "--", &format!("-{pid}")]).status();
        }
    }
}

fn run(mut command: Command, input: &[u8]) -> Output {
    let mut child = command.process_group(0).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let mut group = ProcessGroup(Some(child.id()));
    let input = input.to_vec();
    let (send, receive) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        let mut stdin = child.stdin.take().unwrap();
        let write_result = stdin.write_all(&input);
        drop(stdin);
        let result = match write_result {
            Ok(()) => child.wait_with_output(),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                Err(error)
            }
        };
        let _ = send.send(result);
    });
    match receive.recv_timeout(Duration::from_secs(30)) {
        Ok(Ok(output)) => {
            group.0 = None;
            worker.join().unwrap();
            output
        }
        Ok(Err(error)) => {
            drop(group);
            worker.join().unwrap();
            panic!("plugin command failed: {error}");
        }
        Err(error) => {
            drop(group);
            if receive.recv_timeout(Duration::from_secs(5)).is_ok() {
                worker.join().unwrap();
            }
            panic!("plugin command did not finish: {error}");
        }
    }
}

#[test]
fn example_plugin_encrypts_and_decrypts_through_rage() {
    let fixture = Fixture::new();
    let rage = PathBuf::from(env!("CARGO_BIN_EXE_rage"));
    let plugin = rage.parent().unwrap().join("examples/age-plugin-unencrypted");
    assert!(plugin.exists(), "build the example first: cargo build -p age-plugin --example age-plugin-unencrypted (make test does)");
    symlink(plugin, fixture.0.join("age-plugin-unencrypted")).unwrap();
    let key = fixture.0.join("identity");
    fs::write(&key, b"AGE-PLUGIN-UNENCRYPTED-1CR7ZD5\n").unwrap();
    let path = std::env::join_paths(std::iter::once(fixture.0.clone()).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()))).unwrap();
    let plaintext = b"plugin round trip";
    let mut encrypt = Command::new(&rage);
    encrypt.env("PATH", &path).env_remove("AGE_EXPLODES").env_remove("AGEDEBUG").args(["-r", "age1unencrypted1k5fr0r"]);
    let ciphertext = run(encrypt, plaintext);
    assert!(ciphertext.status.success(), "{}", String::from_utf8_lossy(&ciphertext.stderr));
    let mut decrypt = Command::new(&rage);
    decrypt.env("PATH", &path).env_remove("AGE_EXPLODES").env_remove("AGEDEBUG").args(["-d", "-i"]).arg(key);
    let decrypted = run(decrypt, &ciphertext.stdout);
    assert!(decrypted.status.success(), "{}", String::from_utf8_lossy(&decrypted.stderr));
    assert_eq!(decrypted.stdout, plaintext);
}
