#!/bin/sh
set -eu

root=$(CDPATH='' cd -P "$(dirname "$0")/.." && pwd -P)
fixture=$(mktemp -d "$root/.work/tmp/publish-test.XXXXXX")
trap 'rm -rf "$fixture"' EXIT HUP INT TERM
mkdir -p "$fixture/repo/age-core" "$fixture/repo/tools" "$fixture/bin"
cp "$root/tools/publish_age_core.sh" "$fixture/repo/tools/publish_age_core.sh"
cat > "$fixture/repo/Cargo.toml" <<'MANIFEST'
[workspace]
members = ["age-core"]
[workspace.dependencies]
age-core = { version = "0.12.0", path = "age-core", registry = "mycrates" }
MANIFEST
cat > "$fixture/repo/age-core/Cargo.toml" <<'MANIFEST'
[package]
name = "age-core"
version = "0.12.0"
MANIFEST
printf 'old lock\n' > "$fixture/repo/Cargo.lock"
cat > "$fixture/bin/curl" <<'MOCK'
#!/bin/sh
printf '{"vers":"%s","yanked":false}\n' "${PUBLISHED_VERSION:-0.12.0}"
MOCK
cat > "$fixture/bin/cargo" <<'MOCK'
#!/bin/sh
case "$1" in
    update) printf 'new lock\n' > Cargo.lock ;;
    check) [ ! -f "$FAIL_CHECK" ] ;;
    *) exit 1 ;;
esac
MOCK
cat > "$fixture/bin/stuff" <<'MOCK'
#!/bin/sh
[ "$*" = 'publish cargo -p age-core --registry mycrates' ]
printf 'publish\n' >> "$PUBLISH_LOG"
[ ! -f "$FAIL_PUBLISH" ]
MOCK
chmod +x "$fixture/bin/"*
cd "$fixture/repo"
git init -q
git add .
git -c user.name=test -c user.email=test@example.com commit -qm 'chore: initial'
export PATH="$fixture/bin:$PATH"
hash -r
export PUBLISH_LOG="$fixture/publishes" FAIL_PUBLISH="$fixture/fail" FAIL_CHECK="$fixture/fail-check"
touch "$FAIL_CHECK"
if /bin/sh tools/publish_age_core.sh; then
    printf '%s\n' 'expected failed check to abort release' >&2
    exit 1
fi
[ "$(git status --porcelain)" = '' ]
[ "$(jaq --from toml -r '.package.version' age-core/Cargo.toml)" = '0.12.0' ]
rm "$FAIL_CHECK"
touch "$FAIL_PUBLISH"
if /bin/sh tools/publish_age_core.sh; then
    printf '%s\n' 'expected interrupted publish to fail' >&2
    exit 1
fi
[ "$(jaq --from toml -r '.package.version' age-core/Cargo.toml)" = '0.13.0' ]
[ "$(jaq --from toml -r '.workspace.dependencies["age-core"].version' Cargo.toml)" = '0.13.0' ]
[ "$(git log -1 --format=%s)" = 'chore(age-core): release 0.13.0' ]
[ "$(git tag --list 'age-core-v0.13.0')" = 'age-core-v0.13.0' ]
rm "$FAIL_PUBLISH"
/bin/sh tools/publish_age_core.sh
[ "$(wc -l < "$PUBLISH_LOG" | tr -d ' ')" = '2' ]
export PUBLISHED_VERSION=0.13.0
/bin/sh tools/publish_age_core.sh
[ "$(wc -l < "$PUBLISH_LOG" | tr -d ' ')" = '2' ]
[ "$(git status --porcelain)" = '' ]
printf '%s\n' 'publish retry and version updates passed'
