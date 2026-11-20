use std::io::{self, Write};

struct FailFirstWrite(bool);

impl Write for FailFirstWrite {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.0 {
            self.0 = false;
            Err(io::Error::new(io::ErrorKind::StorageFull, "output full"))
        } else {
            Ok(bytes.len())
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn stanza_writer_returns_the_first_output_error() {
    let mut output = FailFirstWrite(true);
    let error = age_core::format::write::age_stanza(&mut output, "tag", &[] as &[&str], b"body")
        .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::StorageFull);
}
