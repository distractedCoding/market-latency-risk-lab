# Verification Checklist

Use this checklist before opening a pull request for the Rust workspace.

## Required Commands

1. `PATH="$HOME/.cargo/bin:$PATH" cargo fmt --all -- --check`
   - Expected: command exits with status 0 and no formatting diffs are reported.

2. `PATH="$HOME/.cargo/bin:$PATH" cargo clippy --workspace -- -D warnings`
   - Expected: command exits with status 0 and reports no warnings (warnings are treated as errors).

3. `PATH="$HOME/.cargo/bin:$PATH" cargo test --workspace`
   - Expected: command exits with status 0 and all tests pass.

4. `PATH="$HOME/.cargo/bin:$PATH" cargo test -p runtime engine::tests::live_runner_emits_intent_then_fill_events -- --exact`
   - Expected: command exits with status 0 and reports exactly one matching runtime test passed.

5. `PATH="$HOME/.cargo/bin:$PATH" cargo test -p api tests::websocket_emits_paper_fill_event_payload -- --exact`
   - Expected: command exits with status 0 and reports exactly one matching API websocket payload test passed.

6. `PATH="$HOME/.cargo/bin:$PATH" cargo bench -p runtime --no-fail-fast`
   - Expected: command exits with status 0 and runtime benchmarks complete without failures.
