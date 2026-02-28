# Task 1 TDD Proof (Red -> Green)

## Red 1 (before any manifest)

Command:

```bash
cargo test -p core-sim -q
```

Result:

```text
error: could not find `Cargo.toml` in `/home/felix/code/market-latency-risk-lab/.worktrees/rust-latency-monolith` or any parent directory
```

## Red 2 (workspace exists, member missing)

Command:

```bash
cargo test -p core-sim -q
```

Result:

```text
error: package ID specification `core-sim` did not match any packages
```

## Green (after scaffold)

Command:

```bash
cargo test -p core-sim -q
```

Result:

```text
running 1 test
.

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Note

Task 1 followed a state-consistent red -> red -> green flow: first with no manifest, then with a workspace but no `core-sim` member, then with minimal crate scaffolding so `core-sim` tests pass.
