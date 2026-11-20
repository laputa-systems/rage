Prefix: RAGE
Next: 12

# Resolved work

RAGE-1 through RAGE-5 were fixed by `c064fc4` and closed after each fix was found
guarded and its guard was proven against a hand-injected fault (see the commit that
closed them): hard-link and alias rejection and the lazy-output policy in
`rage/tests/io_regressions.rs`; the final-flush error in
`rage/src/bin/rage/app.rs` (`io_tests`); identity short-read replay in
`age/src/cli_common/identities.rs` tests and `io_regressions.rs`; release archive
modes and musl build modes in `tools/test_delivery.sh`.
### RAGE-6 · P3 · Report sub-1024-bit RSA recipients as weak keys

- **Area:** ssh · errors
- **Where:** age/src/ssh.rs:511; age/src/ssh/recipient.rs:224
- **Problem:** The upgraded RSA parser rejects moduli below 1024 bits before the existing 2048-bit weak-key classification. A generated 768-bit SSH RSA recipient makes the built rage binary report unsupported ssh-rsa instead of the specific weak-key error; the public ParseRecipientKeyError contract also distinguishes RsaModulusTooSmall.
- **Recommendation:** Preserve rejection while classifying sub-1024-bit RSA recipients as weak, with a regression covering both sides of the RSA library minimum.
- **Fix with:** error-model
- **Effort:** S

### RAGE-7 · P3 · Delivery regression script is outside make test and CI, and needs GNU tar

- **Area:** tests
- **Where:** tools/test_delivery.sh; .github/workflows/ci.yml
- **Problem:** RAGE-4 (release archive modes) and RAGE-5 (musl build modes) are guarded only by tools/test_delivery.sh. Neither make test nor ci.yml runs it, and it needs GNU tar (--sort, --mtime), jaq and rg, so on macOS (bsdtar) it exits 1 at the package step. On this host the mode and musl-mode assertions were reproduced only through a bsdtar shim and by running the Makefile half directly.
- **Recommendation:** Run the script in the ubuntu CI job (or a dedicated step) once jaq is available there, and state the GNU tar requirement in its header.
- **Fix with:** deslop
- **Effort:** S

### RAGE-8 · P3 · No benchmark covers a rage user journey, passphrase, SSH or armor paths

- **Area:** bench
- **Where:** age/benches/, rage/
- **Problem:** The two harnesses time x25519 header parsing (empty payload, 1-9 recipients) and x25519 encrypt/decrypt of constant bytes through the library. Nothing times the rage binary end to end (startup, file to file, stdin to stdout), scrypt passphrase decryption (work-factor bound), ssh-rsa/ssh-ed25519 recipients and identities, armor, the async reader, or a plugin round trip. A regression in any of these, including the I/O paths this fork rewrote (output identity check, identity replay), passes unnoticed.
- **Recommendation:** Add a Change-tier workload per outcome, starting with rage encrypt and decrypt of a 1 MiB file through the built binary, and a passphrase decrypt at the default work factor.
- **Fix with:** bench-pyramid
- **Effort:** M

### RAGE-9 · P3 · Benchmark results have no budget, baseline or allocation attribution

- **Area:** bench
- **Where:** age/benches/, tools/bench.sh
- **Problem:** Runs print timings and exit 0 whatever they measure, so a slowdown fails nothing. No baseline file is kept, although rustybench can write and check one (baseline, diff, check), and nothing records allocations or peak memory for the stream paths. Each result is also not stamped with the commit or host load, so a busy-host number cannot be told from a quiet one afterwards.
- **Recommendation:** Keep a host-keyed baseline for the Change tier and report paired ratios with rustybench diff; add allocation counters to the throughput workloads. Gate nothing until a quiet-host spread is measured.
- **Fix with:** bench-pyramid
- **Effort:** M

### RAGE-10 · P3 · The plugin round trip test is unverified under make test-ci and the musl release lane

- **Area:** tests
- **Where:** Makefile test-ci; rage/tests/plugin_roundtrip.rs; .github/workflows/release.yml:90,109
- **Problem:** rage/tests/plugin_roundtrip.rs needs the age-plugin-unencrypted example beside the rage test binary. make test builds it, and make test-ci now builds the release example, but test-ci needs a built dist binary and the musl docker lane, neither of which could run here, so where the example lands for a --target or musl-cargo build is unverified.
- **Recommendation:** Run make test-ci in the next release workflow dispatch and, if the example path differs, resolve it from CARGO_BIN_EXE_rage's profile directory or build it from the test with cargo.
- **Fix with:** deslop
- **Effort:** S

### RAGE-11 · P4 · Drop the unused optional num-traits dependency of age

- **Area:** Needs owner · age cargo features
- **Where:** age/Cargo.toml:61, age/Cargo.toml:110
- **Problem:** Nothing in age, rage, or their tests references num_traits: cargo check -p age --all-features --all-targets succeeds with the dependency and its entry in the ssh feature removed. But num-traits is an optional dependency, so it also defines the implicit feature age/num-traits, a public Cargo surface for downstream crates that enable it by name.
- **Recommendation:** Owner decides whether the implicit feature counts as a supported contract. If not, remove the dependency and its ssh feature entry; this is a walk-back of that feature name only.
- **Fix with:** dead-code-archaeology
- **Effort:** S
