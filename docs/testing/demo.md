# Test scenarios: Demo

Scenarios for the browser demo, which CI runs in a browser against the built demo on every change (DEM-007). Conventions are in [README.md](README.md).

## Contents

- [Line endings](#line-endings)
- [Values and keys](#values-and-keys)

## Line endings

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| DEMO-001 | browser | A message with CRLF line endings, pasted into the message box | the user starts the analysis | before the analysis, the demo shows a notice that pasting doesn't keep the original line endings. The core gets the message with each line break as CRLF | [DEM-008](../reference/requirements.md#browser-demo), [STR-011](../reference/requirements.md#streaming), [RFC 6376 §5.3](https://www.rfc-editor.org/rfc/rfc6376.html#section-5.3) |
| DEMO-002 | browser | The pasted message of DEMO-001, analyzed | the user saves the case and replays it | the case holds the message as the text area gave it, with `line_endings` set to `lf`, and the replay gives the same result and the same sorted trace | [DEM-008](../reference/requirements.md#browser-demo), [DEM-004](../reference/requirements.md#browser-demo), [Trace format](../reference/trace-format.md#demo-cases) |
| DEMO-003 | browser | An `.eml` file with CRLF line endings | the user loads it and starts the analysis | the demo shows no warning, and the core gets the file's bytes unchanged | [DEM-008](../reference/requirements.md#browser-demo), [STR-011](../reference/requirements.md#streaming) |
| DEMO-004 | browser | An `.eml` file with LF line endings | the user loads it | before the analysis, the demo warns that the file holds an LF without a CR before it, and offers to convert each such LF to CRLF or to analyze the bytes as they are | [DEM-008](../reference/requirements.md#browser-demo), [RFC 5321 §2.3.8](https://www.rfc-editor.org/rfc/rfc5321.html#section-2.3.8) |
| DEMO-005 | browser | The file of DEMO-004 | the user chooses to convert and starts the analysis | the core gets the file with each LF as CRLF, and a saved case has `line_endings` set to `lf` | [DEM-008](../reference/requirements.md#browser-demo), [RFC 6376 §5.3](https://www.rfc-editor.org/rfc/rfc6376.html#section-5.3) |
| DEMO-006 | browser | The file of DEMO-004 | the user chooses to analyze the bytes as they are | the core gets the bytes unchanged, the trace holds the caution of STR-011 for the bare LFs, and a saved case has `line_endings` set to `crlf` | [DEM-008](../reference/requirements.md#browser-demo), [STR-011](../reference/requirements.md#streaming) |
| DEMO-007 | browser | An `.eml` file whose body holds `A\r\r\n`, a bare CR before a CRLF, and no other bare CR or LF | the user loads it and starts the analysis | the demo shows no warning, because every LF has a CR before it. The core gets the bytes unchanged, and the trace holds the caution of STR-011 for the bare CR | [DEM-008](../reference/requirements.md#browser-demo), [STR-011](../reference/requirements.md#streaming) |

## Values and keys

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| DEMO-008 | browser | `From: "<img src=x onerror=alert(1)>" <a@void.test>`, a pasted TXT record whose text is `<script>alert(2)</script>`, and the SPF explanation `<b>x</b>` | the user analyzes the message and opens every trace step | the page shows each value as its literal text, runs no script and adds no element. It shows the explanation as text from a third party | [DEM-010](../reference/requirements.md#browser-demo), [TRC-013](../reference/requirements.md#trace-and-reasoning), [RFC 7208 §6.2](https://www.rfc-editor.org/rfc/rfc7208.html#section-6.2) |
| DEMO-009 | browser | A message that the user signed in the page with a pasted Ed25519 key | the user saves the case | the case holds no byte of the private key, raw or in base64 | [DEM-004](../reference/requirements.md#browser-demo), [DEM-006](../reference/requirements.md#browser-demo), [SEC-002](../reference/requirements.md#private-keys) |
