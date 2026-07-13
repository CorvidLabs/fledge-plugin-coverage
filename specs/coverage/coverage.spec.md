---
module: coverage
version: 1
status: active
files:
  - src/main.rs

db_tables: []
depends_on: []
---

# Coverage

## Purpose

Detect a project's language, run its native coverage tool through fledge's exec capability, parse a total percentage, and optionally fail below a configured threshold.

## Public API

| Option | Behavior |
|--------|----------|
| default run | Detect language, run coverage, parse the total, and print a summary. |
| threshold | Fail when parsed coverage is below the requested percentage. |
| language override | Select Rust, Python, Bun, Node, or Go explicitly. |
| JSON output | Emit schema-versioned command, exit, percentage, threshold, gate, and overall status fields. |

## Invariants

1. Running a native coverage tool requires the fledge `exec` capability.
2. Detection prefers only supported project markers and an explicit override wins.
3. A successful result requires native tool success and a parsed coverage percentage.
4. The threshold gate fails only when a percentage is below the requested value.
5. JSON output retains schema version 1 and reports the actual command and exit code.
6. Coverage parser results remain bounded from 0 through 100.

## Behavioral Examples

```
Given a parsed total of 79.5 percent and a threshold of 80
When the coverage gate evaluates the result
Then it reports `gate_failed` true and exits 1
```

## Error Cases

| Error | When | Behavior |
|-------|------|----------|
| Missing exec capability | Coverage execution is requested | Report denial and exit 126. |
| Unsupported project | No supported marker or override exists | Report detection failure and exit 2. |
| Native tool failure | Coverage command returns non-zero | Surface failure and exit 1. |
| Unparseable output | No supported total can be extracted | Report parse failure and exit 1. |
| Threshold miss | Parsed total is below threshold | Report the shortfall and exit 1. |

## Dependencies

- fledge-v1 exec capability
- language-native Rust, Python, Bun, Node, or Go coverage tool
- `serde`, `serde_json`, and `regex`

## Change Log

| Version | Date | Changes |
|---------|------|---------|
| 1 | 2026-07-12 | Document existing coverage detection, parsing, and gate behavior for SpecSync 5 adoption. |
