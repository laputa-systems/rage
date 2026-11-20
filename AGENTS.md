# Verification

Source builds and release procedures: [docs/debian.md](docs/debian.md).

Run `make test` for the workspace's default-feature test suite and `make lint` for Clippy across workspace targets. Set `TMPDIR=$PWD/.work/tmp` after creating that directory. On a loaded macOS arm64 host the cold test took 213 s, a consecutive warm test 36 s, and lint 627 s including build-lock wait; these are observations, not performance gates.

- CLI flags, errors, and exit codes: `rage/tests/cli_tests.rs` and `rage/tests/cmd/`.
- CLI input/output preservation: `rage/tests/io_regressions.rs`.
- Stanza wire format: `age-core/src/format.rs` tests; output errors: `age-core/tests/stanza_write.rs`; header format: `age/src/format.rs` tests.
- Library API and encryption/decryption: workspace unit and integration tests; downstream consumer: vault.
- Plugin protocol: `age-core/src/plugin.rs` mock protocol test and `age/src/plugin.rs` tests when the plugin feature is enabled, and `rage/tests/plugin_roundtrip.rs` through `make test`.
- Edit tier: a focused `cargo test -p PACKAGE TEST_NAME` (cost varies); change tier: `make test` and `make lint`. The format testkit uses `cargo test --locked -p age --features armor,async --test testkit` (492 cases, 3.30 s execution on the observed host; compilation 321.57 s wall).
- Performance: `age/benches/parser.rs` and `age/benches/throughput.rs`; absolute timing requires a suitable host.
- SSH RSA recipient admission: `age/src/ssh/recipient.rs` tests (round trip, 1024-bit weak key, sub-minimum rejection); the 768-bit variant's error class is open in DSLP-7.
- Release archive modes and musl build modes: `tools/test_delivery.sh` (needs GNU tar, `jaq` and `rg`; not part of `make test`).
- Coverage and mutation: `cargo-llvm-cov` and `cargo-mutants` are not installed here, so critical contracts are checked by hand-injected faults, recorded in `TODO-DESLOP.md`.
