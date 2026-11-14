# Security policy

## Contents

- [Reporting a vulnerability](#reporting-a-vulnerability)
- [What counts as a vulnerability](#what-counts-as-a-vulnerability)
- [Supported versions](#supported-versions)

## Reporting a vulnerability

Report vulnerabilities privately through [GitHub's private vulnerability reporting](https://github.com/photom/hallmark/security/advisories/new). Do not open a public issue, pull request, or discussion for them.

Include the hallmark version or commit, the check involved (SPF, DKIM, DKIM2, DMARC, or the trace), and the smallest input that shows the problem: the message, the DNS records, the verification time, and the options. Replace real addresses and domains with names under `void.invalid`, and never send a private key that is in use.

## What counts as a vulnerability

hallmark reads messages and DNS answers from strangers, so these are vulnerabilities:
- an input that makes hallmark panic, loop without end, or use memory beyond its configured limits
- a result of `pass` that the specification does not allow, such as a DKIM signature accepted over changed content
- private key material in a trace, an error, a log, or `Debug` output
- a timing difference in signing that leaks information about a private key

A result that differs from the specification in the other direction, such as a valid signature reported as `fail`, is a bug. Report it as an issue.

## Supported versions

hallmark has no release yet. Fixes go to the `main` branch.
