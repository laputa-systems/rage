# Building and releasing rage

## Build from source

Use the toolchain pinned in [`rust-toolchain.toml`](../rust-toolchain.toml).
A normal checkout supplies the registry configuration in
[`.cargo/config.toml`](../.cargo/config.toml).

```sh
cargo build --locked -p rage --bins
```

The binaries are `target/debug/rage` and `target/debug/rage-keygen`.
`rage keygen` is also available in the main executable.
Source archives without Git metadata must set `RAGE_GIT_SHA` to their hexadecimal
source commit ID. Both binaries include that ID in `--version` output.

The CLI enables SSH support by default. Use `--no-default-features` to omit it.
`--features unstable` enables functionality without stability or interoperability
guarantees. Library features are documented in [`age/README.md`](../age/README.md).

## Check a change

`make check` runs workspace Clippy with warnings denied, then library and
integration tests. The feature-gated format testkit also needs `armor` and `async`:

```sh
cargo test --locked -p age --features armor,async --test testkit
```

The publication fixture uses a disposable Git repository and mocked Cargo,
registry reads, and publication. It needs Git, `rg`, and `jaq` with TOML support:

```sh
mkdir -p .work/tmp
sh tools/test_publish_age_core.sh
```

## Build release binaries

[`Makefile`](../Makefile) owns `release`, `verify-release`, `test-ci`, and `install`.
On Linux, use the builder defined by [`Dockerfile`](../Dockerfile), which supplies
musl, LLVM, Rust source, and the `musl-cargo` wrapper. The release workflow in
[`.github/workflows/release.yml`](../.github/workflows/release.yml) contains the
native target commands for Linux x86-64, Linux arm64, and macOS arm64.

`make install` builds a release, installs `rage` under `~/.local/bin`, and creates
`rage-keygen` as a symlink. It also signs the installed binary on macOS.

Hosted releases run manually on the default branch and package the triggering
commit. They reject an existing `release-<commit>` tag. If an attempt fails after
tag creation, inspect that tag and any partial GitHub release before recovery;
the workflow does not automatically resume that state.

## Publish age-core

`make publish` requires a clean working tree, Git, Cargo, `curl`, `rg`, `jaq` with
TOML support, and `stuff` configured to publish to `mycrates`. It bumps the
`age-core` minor version and workspace requirement together, checks the package,
commits and tags the release, then publishes through `stuff`.
It does not push the commit or tag. Rerun it immediately to resume an interrupted
publication while the release commit remains at HEAD.

The implementation is [`tools/publish_age_core.sh`](../tools/publish_age_core.sh).

## Build Debian packages

The package assets are declared in [`rage/Cargo.toml`](../rage/Cargo.toml).
With `cargo-deb` installed, run:

```sh
cargo deb --package rage
```
