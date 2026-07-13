---
spec: coverage.spec.md
---

## User Stories

- As a developer, I want one command to run the native coverage tool for my project.
- As a CI author, I want builds to fail below a chosen coverage threshold.

## Acceptance Criteria

### REQ-coverage-001

The plugin SHALL detect and run supported Rust, Python, Bun, Node, and Go coverage tools.

### REQ-coverage-002

The plugin SHALL parse a total coverage percentage and fail when it is below an optional threshold.

### REQ-coverage-003

The plugin SHALL require the fledge exec capability before running native tooling.

### REQ-coverage-004

JSON output SHALL report schema version, language, command, exit code, percentage, threshold, gate status, and overall status.

## Constraints

- The project must provide its language-native coverage tool and recognizable output.

## Out of Scope

- Installing coverage tools and producing framework-specific rich reports.
