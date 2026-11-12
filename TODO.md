Prefix: RAGE
Next: 6

# Resolved work

Priorities are assumed from impact and likelihood. The observations below were
reproduced against `33c3e18889d60cdbfe468b26b7871cd5d76b81e9` with disposable
fixtures. All five findings below are fixed; the original observations are retained for context.

### RAGE-1 · P1 · Hard-linked input and output can destroy the source while reporting success

- **Status:** Fixed. The lazy output retains the opened input identity and compares it with the opened output before truncation. CLI tests verify hard links during encryption and decryption and aliases introduced after setup, including symlinks.

- **Area:** Data
- **Where:** `rage/src/bin/rage/app.rs:377`, `age/src/cli_common/file_io.rs:286`, `age/src/cli_common/file_io.rs:293`
- **Problem:** The same-file guard compares canonical path strings. Two hard links have different canonical paths but refer to the same file. The lazy output then opens that file with truncation while it is also the input. In a local fixture, encrypting a 40-byte input to its hard-linked alias exited 0, replaced the original contents, and produced ciphertext that decrypted to an age header instead of those 40 bytes. Reproduce with a generated test identity, a small `input`, `ln input output`, and `rage -r RECIPIENT -o output input`, then compare the decrypted result with a saved copy. The source is already lost when the successful exit is reported.
- **Recommendation:** Establish input/output file identity before any truncating write, using open handles and platform file identity rather than path spelling. Reject identical files, including hard links and symlinks, while preserving the lazy-output behavior on earlier failures. Add a disposable-file regression that checks rejection and verifies both aliases still contain the original bytes.
- **Fix with:** `os-boundaries`
- **Effort:** M (days)

### RAGE-2 · P1 · Decryption returns success without finalizing its output

- **Status:** Fixed. Decryption explicitly flushes after a successful copy. Regression tests cover empty new and existing destinations, lazy failure behavior, and a failing final flush; a Linux-only test covers `/dev/full`.

- **Area:** Errors
- **Where:** `rage/src/bin/rage/app.rs:203`, `age/src/cli_common/file_io.rs:144`, `age/src/cli_common/file_io.rs:158`, `age/src/cli_common/file_io.rs:274`
- **Problem:** `write_output` returns after `io::copy` without flushing its writer. `LazyFile` opens only on write or flush, so decrypting an empty file with `-o new-file` exits 0 without creating it; an existing destination keeps its old bytes. Separately, a final stdout flush happens in `StdoutBuffer::drop`, which discards errors. On Linux, decrypting a valid ten-byte plaintext with no newline to stdout redirected to `/dev/full` exited 0 with empty stderr. All three cases were reproduced with a newly generated test identity and valid ciphertext.
- **Recommendation:** Explicitly finalize the output after a successful copy and propagate any flush error before returning success. Ensure successful empty output creates or truncates its destination, while failed operations retain the existing lazy-opening policy. Cover a new empty output, an existing sentinel output, and a writer whose final flush fails.
- **Fix with:** `error-model`
- **Effort:** S (hours)

### RAGE-3 · P2 · Identity input treats an initial short read as end of file

- **Status:** Fixed. Format detection now accumulates short reads into a bounded replay buffer. Segmented-reader tests cover replay and its size limit, and a CLI pipe test sends the comment separately from the identity.

- **Area:** Errors
- **Where:** `age/src/cli_common/identities.rs:155`, `age/src/cli_common/identities.rs:209`, `age/src/cli_common/identities.rs:224`
- **Problem:** While peeking, consuming a buffer shorter than `PEEKABLE_SIZE` makes `fill_buf` return an empty slice without asking the underlying reader for more bytes. A short pipe read is not EOF. In a local subprocess, `rage -d -i - input.age` received only a comment line in its first stdin write and exited 1 with "Missing identities" while the producer still held stdin open, before it could send the valid identity. This also affects identities supplied by other readers that return partial buffers.
- **Recommendation:** Keep a bounded replay buffer for format detection and track actual EOF separately from a short read. Preserve the current encrypted-identity, SSH, then line-oriented identity parsing order and reset contract. Exercise a segmented reader and a pipe whose writer remains open between a comment and a valid identity; the complete input must be consumed without assuming reads fill the buffer.
- **Fix with:** `make-invalid-states-unrepresentable`
- **Effort:** M (days)

### RAGE-4 · P2 · Release archives contain nonexecutable binaries

- **Status:** Fixed. Validated release executables are set to mode `0755` before ZIP creation. `bash tools/test_delivery.sh` packages fixtures initially at `0644`, checks checksums and extracted bytes, and executes each extracted fixture.

- **Area:** Build
- **Where:** `.github/workflows/release.yml:124`, `.github/workflows/release.yml:161`, `.github/workflows/release.yml:440`
- **Problem:** Build jobs upload bare executables with the artifact action's default archive mode. The pinned [upload action](https://github.com/actions/upload-artifact/blob/cf430e030ddbb5b0abf93d22962f4752f3646cd9/README.md#permission-loss) and [download action](https://github.com/actions/download-artifact/blob/9000827ccba6bdab643e8b6fd33ac0654aef8333/README.md#maintaining-file-permissions) document that this transfer produces files with mode `0644`. The release job zips those files without restoring executability. Running the actual packaging block against the three expected artifact filenames at mode `0644` exited 0 and passed its checksum checks, while every final ZIP entry retained mode `0100644`. This proof exercises the packaging code and pinned transfer contract, not a hosted release run.
- **Recommendation:** Restore mode `0755` on the validated executable files before creating the release ZIPs, or create the final archives before artifact transfer. Verify the mode of an extracted archive in the local packaging check as well as its bytes and checksum.
- **Fix with:** `os-boundaries`
- **Effort:** S (hours)

### RAGE-5 · P2 · The development build requests a mode the musl wrapper rejects

- **Status:** Fixed. `make build` selects the wrapper’s existing `static` mode. `bash tools/test_delivery.sh` exercises the actual Makefile and extracted wrapper, checking Cargo arguments and flags for development, test, and release modes.

- **Area:** Build
- **Where:** `Makefile:6`, `Makefile:12`, `Dockerfile:109`, `Dockerfile:117`
- **Problem:** `make build` selects `musl-cargo` when it is present and passes `MUSL_BUILD_MODE=dev`. The wrapper installed by this repository's Dockerfile accepts only `static`, `test`, `driver`, and `release-static`. Running the real Makefile with that exact extracted wrapper and harmless toolchain stubs exited 2 with `musl-cargo: unknown build mode: dev`; Cargo was never called. The normal development target is therefore unusable in the repository's own Linux builder.
- **Recommendation:** Give the Makefile and wrapper one compatible mode vocabulary: implement `dev` with the intended development flags, or select the existing equivalent mode. Verify that the development invocation reaches Cargo with the intended flags as well as preserving release and test modes.
- **Fix with:** `single-source-of-truth`
- **Effort:** S (hours)
