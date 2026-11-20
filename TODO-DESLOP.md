Prefix: DSLP
Next: 10

Deslop campaign backlog ([plan](https://github.com/laputa-systems/stuff/blob/main/docs/campaigns/deslop.md), tier E). One agent works this file with the `deslop` skill: take the lowest-numbered open ticket whose area does not start with `Needs owner ·`. After DSLP-5, work the open P1 and P2 findings, then fill in `## Report` below.

## Report

Not started.

## Tickets

### DSLP-6 · P2 · Bench: audit benchmark coverage and entrypoint

- **Area:** deslop · bench
- **Where:** age/benches/, Makefile
- **Problem:** The fork benchmarks parser and throughput but their outcome coverage and selective entrypoint have not been audited.
- **Recommendation:** Run the deslop Bench step and record ranked gaps and the selective entrypoint evidence.
- **Fix with:** deslop
- **Effort:** M

### DSLP-7 · P3 · Report sub-1024-bit RSA recipients as weak keys

- **Area:** ssh · errors
- **Where:** age/src/ssh.rs:511; age/src/ssh/recipient.rs:224
- **Problem:** The upgraded RSA parser rejects moduli below 1024 bits before the existing 2048-bit weak-key classification. A generated 768-bit SSH RSA recipient makes the built rage binary report unsupported ssh-rsa instead of the specific weak-key error; the public ParseRecipientKeyError contract also distinguishes RsaModulusTooSmall.
- **Recommendation:** Preserve rejection while classifying sub-1024-bit RSA recipients as weak, with a regression covering both sides of the RSA library minimum.
- **Fix with:** error-model
- **Effort:** S

### DSLP-8 · P3 · Delivery regression script is outside make test and CI, and needs GNU tar

- **Area:** deslop · tests
- **Where:** tools/test_delivery.sh; .github/workflows/ci.yml
- **Problem:** RAGE-4 (release archive modes) and RAGE-5 (musl build modes) are guarded only by tools/test_delivery.sh. Neither make test nor ci.yml runs it, and it needs GNU tar (--sort, --mtime), jaq and rg, so on macOS (bsdtar) it exits 1 at the package step. On this host the mode and musl-mode assertions were reproduced only through a bsdtar shim and by running the Makefile half directly.
- **Recommendation:** Run the script in the ubuntu CI job (or a dedicated step) once jaq is available there, and state the GNU tar requirement in its header.
- **Fix with:** deslop
- **Effort:** S

### DSLP-9 · P4 · Drop the unused optional num-traits dependency of age

- **Area:** Needs owner · age cargo features
- **Where:** age/Cargo.toml:61, age/Cargo.toml:110
- **Problem:** Nothing in age, rage, or their tests references num_traits: cargo check -p age --all-features --all-targets succeeds with the dependency and its entry in the ssh feature removed. But num-traits is an optional dependency, so it also defines the implicit feature age/num-traits, a public Cargo surface for downstream crates that enable it by name.
- **Recommendation:** Owner decides whether the implicit feature counts as a supported contract. If not, remove the dependency and its ssh feature entry; this is a walk-back of that feature name only.
- **Fix with:** dead-code-archaeology
- **Effort:** S

