# Task 1 TDD Proof (Red -> Green)

## Red (before scaffold)

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
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Note

Task 1 followed a TDD red -> green flow: first confirmed failure (missing package), then added only the minimal scaffold needed for `core-sim` to exist and pass.
