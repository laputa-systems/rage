//! The age file format.

use std::io::{self, BufRead, Read, Write};

use age_core::format::{Stanza, grease_the_joint};

use crate::{
    EncryptError,
    error::DecryptError,
    primitives::{HmacKey, HmacWriter},
    scrypt,
};

#[cfg(feature = "async")]
use futures::io::{
    AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt,
};

const AGE_MAGIC: &[u8] = b"age-encryption.org/";
const V1_MAGIC: &[u8] = b"v1";
const MAC_TAG: &[u8] = b"---";
const ENCODED_MAC_LENGTH: usize = 43;

#[derive(Debug, PartialEq)]
pub(crate) struct HeaderV1 {
    pub(crate) recipients: Vec<Stanza>,
    pub(crate) mac: [u8; 32],
    /// The serialized bytes of this header. Set if we parsed this header from a reader,
    /// to handle the possibility that the header is not round-trip canonical, such as it
    /// containing a legacy stanza with a body of length 0 mod 64.
    ///
    /// We do not write this back out in `Header::write`, because we never write headers
    /// that we have parsed, and headers we generate for writing will never have this set.
    encoded_bytes: Option<Vec<u8>>,
}

impl HeaderV1 {
    pub(crate) fn new(recipients: Vec<Stanza>, mac_key: HmacKey) -> Result<Self, EncryptError> {
        let mut header = HeaderV1 {
            recipients,
            mac: [0; 32],
            encoded_bytes: None,
        };

        if header.no_scrypt() {
            // Keep the joint well oiled!
            header.recipients.push(grease_the_joint());
        }

        if !header.is_valid() {
            return Err(EncryptError::MixedRecipientAndPassphrase);
        }

        let mut mac = HmacWriter::new(mac_key);
        write::header_v1_minus_mac(&mut mac, &header)
            .expect("can serialize Header into HmacWriter");
        header
            .mac
            .copy_from_slice(mac.finalize().into_bytes().as_slice());

        Ok(header)
    }

    pub(crate) fn verify_mac(&self, mac_key: HmacKey) -> Result<(), hmac::digest::MacError> {
        let mut mac = HmacWriter::new(mac_key);
        if let Some(bytes) = &self.encoded_bytes {
            // The MAC covers all bytes up to the end of the --- prefix.
            mac.write_all(&bytes[..bytes.len() - ENCODED_MAC_LENGTH - 2])
                .expect("can serialize Header into HmacWriter");
        } else {
            write::header_v1_minus_mac(&mut mac, self)
                .expect("can serialize Header into HmacWriter");
        }
        mac.verify(&self.mac)
    }

    fn any_scrypt(&self) -> bool {
        self.recipients
            .iter()
            .any(|r| r.tag == scrypt::SCRYPT_RECIPIENT_TAG)
    }

    /// Checks whether the header contains a single recipient of type `scrypt`.
    ///
    /// This can be used along with [`Self::no_scrypt`] to enforce the structural
    /// requirements on the v1 header.
    pub(crate) fn valid_scrypt(&self) -> bool {
        self.any_scrypt() && self.recipients.len() == 1
    }

    /// Checks whether the header contains no `scrypt` recipients.
    ///
    /// This can be used along with [`Self::valid_scrypt`] to enforce the structural
    /// requirements on the v1 header.
    pub(crate) fn no_scrypt(&self) -> bool {
        !self.any_scrypt()
    }

    /// Enforces structural requirements on the v1 header.
    pub(crate) fn is_valid(&self) -> bool {
        self.valid_scrypt() || self.no_scrypt()
    }
}

impl Header {
    pub(crate) fn read<R: Read>(mut input: R) -> Result<Self, DecryptError> {
        let mut data = vec![];
        loop {
            match read::header(&data) {
                Ok((_, mut header)) => {
                    if let Header::V1(h) = &mut header {
                        h.encoded_bytes = Some(data);
                    }
                    break Ok(header);
                }
                Err(nom::Err::Incomplete(nom::Needed::Size(n))) => {
                    // Read the needed additional bytes. We need to be careful how the
                    // parser is constructed, because if we read more than we need, the
                    // remainder of the input will be truncated.
                    let m = data.len();
                    let new_len = m + n.get();
                    data.resize(new_len, 0);
                    input.read_exact(&mut data[m..new_len])?;
                }
                Err(_) => {
                    break Err(DecryptError::InvalidHeader);
                }
            }
        }
    }

    pub(crate) fn read_buffered<R: BufRead>(mut input: R) -> Result<Self, DecryptError> {
        let mut data = vec![];
        loop {
            match read::header(&data) {
                Ok((_, mut header)) => {
                    if let Header::V1(h) = &mut header {
                        h.encoded_bytes = Some(data);
                    }
                    break Ok(header);
                }
                Err(nom::Err::Incomplete(nom::Needed::Size(_))) => {
                    // As we have a buffered reader, we can leverage the fact that the
                    // currently-defined header formats are newline-separated, to more
                    // efficiently read data for the parser to consume.
                    if input.read_until(b'\n', &mut data)? == 0 {
                        break Err(DecryptError::Io(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "Incomplete header",
                        )));
                    }
                }
                Err(_) => {
                    break Err(DecryptError::InvalidHeader);
                }
            }
        }
    }

    #[cfg(feature = "async")]
    #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
    pub(crate) async fn read_async<R: AsyncRead + Unpin>(
        mut input: R,
    ) -> Result<Self, DecryptError> {
        let mut data = vec![];
        loop {
            match read::header(&data) {
                Ok((_, mut header)) => {
                    if let Header::V1(h) = &mut header {
                        h.encoded_bytes = Some(data);
                    }
                    break Ok(header);
                }
                Err(nom::Err::Incomplete(nom::Needed::Size(n))) => {
                    // Read the needed additional bytes. We need to be careful how the
                    // parser is constructed, because if we read more than we need, the
                    // remainder of the input will be truncated.
                    let m = data.len();
                    let new_len = m + n.get();
                    data.resize(new_len, 0);
                    input.read_exact(&mut data[m..new_len]).await?;
                }
                Err(_) => {
                    break Err(DecryptError::InvalidHeader);
                }
            }
        }
    }

    #[cfg(feature = "async")]
    #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
    pub(crate) async fn read_async_buffered<R: AsyncBufRead + Unpin>(
        mut input: R,
    ) -> Result<Self, DecryptError> {
        let mut data = vec![];
        loop {
            match read::header(&data) {
                Ok((_, mut header)) => {
                    if let Header::V1(h) = &mut header {
                        h.encoded_bytes = Some(data);
                    }
                    break Ok(header);
                }
                Err(nom::Err::Incomplete(nom::Needed::Size(_))) => {
                    // As we have a buffered reader, we can leverage the fact that the
                    // currently-defined header formats are newline-separated, to more
                    // efficiently read data for the parser to consume.
                    if input.read_until(b'\n', &mut data).await? == 0 {
                        break Err(DecryptError::Io(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "Incomplete header",
                        )));
                    }
                }
                Err(_) => {
                    break Err(DecryptError::InvalidHeader);
                }
            }
        }
    }

    pub(crate) fn write<W: Write>(&self, mut output: W) -> io::Result<()> {
        write::header(&mut output, self)
    }

    #[cfg(feature = "async")]
    #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
    pub(crate) async fn write_async<W: AsyncWrite + Unpin>(&self, mut output: W) -> io::Result<()> {
        let mut buf = vec![];
        write::header(&mut buf, self)?;

        output.write_all(&buf).await
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Header {
    V1(HeaderV1),
    Unknown(String),
}

mod read {
    use age_core::format::read::{arbitrary_string, legacy_age_stanza};
    use nom::{
        IResult, Parser,
        branch::alt,
        bytes::streaming::{tag, take},
        character::streaming::newline,
        combinator::{map, map_opt},
        multi::many1,
        sequence::{pair, preceded, terminated},
    };

    use super::*;
    use crate::util::read::base64_arg;

    fn header_v1(input: &[u8]) -> IResult<&[u8], HeaderV1> {
        preceded(
            pair(tag(V1_MAGIC), newline),
            map(
                pair(
                    many1(legacy_age_stanza),
                    preceded(
                        pair(tag(MAC_TAG), tag(" ")),
                        terminated(
                            map_opt(take(ENCODED_MAC_LENGTH), |tag| {
                                base64_arg::<_, 32, 33>(&tag)
                            }),
                            newline,
                        ),
                    ),
                ),
                |(recipients, mac)| HeaderV1 {
                    recipients: recipients.into_iter().map(Stanza::from).collect(),
                    mac,
                    encoded_bytes: None,
                },
            ),
        )
        .parse(input)
    }

    /// From the age specification:
    /// ```text
    /// The first line of the header is age-encryption.org/ followed by an arbitrary
    /// version string. ... We describe version v1, other versions can change anything
    /// after the first line.
    /// ```
    pub(super) fn header(input: &[u8]) -> IResult<&[u8], Header> {
        preceded(
            tag(AGE_MAGIC),
            alt((
                map(header_v1, Header::V1),
                map(terminated(arbitrary_string, newline), |s| {
                    Header::Unknown(s.to_string())
                }),
            )),
        )
        .parse(input)
    }
}

mod write {
    use age_core::format::write::age_stanza;
    use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
    use std::io::{self, Write};

    use super::*;

    fn recipient_stanza<W: Write>(w: &mut W, r: &Stanza) -> io::Result<()> {
        let args: Vec<_> = r.args.iter().map(|s| s.as_str()).collect();
        age_stanza(w, &r.tag, &args, &r.body)
    }

    pub(super) fn header_v1_minus_mac<W: Write>(w: &mut W, h: &HeaderV1) -> io::Result<()> {
        w.write_all(AGE_MAGIC)?;
        w.write_all(V1_MAGIC)?;
        w.write_all(b"\n")?;
        for r in &h.recipients {
            recipient_stanza(w, r)?;
        }
        w.write_all(MAC_TAG)
    }

    fn header_v1<W: Write>(w: &mut W, h: &HeaderV1) -> io::Result<()> {
        header_v1_minus_mac(w, h)?;
        w.write_all(b" ")?;
        w.write_all(BASE64_STANDARD_NO_PAD.encode(h.mac).as_bytes())?;
        w.write_all(b"\n")
    }

    pub(super) fn header<W: Write>(w: &mut W, h: &Header) -> io::Result<()> {
        match h {
            Header::V1(v1) => header_v1(w, v1),
            Header::Unknown(version) => {
                w.write_all(AGE_MAGIC)?;
                w.write_all(version.as_bytes())?;
                w.write_all(b"\n")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Header;

    #[test]
    fn parse_header() {
        let test_header = "age-encryption.org/v1
-> X25519 CJM36AHmTbdHSuOQL+NESqyVQE75f2e610iRdLPEN20
C3ZAeY64NXS4QFrksLm3EGz+uPRyI0eQsWw7LWbbYig
-> X25519 ytazqsbmUnPwVWMVx0c1X9iUtGdY4yAB08UQTY2hNCI
N3pgrXkbIn/RrVt0T0G3sQr1wGWuclqKxTSWHSqGdkc
-> scrypt bBjlhJVYZeE4aqUdmtRHfw 15
ZV/AhotwSGqaPCU43cepl4WYUouAa17a3xpu4G2yi5k
-> ssh-rsa mhir0Q
xD7o4VEOu1t7KZQ1gDgq2FPzBEeSRqbnqvQEXdLRYy143BxR6oFxsUUJCRB0ErXA
mgmZq7tIm5ZyY89OmqZztOgG2tEB1TZvX3Q8oXESBuFjBBQkKaMLkaqh5GjcGRrZ
e5MmTXRdEyNPRl8qpystNZR1q2rEDUHSEJInVLW8OtvQRG8P303VpjnOUU53FSBw
yXxDtzxKxeloceFubn/HWGcR0mHU+1e9l39myQEUZjIoqFIELXvh9o6RUgYzaAI+
m/uPLMQdlIkiOOdbsrE6tFesRLZNHAYspeRKI9MJ++Xg9i7rutU34ZM+1BL6KgZf
J9FSm+GFHiVWpr1MfYCo/w
-> ssh-ed25519 BjH7FA RO+wV4kbbl4NtSmp56lQcfRdRp3dEFpdQmWkaoiw6lY
51eEu5Oo2JYAG7OU4oamH03FDRP18/GnzeCrY7Z+sa8
-> some-empty-body-recipient BjH7FA 37 mhir0Q

-> some-full-body-recipient BjH7FA 37 mhir0Q
xD7o4VEOu1t7KZQ1gDgq2FPzBEeSRqbnqvQEXdLRYy143BxR6oFxsUUJCRB0ErXA

-> some-other-recipient mhir0Q BjH7FA 37
m/uPLMQdlIkiOOdbsrE6tFesRLZNHAYspeRKI9MJ++Xg9i7rutU34ZM+1BL6KgZf
J9FSm+GFHiVWpr1MfYCo/w
--- fgMiVLJHMlg9fW7CVG/hPS5EAU4Zeg19LyCP7SoH5nA
";
        let h = Header::read(test_header.as_bytes()).unwrap();
        let mut data = vec![];
        h.write(&mut data).unwrap();
        assert_eq!(std::str::from_utf8(&data), Ok(test_header));
    }

    #[test]
    fn parse_legacy_header() {
        let test_header = "age-encryption.org/v1
-> X25519 CJM36AHmTbdHSuOQL+NESqyVQE75f2e610iRdLPEN20
C3ZAeY64NXS4QFrksLm3EGz+uPRyI0eQsWw7LWbbYig
-> some-empty-body-recipient BjH7FA 37 mhir0Q

-> some-full-body-recipient BjH7FA 37 mhir0Q
xD7o4VEOu1t7KZQ1gDgq2FPzBEeSRqbnqvQEXdLRYy143BxR6oFxsUUJCRB0ErXA

-> some-other-recipient mhir0Q BjH7FA 37
m/uPLMQdlIkiOOdbsrE6tFesRLZNHAYspeRKI9MJ++Xg9i7rutU34ZM+1BL6KgZf
J9FSm+GFHiVWpr1MfYCo/w
--- fgMiVLJHMlg9fW7CVG/hPS5EAU4Zeg19LyCP7SoH5nA
";
        let test_legacy_header = "age-encryption.org/v1
-> X25519 CJM36AHmTbdHSuOQL+NESqyVQE75f2e610iRdLPEN20
C3ZAeY64NXS4QFrksLm3EGz+uPRyI0eQsWw7LWbbYig
-> some-empty-body-recipient BjH7FA 37 mhir0Q
-> some-full-body-recipient BjH7FA 37 mhir0Q
xD7o4VEOu1t7KZQ1gDgq2FPzBEeSRqbnqvQEXdLRYy143BxR6oFxsUUJCRB0ErXA
-> some-other-recipient mhir0Q BjH7FA 37
m/uPLMQdlIkiOOdbsrE6tFesRLZNHAYspeRKI9MJ++Xg9i7rutU34ZM+1BL6KgZf
J9FSm+GFHiVWpr1MfYCo/w
--- fgMiVLJHMlg9fW7CVG/hPS5EAU4Zeg19LyCP7SoH5nA
";
        let mut h = Header::read(test_header.as_bytes()).unwrap();
        let mut h_legacy = Header::read(test_legacy_header.as_bytes()).unwrap();
        // Remove the `encoded_bytes` field from both headers, which is intentionally
        // different (to ensure their MACs would validate, if these were real headers).
        match (&mut h, &mut h_legacy) {
            (Header::V1(h), Header::V1(h_legacy)) => {
                h.encoded_bytes = None;
                h_legacy.encoded_bytes = None;
            }
            _ => unreachable!(),
        }
        // The remainder of the headers should be identical.
        assert_eq!(h, h_legacy);
    }
}
