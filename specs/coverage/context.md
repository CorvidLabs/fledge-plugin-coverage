---
spec: coverage.spec.md
---

## Context

This Rust plugin provides a language-neutral coverage gate over native project tools using fledge-v1 execution.

## Related Modules

- fledge-v1 exec capability

## Design Decisions

- Preserve native tooling rather than reimplement instrumentation.
- Normalize only the headline total needed for a portable threshold gate.
