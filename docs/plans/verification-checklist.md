# Verification Checklist

Use this checklist before opening a pull request for the Rust workspace.

## Required Commands

1. `PATH="$HOME/.cargo/bin:$PATH" cargo fmt --check`
   - Expected: command exits with status 0 and no formatting diffs are reported.

2. `PATH="$HOME/.cargo/bin:$PATH" cargo clippy --workspace -- -D warnings`
   - Expected: command exits with status 0 and reports no warnings (warnings are treated as errors).

3. `PATH="$HOME/.cargo/bin:$PATH" cargo test --workspace`
   - Expected: command exits with status 0 and all tests pass.

4. `PATH="$HOME/.cargo/bin:$PATH" cargo bench -p runtime --no-fail-fast`
   - Expected: command exits with status 0 and runtime benchmarks complete without failures.
