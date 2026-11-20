#!/usr/bin/env bash
# Benchmark entrypoint for the age crate's rustybench harnesses.
#
#   tools/bench.sh [--check|--list] [--tier edit|change|full] [--bench parser|throughput]
#                  [--dry-run] [FILTER...]
#
# Tiers:
#   edit    each selected benchmark once, untimed (rustybench --test): proves the
#           harness and its oracles still run. Seconds after the build.
#   change  one short timed round (10 samples) of header parsing with 1 and 9
#           recipients and 64 KiB encrypt and decrypt. About a minute.
#   full    every registered benchmark with rustybench's default sampling. Run it
#           only when asked; it is never the default and no check runs it.
#
# FILTER arguments are regular expressions over names such as
# throughput::decrypt::65536 and replace the tier's own selection. Selecting
# nothing, or naming an unknown tier or bench, is an error. Timing numbers from a
# busy host are not evidence on their own; report paired ratios (see AGENTS.md).
set -euo pipefail

root=$(CDPATH='' cd -P "$(dirname "$0")/.." && pwd -P)
cd "$root"

mode=run tier=edit only=all dry=0
filters=()
while [ $# -gt 0 ]; do
  case "$1" in
    --check) mode=check ;;
    --list) mode=list ;;
    --dry-run) dry=1 ;;
    --tier) tier=${2:?--tier needs a value}; shift ;;
    --bench) only=${2:?--bench needs a value}; shift ;;
    -h|--help) sed -n '2,20p' "$0"; exit 0 ;;
    -*) echo "bench: unknown option $1" >&2; exit 2 ;;
    *) filters+=("$1") ;;
  esac
  shift
done

case "$tier" in edit|change|full) ;; *) echo "bench: unknown tier '$tier' (edit, change, full)" >&2; exit 2 ;; esac
case "$only" in all) benches="parser throughput" ;; parser|throughput) benches=$only ;;
  *) echo "bench: unknown bench '$only' (parser, throughput)" >&2; exit 2 ;; esac

tier_filter() { # bench
  case "$tier/$1" in
    change/parser) echo 'parse::(1|9)$' ;;
    change/throughput) echo '::65536$' ;;
    *) echo '' ;;
  esac
}

cargo_bench() { cargo bench --offline -p age --bench "$@"; }

if [ "$mode" = check ]; then
  command -v cargo >/dev/null || { echo "bench: cargo not found; install the toolchain in rust-toolchain.toml" >&2; exit 1; }
  case $(rustc -vV) in *"release: "*nightly*) ;; *) echo "bench: rustybench needs the nightly in rust-toolchain.toml; run from the repository root" >&2; exit 1 ;; esac
  free_kb=$(df -k "$root" | awk 'NR==2 {print $4}')
  [ "$free_kb" -gt 2097152 ] || { echo "bench: less than 2 GiB free under $root" >&2; exit 1; }
  echo "bench: load $(uptime | sed 's/.*load averages*: //') (information; a busy host needs paired ratios)"
  if [ "$dry" = 1 ]; then echo "would run: cargo bench --offline -p age --no-run --benches"; exit 0; fi
  cargo bench --offline -p age --no-run --benches >/dev/null
  echo "bench: ready"
  exit 0
fi

selected=0
for b in $benches; do
  if [ ${#filters[@]} -gt 0 ]; then f=("${filters[@]}"); elif [ -n "$(tier_filter "$b")" ]; then f=("$(tier_filter "$b")"); else f=(); fi
  if [ "$dry" = 1 ]; then
    args=("$b" --); [ "$mode" = list ] && args+=(--list --format terse)
    [ "$mode" = run ] && [ "$tier" = edit ] && args+=(--test)
    [ "$mode" = run ] && [ "$tier" = change ] && args+=(--sample-count 10)
    echo "would run: cargo bench --offline -p age --bench ${args[*]} ${f[*]-}"
    selected=1
    continue
  fi
  names=$(cargo_bench "$b" -- --list --format terse ${f[@]+"${f[@]}"} 2>/dev/null | grep ': benchmark$' || true)
  [ -n "$names" ] || continue
  selected=$((selected + 1))
  if [ "$mode" = list ]; then printf '%s\n' "$names"; continue; fi
  echo "bench: $tier tier, $b ($(printf '%s\n' "$names" | wc -l | tr -d ' ') benchmarks)"
  case "$tier" in
    edit) cargo_bench "$b" -- --test ${f[@]+"${f[@]}"} ;;
    change) cargo_bench "$b" -- --sample-count 10 ${f[@]+"${f[@]}"} ;;
    full) cargo_bench "$b" -- ${f[@]+"${f[@]}"} ;;
  esac
done
[ "$selected" -gt 0 ] || { echo "bench: no benchmark matches the selection" >&2; exit 2; }
