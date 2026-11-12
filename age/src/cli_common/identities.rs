use std::io::{self, BufReader};

use super::{ReadError, StdinGuard, UiCallbacks};
use crate::{Identity, identity::IdentityFile};

#[cfg(feature = "armor")]
use crate::{armor::ArmoredReader, cli_common::file_io::InputReader};

#[cfg(feature = "ssh")]
use crate::util::LimitedReader;

#[cfg(feature = "ssh")]
const SSH_IDENTITY_SIZE_LIMIT: usize = 1 << 14; // 16 KiB

/// Reads identities from the provided files.
///
/// `filenames` may contain at most one entry of `"-"`, which will be interpreted as
/// reading from standard input. An error will be returned if `stdin_guard` is guarding an
/// existing usage of standard input.
///
/// Each file in `filenames` may be at most 16 MiB. SSH keys are limited to 16 kiB.
pub fn read_identities(
    filenames: Vec<String>,
    max_work_factor: Option<u8>,
    stdin_guard: &mut StdinGuard,
) -> Result<Vec<Box<dyn Identity>>, ReadError> {
    let mut identities: Vec<Box<dyn Identity>> = Vec::with_capacity(filenames.len());

    parse_identity_files::<_, ReadError>(
        filenames,
        max_work_factor,
        stdin_guard,
        &mut identities,
        #[cfg(feature = "armor")]
        |identities, identity| {
            identities.push(Box::new(identity));
            Ok(())
        },
        #[cfg(feature = "ssh")]
        |identities, _, identity| {
            identities.push(Box::new(identity.with_callbacks(UiCallbacks)));
            Ok(())
        },
        |identities, identity_file| {
            let new_identities = identity_file.into_identities();

            #[cfg(feature = "plugin")]
            let new_identities = new_identities.map_err(|e| match e {
                #[cfg(feature = "plugin")]
                crate::DecryptError::PluginResolve(e) => ReadError::PluginResolve(e),
                // DecryptError::PluginResolve is the only possible error kind returned by
                // IdentityFileEntry::into_identity.
                _ => unreachable!(),
            })?;

            // IdentityFileEntry::into_identity will never return a PluginResolve error
            // when plugin feature is not enabled.
            #[cfg(not(feature = "plugin"))]
            let new_identities = new_identities.unwrap();

            identities.extend(new_identities.into_iter().map(|i| i as _));

            Ok(())
        },
    )?;

    Ok(identities)
}

/// Parses the provided identity files.
pub(super) fn parse_identity_files<Ctx, E: From<ReadError> + From<io::Error>>(
    filenames: Vec<String>,
    _max_work_factor: Option<u8>,
    stdin_guard: &mut StdinGuard,
    ctx: &mut Ctx,
    #[cfg(feature = "armor")] encrypted_identity: impl Fn(
        &mut Ctx,
        crate::encrypted::Identity<ArmoredReader<BufReader<PeekableReader<InputReader>>>, UiCallbacks>,
    ) -> Result<(), E>,
    #[cfg(feature = "ssh")] ssh_identity: impl Fn(&mut Ctx, &str, crate::ssh::Identity) -> Result<(), E>,
    identity_file: impl Fn(&mut Ctx, crate::IdentityFile<UiCallbacks>) -> Result<(), E>,
) -> Result<(), E> {
    for filename in filenames {
        #[cfg_attr(not(any(feature = "armor", feature = "ssh")), allow(unused_mut))]
        let mut reader =
            PeekableReader::new(stdin_guard.open(filename.clone()).map_err(|e| match e {
                ReadError::Io(e) if matches!(e.kind(), io::ErrorKind::NotFound) => {
                    ReadError::IdentityNotFound(filename.clone())
                }
                _ => e,
            })?);

        #[cfg(feature = "armor")]
        // Try parsing as an encrypted age identity.
        if crate::encrypted::Identity::from_buffer(
            ArmoredReader::new_buffered(&mut reader),
            Some(filename.clone()),
            UiCallbacks,
            _max_work_factor,
        )
        .is_ok()
        {
            reader.reset()?;
            let identity = crate::encrypted::Identity::from_buffer(
                ArmoredReader::new_buffered(BufReader::new(reader)),
                Some(filename.clone()),
                UiCallbacks,
                _max_work_factor,
            )
            .expect("already parsed the age ciphertext header");

            encrypted_identity(
                ctx,
                identity.ok_or(ReadError::IdentityEncryptedWithoutPassphrase(filename))?,
            )?;
            continue;
        }

        #[cfg(feature = "armor")]
        reader.reset()?;

        // Try parsing as a single multi-line SSH identity.
        #[cfg(feature = "ssh")]
        match crate::ssh::Identity::from_buffer(
            LimitedReader::new(&mut reader, SSH_IDENTITY_SIZE_LIMIT),
            Some(filename.clone()),
        ) {
            Ok(crate::ssh::Identity::Unsupported(k)) => {
                return Err(ReadError::UnsupportedKey(filename, k).into());
            }
            Ok(identity) => {
                ssh_identity(ctx, &filename, identity)?;
                continue;
            }
            Err(_) => (),
        }

        #[cfg(feature = "ssh")]
        reader.reset()?;

        // Try parsing as multiple single-line age identities.
        identity_file(
            ctx,
            IdentityFile::from_buffer(reader)?.with_callbacks(UiCallbacks),
        )?;
    }

    Ok(())
}

const PEEKABLE_SIZE: usize = 8 * 1024;

enum PeekState {
    Peeking { consumed: usize },
    Reading,
}

pub(super) struct PeekableReader<R: io::Read> {
    inner: BufReader<R>,
    state: PeekState,
    replay: Vec<u8>,
}

impl<R: io::Read> PeekableReader<R> {
    fn new(inner: R) -> Self {
        Self {
            inner: BufReader::with_capacity(PEEKABLE_SIZE, inner),
            state: PeekState::Peeking { consumed: 0 },
            replay: Vec::new(),
        }
    }

    #[cfg(any(feature = "armor", feature = "ssh"))]
    fn reset(&mut self) -> io::Result<()> {
        match &mut self.state {
            PeekState::Peeking { consumed } => {
                *consumed = 0;
                Ok(())
            }
            PeekState::Reading => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Tried to reset after the underlying buffer was exceeded.",
            )),
        }
    }
}

impl<R: io::Read> io::Read for PeekableReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.state {
            PeekState::Peeking { .. } => {
                // Perform a read that will never exceed the size of the inner buffer.
                use std::io::BufRead;
                let nread = {
                    let mut rem = self.fill_buf()?;
                    rem.read(buf)?
                };
                self.consume(nread);
                Ok(nread)
            }
            PeekState::Reading => self.inner.read(buf),
        }
    }
}

impl<R: io::Read> io::BufRead for PeekableReader<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        match self.state {
            PeekState::Peeking { consumed } => {
                if consumed < self.replay.len() {
                    return Ok(&self.replay[consumed..]);
                }
                if self.replay.len() == PEEKABLE_SIZE {
                    self.state = PeekState::Reading;
                    self.replay.clear();
                    return self.inner.fill_buf();
                }
                let available = self.inner.fill_buf()?;
                let len = available.len().min(PEEKABLE_SIZE - self.replay.len());
                self.replay.extend_from_slice(&available[..len]);
                self.inner.consume(len);
                Ok(&self.replay[consumed..])
            }
            PeekState::Reading => self.inner.fill_buf(),
        }
    }

    fn consume(&mut self, amt: usize) {
        match &mut self.state {
            PeekState::Peeking { consumed, .. } => *consumed += amt,
            PeekState::Reading => self.inner.consume(amt),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, Read};

    struct Segmented<'a>(&'a [u8]);

    impl Read for Segmented<'_> {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            let len = output.len().min(3);
            self.0.read(&mut output[..len])
        }
    }

    #[test]
    fn short_reads_replay_until_the_peek_limit() {
        let input = b"# comment\nAGE-SECRET-KEY-1EXAMPLE\n";
        let mut reader = PeekableReader::new(Segmented(input));
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert_eq!(line, "# comment\n");
        #[cfg(any(feature = "armor", feature = "ssh"))]
        reader.reset().unwrap();
        #[cfg(not(any(feature = "armor", feature = "ssh")))]
        let mut reader = PeekableReader::new(Segmented(input));
        let mut replayed = Vec::new();
        reader.read_to_end(&mut replayed).unwrap();
        assert_eq!(replayed, input);
    }

    #[cfg(any(feature = "armor", feature = "ssh"))]
    #[test]
    fn reset_is_bounded_by_the_replay_limit() {
        for len in [0, PEEKABLE_SIZE - 1, PEEKABLE_SIZE, PEEKABLE_SIZE + 1] {
            let input = vec![b'x'; len];
            let mut reader = PeekableReader::new(Segmented(&input));
            let mut output = Vec::new();
            reader.read_to_end(&mut output).unwrap();
            assert_eq!(output, input);
            if len < PEEKABLE_SIZE {
                reader.reset().unwrap();
                output.clear();
                reader.read_to_end(&mut output).unwrap();
                assert_eq!(output, input);
            } else {
                assert_eq!(reader.reset().unwrap_err().kind(), io::ErrorKind::InvalidInput);
            }
        }
    }
}
