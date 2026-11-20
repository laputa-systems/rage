Prefix: RAGE
Next: 6

# Resolved work

RAGE-1 through RAGE-5 were fixed by `c064fc4` and closed after each fix was found
guarded and its guard was proven against a hand-injected fault (see the commit that
closed them): hard-link and alias rejection and the lazy-output policy in
`rage/tests/io_regressions.rs`; the final-flush error in
`rage/src/bin/rage/app.rs` (`io_tests`); identity short-read replay in
`age/src/cli_common/identities.rs` tests and `io_regressions.rs`; release archive
modes and musl build modes in `tools/test_delivery.sh`.
