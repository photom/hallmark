## Summary

<!-- What this pull request changes, and why. -->

## Related issues

<!-- "Closes #123", or "None". -->

## Testing

<!-- How you checked the change. Mark what you ran. -->

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo clippy -p hallmark -p hallmark-wasm --target wasm32-unknown-unknown --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `cargo deny --all-features check`

## Checklist

- [ ] Documentation in `docs/` matches the change.
- [ ] A design decision has an ADR in `docs/adr/`.
- [ ] Every new test has a scenario in `docs/testing/` that cites the RFC or draft section it checks.
- [ ] Email addresses and domains in tests and examples use `.invalid`.
- [ ] No secrets, private keys (other than RFC example keys or keys generated in a test), or real email addresses are committed.
