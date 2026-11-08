#!/bin/sh
set -eu

root=$(CDPATH='' cd -P "$(dirname "$0")/.." && pwd -P)
cd "$root"

fail() {
    printf 'make publish: %s\n' "$*" >&2
    exit 1
}

[ -z "$(git status --porcelain=v1)" ] || fail 'release requires a clean working tree'
current=$(jaq --from toml -r '.package.version' age-core/Cargo.toml)
requirement=$(jaq --from toml -r '.workspace.dependencies["age-core"].version' Cargo.toml)
[ "$current" = "$requirement" ] || fail 'age-core version and workspace dependency differ'
printf '%s\n' "$current" | rg -q '^[0-9]+\.[0-9]+\.[0-9]+$' || fail 'age-core version must be a stable version'
index=$(curl -fsSL https://crates.17166969.xyz/current/registry/ag/e-/age-core) || fail 'cannot read the mycrates index'
latest=$(printf '%s\n' "$index" | jaq -sr '[.[] | .vers | select(test("^[0-9]+\\.[0-9]+\\.[0-9]+$")) | split(".") | map(tonumber)] | sort | last | join(".")') || fail 'cannot read published versions'
[ -n "$latest" ] || fail 'no published stable age-core version found'
subject=$(git log -1 --format=%s)
tag="age-core-v$current"

if [ "$subject" = "chore(age-core): release $current" ]; then
    published=$(printf '%s\n' "$index" | jaq -sr --arg version "$current" '[.[] | select(.vers == $version)] | if length == 0 then "missing" elif .[0].yanked then "yanked" else "published" end')
    [ "$published" != yanked ] || fail "age-core $current is yanked"
    if [ "$published" = missing ]; then
        newer=$(jaq -nr --arg current "$current" --arg latest "$latest" '($latest | split(".") | map(tonumber)) >= ($current | split(".") | map(tonumber))')
        [ "$newer" = false ] || fail "latest published version $latest is not older than $current"
    fi
    if git rev-parse -q --verify "refs/tags/$tag" >/dev/null; then
        [ "$(git rev-list -n 1 "$tag")" = "$(git rev-parse HEAD)" ] || fail "$tag points to another commit"
    else
        git tag -a "$tag" -m "age-core $current"
    fi
    if [ "$published" = published ]; then
        printf 'age-core %s is already published\n' "$current"
        exit 0
    fi
    stuff publish cargo -p age-core --registry mycrates
    exit 0
fi

[ "$current" = "$latest" ] || fail "local version $current differs from latest published version $latest"
next=$(printf '%s\n' "$current" | jaq -Rr 'split(".") | .[0] + "." + ((.[1] | tonumber) + 1 | tostring) + ".0"')
tag="age-core-v$next"
if git rev-parse -q --verify "refs/tags/$tag" >/dev/null; then
    fail "$tag already exists"
fi

git check-ignore -q .work/tmp || printf '\n.work/\n' >> "$(git rev-parse --git-path info/exclude)"
mkdir -p .work/tmp
temporary=$(mktemp .work/tmp/publish-age-core.XXXXXX)
initial_head=$(git rev-parse HEAD)
cleanup() {
    status=$?
    trap - EXIT HUP INT TERM
    rm -f "$temporary"
    if [ "$status" -ne 0 ] && [ "$(git rev-parse HEAD)" = "$initial_head" ]; then
        git restore --staged --worktree -- Cargo.toml Cargo.lock age-core/Cargo.toml
    fi
    exit "$status"
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM

awk -v old="$current" -v new="$next" '
    $0 == "version = \"" old "\"" { print "version = \"" new "\""; count++; next }
    { print }
    END { if (count != 1) exit 1 }
' age-core/Cargo.toml > "$temporary" || fail 'cannot update age-core version'
cat "$temporary" > age-core/Cargo.toml
awk -v old="$current" -v new="$next" '
    $0 == "age-core = { version = \"" old "\", path = \"age-core\", registry = \"mycrates\" }" {
        print "age-core = { version = \"" new "\", path = \"age-core\", registry = \"mycrates\" }"
        count++
        next
    }
    { print }
    END { if (count != 1) exit 1 }
' Cargo.toml > "$temporary" || fail 'cannot update workspace dependency'
cat "$temporary" > Cargo.toml
cargo update --workspace --offline
cargo check -p age-core --locked
git add -- Cargo.toml Cargo.lock age-core/Cargo.toml
git commit -m "chore(age-core): release $next"
git tag -a "$tag" -m "age-core $next"
stuff publish cargo -p age-core --registry mycrates
