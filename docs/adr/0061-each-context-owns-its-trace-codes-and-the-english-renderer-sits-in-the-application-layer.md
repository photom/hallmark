# ADR 0061: Each context owns its trace codes, and the English renderer sits in the application layer

- Status: Accepted
- Date: 2026-11-22
- Affects: the `trace` module, a new application module `render`, the modules that record trace steps, and [Modules](../reference/design/modules.md)

## Context

Each trace step carries a code, and a person reads text rendered from the code and its parameters (TRC-011). [Trace codes](../reference/trace-codes.md) lists every code with its English text ([ADR 0034](0034-trace-step-codes-are-strings-listed-in-a-reference-page.md)). The first part of a code names its area: `trace`, `spf`, `dkim`, `dkim2`, `dmarc`, `dns`, `header` or `body`.

The public method `Event::render_english` renders any event, including one read back from trace bytes, so it needs the English text of every code. `Event` belongs to the `trace` module in the kernel, and the kernel uses no layer above it ([ADR 0059](0059-the-modules-of-the-hallmark-crate-are-bounded-contexts-in-five-layers.md)).

## Options considered

- (a) Each context declares its own codes and their English texts, and the renderer sits in the application layer, which knows every context.
- (b) One table of all codes and English texts in the `trace` module.

## Decision

Option (a).

| Area | Module that owns its codes |
|---|---|
| `trace` | `trace` |
| `spf` | `spf` |
| `dkim` | `dkim` |
| `dkim2` | `dkim2` |
| `dmarc` | `dmarc` |
| `dns` | `dns` |
| `header` | `header` |
| `body` | `canonical` |

- Each module in the table declares its codes as a crate-private enum in a `codes` submodule, with the text of each code and its English template. The enum is visible only inside the module's own tree, so no other module can record its codes.
- The kernel declares a crate-private trait, `Code`, that gives the text of a code. Each enum implements it, and the recorder takes any value that implements it, so the kernel depends on no context.
- Each module in the table also gives the crate a function that returns the English template of one of its codes, from the code's text.
- The application module `render` holds the `impl Event` block with `render_english`, a public method of `Event`. It picks the module from the first part of the code.

Option (b) needs an edit to the kernel for each new code of any context, and lets any module record any code.

## Consequences

Positive:
- A change to one specification adds or changes codes in one module.
- A context can't record another context's codes by mistake.
- The kernel stays free of the contexts.

Negative:
- A new area means a new line in `render`.
- The codes live in eight modules, and the reference page lists them in one place. The test that compares the page with the crate's codes has to collect them from all eight.

## References

- [ADR 0031](0031-the-code-that-decides-writes-the-trace-as-events-with-tree-positions.md), the trace written by the deciding code
- [ADR 0034](0034-trace-step-codes-are-strings-listed-in-a-reference-page.md), codes as strings in a reference page
- [ADR 0059](0059-the-modules-of-the-hallmark-crate-are-bounded-contexts-in-five-layers.md), the layers and contexts
- [Trace codes](../reference/trace-codes.md)
