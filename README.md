# fledge-plugin-coverage

Run test coverage for the project and (optionally) fail the build if it drops below a threshold. Auto-detects the language and shells out to the right tool.

Built in Rust. Uses the [fledge-v1 plugin protocol](https://corvidlabs.github.io/fledge/plugin-protocol.html) with the `exec` capability so the language tool runs through fledge's sandboxed exec rather than directly.

## Install

```bash
fledge plugins install CorvidLabs/fledge-plugin-coverage
```

You'll be prompted to grant the `exec` capability.

## Usage

```bash
fledge coverage                          # detect language, run coverage, print %
fledge coverage --threshold 80           # gate: exit 1 if coverage < 80%
fledge coverage --lang python            # force language detection
fledge coverage --json                   # machine-readable
```

## Detection

| Language | Marker file(s) | Tool |
|----------|---------------|------|
| Rust | `Cargo.toml` | `cargo llvm-cov --summary-only --workspace` |
| Python | `pyproject.toml`, `setup.py`, `setup.cfg` | `pytest --cov=. --cov-report=term` |
| Bun | `bun.lockb`, `bun.lock` | `bun test --coverage` |
| Node | `package.json` | `npx jest --coverage --coverageReporters=text` |
| Go | `go.mod` | `go test -cover ./...` |

If the wrong tool is picked (e.g. you use vitest, not jest), pass `--lang` or wrap your tool in a fledge task and call that instead.

## Use in lanes

```toml
[lanes.coverage-gate]
description = "Block if coverage drops below 80%"
steps = [{ run = "fledge coverage --threshold 80" }]

[lanes.ci]
steps = [
  { parallel = ["fmt", "lint"] },
  { run = "fledge coverage --threshold 80" },
  "build",
]
```

## JSON output

```json
{
  "schema_version": 1,
  "action": "coverage",
  "language": "rust",
  "command": "cargo llvm-cov --summary-only --workspace",
  "exit_code": 0,
  "coverage_percent": 87.4,
  "threshold": 80.0,
  "gate_failed": false,
  "ok": true
}
```

`ok` is true when: the tool exited 0, coverage was extracted, and the threshold (if any) was met.

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | Coverage ran, parsed, and (if `--threshold` given) met the threshold |
| `1` | Tool failed, parse failed, or threshold gate failed |
| `2` | Couldn't detect a supported language (pass `--lang`) |
| `126` | `exec` capability not granted |

## Build

A pre-built binary ships at `bin/fledge-coverage`. If `cargo` is on PATH at install time, the build hook recompiles from source for the host platform.

## License

MIT
