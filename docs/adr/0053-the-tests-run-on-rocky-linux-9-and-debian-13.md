# ADR 0053: The tests run on Rocky Linux 9 and Debian 13

- Status: Accepted
- Date: 2026-11-18
- Affects: the test job of CI

## Context

CI ran the tests on two GitHub-hosted runners, `ubuntu-latest` and `macos-latest`. The Ubuntu jobs started within seconds, but the macOS test job could wait many minutes for a runner. On 2026-11-18, the macOS test job waited in three of the four CI runs, for 5, 9 and 15 minutes, and each was cancelled before a runner picked it up. In the run that waited 9 minutes, the other jobs had all finished within 40 seconds. The tests themselves take seconds.

The core opens no socket and no file, reads no clock and draws no random number (IO-001), so its behavior depends on Rust and the target, not on the operating system. The WebAssembly job already builds it for a second target.

hallmark-milter runs inside a Sendmail or Postfix MTA on a Linux server. The project targets the two families of distributions that such servers run, RHEL and Debian, and Rocky Linux 9 and Debian 13 stand for them. GitHub hosts Ubuntu, macOS and Windows runners, and a job on an Ubuntu runner can run inside a container image of another distribution.

## Options considered

- (a) Keep the tests on Ubuntu and macOS runners.
- (b) Run the tests on the Ubuntu runner alone.
- (c) Run the tests in Rocky Linux 9 and Debian 13 containers on Ubuntu runners.
- (d) Run the tests on self-hosted runners.

## Decision

Option (c).

- The test job runs `cargo test` with every feature and with none, in `rockylinux/rockylinux:9` and in `debian:trixie`, each in a container on an `ubuntu-latest` runner.
- Inside each container, the job installs gcc, git and rustup, and rustup installs the toolchain that `rust-toolchain.toml` names.
- The other jobs stay on `ubuntu-latest` itself: formatting, Clippy, the WebAssembly build, the MSRV check, rustdoc, the dependency check, CodeQL and the Pages build. They check tools and the code, and the distribution doesn't change their outcome.
- CI runs no test on macOS or Windows.

Option (a) keeps the wait for macOS runners. Option (b) tests a distribution of the Debian family alone, and none of the RHEL family. Option (d) needs machines to maintain.

## Consequences

Positive:
- No CI run waits for a macOS runner.
- The tests run on the userland of both families that the project targets.

Negative:
- A problem that only macOS or Windows shows goes unnoticed in CI.
- Each test job installs its toolchain inside the container, which takes time that a hosted runner's preinstalled Rust doesn't.
- The image tags follow their distributions, so an update of an image can change a run without a change in the repository.

## References

- [requirements.md: Quality and process](../reference/requirements.md#quality-and-process)
- [ADR 0036](0036-the-minimum-supported-rust-version-starts-at-1-88-and-rises-when-a-dependency-needs-it.md), the MSRV check that stays on the Ubuntu runner
