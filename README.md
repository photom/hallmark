# hallmark

Email sender authentication for Rust: SPF, DKIM (v1 and DKIM2) and DMARC. hallmark verifies all of them, signs with DKIM and DKIM2, and records a trace that explains every step, so you can see why a message passed or failed. It runs natively and in WebAssembly.

hallmark is at the design stage and has no release yet.

## Repository layout

| Path | Contents |
|---|---|
| `crates/hallmark` | The core library. It does no I/O of its own: the caller supplies DNS answers, the time, and the message as a stream. |
| `crates/hallmark-wasm` | WebAssembly bindings and a DNS-over-HTTPS resolver for the browser demo. Not published. |

## Documentation

- [Requirements](docs/reference/requirements.md): what hallmark must do, with an ID for each requirement
- [Architecture](docs/explanation/architecture.md): how the parts fit together, and why
- [Architecture decision records](docs/adr/README.md)
- [Test scenarios](docs/testing/README.md), written before the tests

## Security

Report vulnerabilities privately, as described in [SECURITY.md](SECURITY.md).

## License

MIT. See [LICENSE](LICENSE).
