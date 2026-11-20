Prefix: DSLP
Next: 9

Deslop campaign backlog ([plan](https://github.com/laputa-systems/stuff/blob/main/docs/campaigns/deslop.md), tier E). One agent works this file with the `deslop` skill: take the lowest-numbered open ticket whose area does not start with `Needs owner ·`. After DSLP-5, work the open P1 and P2 findings, then fill in `## Report` below.

## Report

Not started.

## Tickets

### DSLP-5 · P2 · Dead code: tools and debloat

- **Area:** deslop · dead code
- **Where:** whole repository (our changes)
- **Problem:** No dead-code scan has run.
- **Recommendation:** Run the deslop Dead code step. Accept: tools clean or allowlisted with reasons; walk-backs filed as Needs owner tickets.
- **Fix with:** deslop
- **Effort:** M

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

