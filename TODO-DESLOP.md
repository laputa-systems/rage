Prefix: DSLP
Next: 13

Deslop campaign backlog ([plan](https://github.com/laputa-systems/stuff/blob/main/docs/campaigns/deslop.md), tier E). One agent works this file with the `deslop` skill: take the lowest-numbered open ticket whose area does not start with `Needs owner ·`. After DSLP-5, work the open P1 and P2 findings, then fill in `## Report` below.

## Report

Base for the pass: `1c342bc` (origin/HEAD). Steps and commits: Ground `83e9979`, Diagnose `f95ea7b`, Types `9702bbd` (first worker, kept); Tests `d14e09f`, RAGE-1..5 reconciliation `949aa7b`, Dead code `5eeba2f`, Bench `a66cfe5`, completion-review fix `2874438`, Report (this commit). No P1 or P2 finding is open; the one open `Needs owner ·` item is DSLP-9. CI runs `make test` on `pull_request` (`.github/workflows/ci.yml`).

**Scorecard** (aarch64-apple-darwin, host load 10 to 30 throughout, so every wall time is inflated):

| Measure | Before (Ground) | After |
| --- | --- | --- |
| Rust lines, age + age-core + age-plugin + rage | 16,264 | 16,405 (tests/ dirs 1,101 to 1,230; benches 74 to 74) |
| Tests passed (make test) | 105 | 109 passed, 1 ignored |
| Warm `make test` | 36 s | 11 s consecutive; 480 s when it alternates with lint, because `make test` sets a linker env that invalidates the shared target dir |
| Cold `make test` / `make lint` | 213 s / 627 s | 480 to 771 s / 358 to 434 s (loaded host) |
| Line and branch coverage | unmeasured | unmeasured: `cargo-llvm-cov` is not installed |
| Mutants (`cargo-mutants` not installed) | none | 17 hand-injected faults: 13 caught, 3 survived and dispositioned, 1 unviable (build killed, rerun as another fault) |
| Escape hatches (unsafe, allow, unwrap, expect) | 258 | 258 |
| Dead code removed | 0 | 1 dependency (sha3), 1 macro arm (4 lines), 1 visibility narrowed; num-traits filed |
| Findings | 1 filed (DSLP-7, P3) | filed: DSLP-7 P3, DSLP-8 P3, DSLP-9 P4 (Needs owner), DSLP-10 P3, DSLP-11 P3, DSLP-12 P3. Fixed: none were P1 or P2. RAGE-1..5 (2 P1, 3 P2) were already fixed by `c064fc4` and were closed after their guards were proven |
| Perf verdict | n/a | unaffected: no runtime path changed in a measured workload; no quiet host, no budgets; Full never run |
| Escaped regressions | | none seen |

**Per step.** Ground and Diagnose (first worker): baseline costs, boundary map in AGENTS.md, one reproduced defect (DSLP-7); cold test 213 s, warm test 36 s, lint 627 s including a build-lock wait. Types: strict lint candidates measured, none adopted (no defect witness or only upstream false positives); no cost beyond the measurement. Tests: found three unguarded boundaries (stanza writer output errors, plugin protocol through the rage binary, sub-2048-bit SSH RSA recipients) and added one test file each plus two unit tests; test cost is under 1 s warm. Fault survivors: plugin send without flush (equivalent, the child's stdin pipe is unbuffered), RSA lower size bound 1024 to 0 (changes only the error variant, which is DSLP-7), final flush ignored (survives the CLI tier, caught by the `io_tests` unit). RAGE reconciliation: all five tickets fixed and guarded; guards proven by fault (alias check off, flush error ignored or removed, short read treated as EOF), RAGE-4 and RAGE-5 only through a bsdtar shim and the Makefile half of `tools/test_delivery.sh` because the script needs GNU tar (DSLP-8). Dead code: clippy `--all-features -D warnings` clean; sha3 proven unused by building without it. Bench: tiered `make bench` (edit 1.3 s warm, change 10 s, full manual) proven with unknown tier, unknown bench, empty selection, unmet preflight, dry run, and a faulted harness. Bench audit gaps: DSLP-10 and DSLP-11 (no rage journey, scrypt, SSH or armor workload; no budget, baseline or allocation attribution). Vault was not run: no public API, feature or format changed.

**Method feedback.** `make lint`, `make test` and plain `cargo test` fingerprint differently, so alternating them recompiles for 6 to 13 minutes; time warm as two consecutive `make test` runs. On a loaded host rustc is intermittently SIGKILLed; each lint or test failure here was that, with a clean rerun. The `todo` tool cannot parse a ticket with a `Status:` field or an effort such as `M (days)`, which kept RAGE-1..5 invisible to it. A `pgrep -f` wait loop matches its own command line. `rustc -vV | grep -q` under `pipefail` fails spuriously by SIGPIPE.

**Shared-file gaps.** The deslop tools reference names `cargo-llvm-cov` and `cargo-mutants` as available, but neither is installed on this host, so the Tests step cannot meet its coverage and mutation acceptance; a mirrored install or a hand-fault allowance in `references/tools.md` would settle it. `make coverage` and `make mutants-diff` were not added for the same reason. `tools/test_delivery.sh` needs GNU tar and `jaq` on the CI image (DSLP-8).

## Tickets

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

### DSLP-10 · P3 · No benchmark covers a rage user journey, passphrase, SSH or armor paths

- **Area:** deslop · bench
- **Where:** age/benches/, rage/
- **Problem:** The two harnesses time x25519 header parsing (empty payload, 1-9 recipients) and x25519 encrypt/decrypt of constant bytes through the library. Nothing times the rage binary end to end (startup, file to file, stdin to stdout), scrypt passphrase decryption (work-factor bound), ssh-rsa/ssh-ed25519 recipients and identities, armor, the async reader, or a plugin round trip. A regression in any of these, including the I/O paths this fork rewrote (output identity check, identity replay), passes unnoticed.
- **Recommendation:** Add a Change-tier workload per outcome, starting with rage encrypt and decrypt of a 1 MiB file through the built binary, and a passphrase decrypt at the default work factor.
- **Fix with:** bench-pyramid
- **Effort:** M

### DSLP-11 · P3 · Benchmark results have no budget, baseline or allocation attribution

- **Area:** deslop · bench
- **Where:** age/benches/, tools/bench.sh
- **Problem:** Runs print timings and exit 0 whatever they measure, so a slowdown fails nothing. No baseline file is kept, although rustybench can write and check one (baseline, diff, check), and nothing records allocations or peak memory for the stream paths. Each result is also not stamped with the commit or host load, so a busy-host number cannot be told from a quiet one afterwards.
- **Recommendation:** Keep a host-keyed baseline for the Change tier and report paired ratios with rustybench diff; add allocation counters to the throughput workloads. Gate nothing until a quiet-host spread is measured.
- **Fix with:** bench-pyramid
- **Effort:** M

### DSLP-12 · P3 · The plugin round trip test is unverified under make test-ci and the musl release lane

- **Area:** deslop · tests
- **Where:** Makefile test-ci; rage/tests/plugin_roundtrip.rs; .github/workflows/release.yml:90,109
- **Problem:** rage/tests/plugin_roundtrip.rs needs the age-plugin-unencrypted example beside the rage test binary. make test builds it, and make test-ci now builds the release example, but test-ci needs a built dist binary and the musl docker lane, neither of which could run here, so where the example lands for a --target or musl-cargo build is unverified.
- **Recommendation:** Run make test-ci in the next release workflow dispatch and, if the example path differs, resolve it from CARGO_BIN_EXE_rage's profile directory or build it from the test with cargo.
- **Fix with:** deslop
- **Effort:** S

### DSLP-9 · P4 · Drop the unused optional num-traits dependency of age

- **Area:** Needs owner · age cargo features
- **Where:** age/Cargo.toml:61, age/Cargo.toml:110
- **Problem:** Nothing in age, rage, or their tests references num_traits: cargo check -p age --all-features --all-targets succeeds with the dependency and its entry in the ssh feature removed. But num-traits is an optional dependency, so it also defines the implicit feature age/num-traits, a public Cargo surface for downstream crates that enable it by name.
- **Recommendation:** Owner decides whether the implicit feature counts as a supported contract. If not, remove the dependency and its ssh feature entry; this is a walk-back of that feature name only.
- **Fix with:** dead-code-archaeology
- **Effort:** S

