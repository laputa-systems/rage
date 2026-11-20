# Verification

Source builds and release procedures: [docs/debian.md](docs/debian.md).

Run `make test` for the workspace's default-feature test suite and `make lint` for Clippy across workspace targets. Set `TMPDIR=$PWD/.work/tmp` after creating that directory. On a loaded macOS arm64 host the cold test took 213 s, a consecutive warm test 36 s, and lint 627 s including build-lock wait; these are observations, not performance gates.

- CLI flags, errors, and exit codes: `rage/tests/cli_tests.rs` and `rage/tests/cmd/`.
- CLI input/output preservation: `rage/tests/io_regressions.rs`.
- Stanza wire format: `age-core/src/format.rs` tests; header format: `age/src/format.rs` tests.
- Library API and encryption/decryption: workspace unit and integration tests; downstream consumer: vault.
- Plugin protocol: `age/src/plugin.rs` tests when the plugin feature is enabled; `age-plugin` boundary: none identified.
- Performance: `age/benches/parser.rs` and `age/benches/throughput.rs`; absolute timing requires a suitable host.
