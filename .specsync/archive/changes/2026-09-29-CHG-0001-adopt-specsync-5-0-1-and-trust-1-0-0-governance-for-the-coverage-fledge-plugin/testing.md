---
change: CHG-0001-adopt-specsync-5-0-1-and-trust-1-0-0-governance-for-the-coverage-fledge-plugin
artifact: testing
---

# Testing

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `specsync check --strict --require-coverage 100 --force`
- `specsync agents status`
- `fledge trust doctor` and `fledge trust verify`
