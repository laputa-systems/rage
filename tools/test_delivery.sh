#!/usr/bin/env bash
set -euo pipefail

root=$(CDPATH='' cd -P "$(dirname "$0")/.." && pwd -P)
mkdir -p "$root/.work/tmp"
fixture=$(mktemp -d "$root/.work/tmp/delivery-test.XXXXXX")
trap 'rm -rf "$fixture"' EXIT
mkdir -p "$fixture/bin" "$fixture/dist"
jaq --from yaml -r '.jobs.release.steps[] | select(.name == "Validate and package release artifacts") | .run' \
  "$root/.github/workflows/release.yml" > "$fixture/package.sh"
for spec in \
  rage-aarch64-apple-darwin:rage-aarch64-apple-darwin \
  rage-aarch64-unknown-linux-musl:rage-aarch64-unknown-linux-musl-static \
  rage-x86_64-unknown-linux-musl:rage-x86_64-unknown-linux-musl-static; do
  artifact=${spec%%:*}
  binary=${spec#*:}
  mkdir "$fixture/dist/$artifact"
  printf '#!/bin/sh\nexit 0\n' > "$fixture/dist/$artifact/$binary"
  chmod 0644 "$fixture/dist/$artifact/$binary"
done
(cd "$fixture" && bash -eu package.sh)
for archive in "$fixture"/dist/zips/*.zip; do
  destination="$fixture/extracted/$(basename "$archive" .zip)"
  mkdir -p "$destination"
  unzip -q "$archive" -d "$destination"
  binary=$(basename "$archive" .zip)
  test -x "$destination/$binary"
  cmp "$destination/$binary" "$fixture/dist/${binary%-static}/$binary"
  "$destination/$binary"
done

awk '/^#!\/bin\/sh$/ { copying=1 } copying && /^EOF$/ { exit } copying { print }' \
  "$root/Dockerfile" > "$fixture/bin/musl-cargo"
cat > "$fixture/bin/cargo" <<'STUB'
#!/bin/sh
set -eu
printf '%s\n' "$*" "${MUSL_BUILD_MODE:-}" "${RUSTFLAGS:-}" \
  "${CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS:-}" > "$CARGO_LOG"
STUB
chmod +x "$fixture/bin/musl-cargo" "$fixture/bin/cargo"
export PATH="$fixture/bin:$PATH" CARGO_LOG="$fixture/cargo.log"
unset RUSTFLAGS CARGO_ENCODED_RUSTFLAGS
cd "$fixture"
make -s -f "$root/Makefile" TARGET=x86_64-unknown-linux-musl build
rg -x 'build' "$CARGO_LOG"
rg -x 'static' "$CARGO_LOG"
rg -x -- '-L native=/usr/lib -Clink-arg=-B/usr/lib/e-crt/x86_64-unknown-linux-musl' "$CARGO_LOG"
make -s -f "$root/Makefile" TARGET=x86_64-unknown-linux-musl test
rg -x 'test --quiet' "$CARGO_LOG"
rg -x 'test' "$CARGO_LOG"
make -s -f "$root/Makefile" TARGET=x86_64-unknown-linux-musl release
rg -- '^build --profile dist -p rage ' "$CARGO_LOG"
rg -x 'release-static' "$CARGO_LOG"
rg -- '-Cpanic=abort -Ctarget-feature=\+crt-static$' "$CARGO_LOG"
printf '%s\n' 'delivery regression checks passed'
