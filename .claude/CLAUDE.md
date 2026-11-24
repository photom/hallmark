# Working rules

These rules apply to every task in this repository.

## 1. Instructions and reporting

- Follow the requirements and instructions you are given for the whole task.
- Report progress as you go. When you hit a problem or a blocker, tell the user at once and do not push on without them.

## 2. Problem solving

- When an error occurs, find the root cause yourself and propose a fix.
- When several approaches are possible, present them and say which one you recommend.
- If the problem lies outside the source code, for example in the environment, the configuration, or the requirements, say so.

## 3. Existing code

- Follow the existing code style, naming conventions, and architecture.
- Before a major architectural change, explain why and get approval.

## 4. Two-strike rule

- If a fix or a test fails twice in a row, stop and summarize where things stand.
- Do not retry the same logic a third time. Re-examine the approach and propose a new hypothesis or solution.

## Practices

- Start small and grow the change step by step.
- Avoid over-abstraction. Readable, maintainable code comes first.
- Prefer strong type definitions to complex runtime checks. Hold an assumption in a type instead of filling a missing value with a default that no specification gives, or an `Option` that is only `None` while a value is built (ADR 0064).
- Keep each pull request within 400 lines of Rust under `crates/`, and split larger work. Ask the user before going over the limit. In the description of each pull request, explain what the change assumes, which invariants its types hold, and why it takes this approach.
- Match the sophistication of the approach to the size of the task.
- Check the output or logs to confirm each command did what you expected.
- Follow test-driven development (see the canon-tdd skill).
- Apply the humanizer and stop-slop skills whenever you write or revise prose: skills, documentation, README files, commit messages, and PR descriptions. Keep code, identifiers, commands, and technical meaning unchanged.

## Specifications

- Requirements, design, and tests come only from the specifications: the RFCs and the IETF drafts that `docs/reference/requirements.md` names. Each test cites the section it checks.
- Do not read, copy, or compare behavior against other sender-authentication implementations, and do not cite them in documents, tests, or commit messages.
- Published conformance test data may be used after a license check. The documentation records each source and its license.
- Test messages and DNS names use domains under `.test`, such as `void.test` (ADR 0058). Tests make no network calls; DNS answers come from snapshots. Published conformance data keeps its own names and DNS records (ADR 0039).

## Documentation

- Maintain all documentation under Diátaxis (see the diataxis skill). Each document is exactly one type: tutorial (`docs/tutorials/`), how-to guide (`docs/how-to/`), reference (`docs/reference/`), or explanation (`docs/explanation/`). Do not mix types in one document; link between them instead.
- Write the API reference as doc comments in the crates, not in `docs/` (ADR 0052). An item that isn't implemented yet carries `#[cfg(doc)]`, and the bodies of its functions are `unimplemented!()`. When you implement an item, remove its `#[cfg(doc)]` and keep its doc comment.
- Public items follow the API conventions of ADR 0065: how callers build and read values, which enums are `#[non_exhaustive]`, the error enums, and what a doc comment holds.
- Link to an API item by its GitHub Pages URL with the rustdoc anchor, such as `https://photom.github.io/hallmark/api/hallmark/struct.Check.html#method.answer`. CI checks these links against the rustdoc it builds.
- Record architecture decisions as ADRs in `docs/adr/`, outside the Diátaxis structure (see `docs/adr/README.md`). Do not rewrite an accepted ADR; supersede it with a new one.
- Write test scenarios in `docs/testing/`, outside the Diátaxis structure, as Given/When/Then tables with IDs that link to spec rules (see `docs/testing/README.md`). Add a scenario before writing its test.
- Write diagrams in Mermaid. Do not draw ASCII diagrams.
- Do not write code in documentation, except example code.
- Put example code in tutorials (`docs/tutorials/`) and how-to guides (`docs/how-to/`). The doc comment of an implemented item may also hold examples, which run as doctests. Items under `#[cfg(doc)]` hold none. Reference documents in `docs/` and the README link to the tutorials and how-to guides instead.

## Security and confidentiality

### Restricted files and data

Do not read, modify, or expose the following. If a task requires changing one of them, contact the user first.

- `.env` files
- files under `src/env`
- secrets configuration, such as `*/config/secrets.*`
- private keys and certificates, such as `*/.pem` and `*/.key`
- any file that contains API keys, tokens, or credentials

Tests may use keys generated while the test runs, and the example keys published in the RFCs. Never commit any other private key.

### Security rules

- Do not hardcode secrets such as API keys or passwords. Read them from environment variables managed by a secret manager.
- Never commit sensitive files. Keep `.gitignore` up to date.
- Validate and sanitize all user input to prevent injection attacks such as SQL injection and XSS.
- Keep credentials and secrets out of logs, console output, and error messages.
- DKIM private keys never appear in traces, errors, logs, or `Debug` output.

### Vulnerability detection

- Run the standard static analysis (SAST) tools for the language, for example SonarQube, Snyk, Bandit, or ESLint-plugin-security.
- Scan third-party dependencies automatically and alert on known vulnerabilities (CVEs).
- Design with Zero Trust and least privilege, following cloud security best practices.
- Run the security automation (SAST, DAST, and audit logging) in the CI/CD pipeline.

## Architecture

- Zero Trust: trust no entity, internal or external, by default. Verify everything.
- Least privilege: grant each function only the access it needs.

## Skills

Skills do not load automatically. Invoke one when its domain knowledge is needed:

- `/canon-tdd`: Kent Beck's TDD workflow (red, green, refactor)
- `/rust-master`: Rust conventions, error handling, async code
- `/procedural-rust`: the style, efficiency, type, exception and size rules of ADR 0064, the panic rule of ADR 0004, and how to check and fix code
- `/design-master`: DDD, Clean Architecture, SOLID, system design
- `/docs-first`: documentation-first design with Mermaid diagrams
- `/diataxis`: technical documentation with the Diátaxis framework
- `/external-integration`: reliability patterns for external services, such as DNS over HTTPS
- `/humanizer`: remove AI writing patterns from prose
- `/stop-slop`: quick checks against AI writing patterns
