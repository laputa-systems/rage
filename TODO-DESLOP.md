Prefix: DSLP
Next: 8

Deslop campaign backlog ([plan](https://github.com/laputa-systems/stuff/blob/main/docs/campaigns/deslop.md), tier E). One agent works this file with the `deslop` skill: take the lowest-numbered open ticket whose area does not start with `Needs owner ·`. After DSLP-5, work the open P1 and P2 findings, then fill in `## Report` below.

## Report

Not started.

## Tickets

### DSLP-3 · P2 · Types: strict profile and typing findings

- **Area:** deslop · types
- **Where:** type checker and lint configuration; Rust sources
- **Problem:** The strict-profile candidates in the deslop skill's tools reference have not been measured here.
- **Recommendation:** Run the deslop Types step. Accept: clean at the adopted profile, escape hatches not increased, witnesses recorded.
- **Fix with:** deslop
- **Effort:** M

### DSLP-4 · P2 · Tests: coverage, boundary gaps, mutation

- **Area:** deslop · tests
- **Where:** test suites and CI
- **Problem:** Coverage and mutation results are unmeasured; no deslop mutation lane exists.
- **Recommendation:** Run the deslop Tests step. Accept: no undispositioned survivor in critical code, the test cost budget holds, and the lane is green where the tools are mirrored.
- **Fix with:** deslop
- **Effort:** M

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

