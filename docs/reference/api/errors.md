# Errors

Each kind of failure and where it goes, the error types, the parts of every error, and every error code with the APIs and rules that give it.

## Contents

- [Error model](#error-model)
- [Error types](#error-types)
- [What an error carries](#what-an-error-carries)
- [Error codes](#error-codes)
- [Signing errors in the results](#signing-errors-in-the-results)
- [Parse errors](#parse-errors)
- [Stability](#stability)

## Error model

```mermaid
flowchart TD
    fail["Something goes wrong"] --> from{"Where it comes from"}
    from -->|"the caller breaks a rule of an API"| ret["An error returned from the call. The value called stays as it was"]
    from -->|"the signer fails, or its signature is wrong"| sign["That signing stops, with a SigningError in the results"]
    from -->|"the message, the envelope or a DNS record"| proto["The protocol's result, and a trace step"]
    from -->|"a limit on a check's work"| lim["The protocol's error result, and a trace step that names the limit"]
    from -->|"the trace limit"| cut["The trace is cut short, and the results stay the same"]
    from -->|"the caller's reader or writer"| stream["A StreamError that names the stream"]
```

| Rule | Requirements |
|---|---|
| An error returned from a call never becomes a protocol result, and a protocol result never comes back as an error | CHK-004, INV-010 |
| A call that returns an error changes nothing in the value called | CHK-004, INV-003 |
| Input from a sender or a DNS publisher never gives a returned error. It gives a protocol result | GEN-005, SAF-001 |
| No input makes hallmark panic | SAF-001 |

## Error types

| Type | Returned by | Code prefix | Requirements |
|---|---|---|---|
| `CheckError` | Every method of `Check` except `new` | `check.` | CHK-004 |
| `OptionsError` | `Check::new`, `VerificationTime::from_unix_seconds` | `options.` | GEN-001 |
| `LimitsError` | The setters of `Limits` and `DocumentLimits` | `limits.` | PAR-004 |
| `ResultsError` | `HeaderFieldOut::read_value` | `results.` | STR-002 |
| `SigningError` | Held in `Signing::Stopped` in the results | `signing.` | SGN-004 |
| `KeyError` | `MemorySigner::add_rsa_key`, `add_ed25519_key`, `add_mldsa44_key` | `key.` | SGN-005 |
| `SignerFailure` | `MemorySigner::sign`, and `Signer` for `MemorySigner` | `signer.` | SGN-004, SGN-005 |
| `TraceError` | `sort_trace`, `Event::parse` | `trace.` | TRC-007 |
| `CanonicalizerError` | Every method of `Canonicalizer` | `canonicalizer.` | STR-012 |
| `AuthResError` | `AuthservId::new`, `AuthenticationResults::parse`, `carries_authserv_id` | `authres.` | AR-003, AR-005 |
| `DriveError` | `drive` | `driver.` | CHK-005 |
| `StreamError` | `io::read_message`, `io::write_trace` | `stream.` | STR-006 |
| `DocumentError` | `Snapshot::parse`, `Snapshot::from_trace`, `PastedRecords::parse` | `document.` | DNS-004, DNS-010 |

## What an error carries

| Part | Content | Requirements |
|---|---|---|
| Code | One of the codes below, which a program matches on | STB-001 |
| Details | The values the code needs: the rule of the grammar and the offset, the limit and its value, the line, the request ID, or the error underneath | PAR-001 |
| English text | `Display` writes it from the code and the details | TRC-011 |

| Rule | Requirements |
|---|---|
| No error, in its text, its details or its `Debug` output, holds a byte of a private key | SEC-002, THR-012 |
| An error names input by its role and offset, such as the header field and the byte, and doesn't copy body content | PAR-001, TRC-010 |
| `SignerFailure` keeps the signer's description only in the error. The trace says that the signer failed, without the description | SGN-004, SEC-002 |

## Error codes

| Code | Meaning | APIs | Rules | Requirements |
|---|---|---|---|---|
| `check.out_of_order` | Message input in a state that doesn't accept it | `envelope`, `header_field`, `end_of_header_fields`, `header_section`, `whole_message`, `body_chunk`, `end_of_message` | VAL-101, VAL-107 to VAL-109, VAL-111 to VAL-113 | CHK-004 |
| `check.envelope_incomplete` | The envelope lacks a part that a kind needs | `envelope` | VAL-102 to VAL-105 | GEN-002 |
| `check.rcpt_to_limit` | More RCPT TO paths than `rcpt_to` | `envelope` | VAL-106 | SAF-004 |
| `check.body_in_header_section` | An empty line inside the raw header section | `header_section` | VAL-110 | HDR-001 |
| `check.unknown_question` | No question with that ID was handed out | `answer`, `answer_suffix` | VAL-120, VAL-130 | CHK-004 |
| `check.unknown_request` | No signing request with that ID was handed out | `answer_signing` | VAL-140 | CHK-004 |
| `check.already_answered` | The request has its answer | `answer`, `answer_suffix`, `answer_signing` | VAL-121, VAL-131, VAL-141 | CHK-004 |
| `check.answer_type_mismatch` | A record of another type than the question's | `answer` | VAL-122 | DNS-006 |
| `check.answer_malformed` | An empty list of records, or a TXT string over 255 octets | `answer` | VAL-123, VAL-124 | DNS-002, DNS-006 |
| `check.suffix_out_of_range` | A suffix of 0 labels, or of more labels than the domain | `answer_suffix` | VAL-132 | DMC-011 |
| `check.list_name_invalid` | A list name that isn't UTF-8 of 1 to 256 octets | `answer_suffix` | VAL-133 | DMC-010 |
| `check.buffer_empty` | A buffer of 0 bytes | `read_trace` | VAL-150 | STR-002 |
| `check.not_finished` | The check isn't Finished. Carries `Waiting` | `take_results` | VAL-160 | CHK-002 |
| `check.results_taken` | The results were taken before | `take_results` | VAL-161 | CHK-002 |
| `options.time_out_of_range` | A time over 999,999,999,999 seconds | `VerificationTime::from_unix_seconds` | VAL-201 | TIM-001 |
| `options.no_kind` | No kind of check | `Check::new` | VAL-210 | GEN-001 |
| `options.source_kind_missing` | DMARC takes an identifier from a kind the check doesn't run | `Check::new` | VAL-211, VAL-212 | DMC-005 |
| `options.earlier_result_invalid` | An earlier SPF result for the HELO identity | `Check::new` | VAL-213 | DMC-005 |
| `options.message_id_invalid` | A message ID that isn't UTF-8 of 1 to 256 octets | `Check::new` | VAL-214 | TRC-014 |
| `options.receiving_host_invalid` | A receiving host with no lookup form | `Check::new` | VAL-215 | REC-014 |
| `options.signing_mismatch` | A signing kind without its signing options, or the reverse | `Check::new` | VAL-216, VAL-217 | GEN-001 |
| `options.signing_domain_invalid` | A `d=` or `nd=` domain with no lookup form | `Check::new` | VAL-220, VAL-230, VAL-240 | ADS-002 |
| `options.selector_invalid` | A selector that breaks RFC 6376 §3.1 or makes a key name too long | `Check::new` | VAL-221, VAL-232 | DNS-008 |
| `options.selector_repeated` | One selector twice in a DKIM2 signature | `Check::new` | VAL-233 | DK2-009 |
| `options.algorithm_not_allowed` | `mldsa44` for DKIM | `Check::new` | VAL-222 | DKS-001 |
| `options.public_key_invalid` | A stated public key that doesn't decode for its algorithm | `Check::new` | VAL-223, VAL-234 | SGN-003 |
| `options.rsa_key_too_small` | A stated RSA key under 1,024 bits | `Check::new` | VAL-224, VAL-235 | SGN-003 |
| `options.rsa_key_too_large` | A stated RSA key over `rsa_key_bits` | `Check::new` | VAL-225, VAL-235 | SAF-002 |
| `options.rsa_exponent_not_65537` | A stated DKIM2 RSA key whose exponent isn't 65537 | `Check::new` | VAL-236 | SGN-003 |
| `options.header_name_invalid` | A name to sign that isn't a field name | `Check::new` | VAL-226 | DKS-003 |
| `options.from_not_signed` | A list of fields to sign without From | `Check::new` | VAL-227 | DKS-003 |
| `options.expiration_invalid` | An expiration of 0, or one that makes `x=` longer than twelve digits | `Check::new` | VAL-228 | DKS-004 |
| `options.no_signer` | DKIM2 signing with no group, or an empty group | `Check::new` | VAL-231 | DK2-009 |
| `options.dkim2_envelope_invalid` | `nd=` together with `mf=` or `rt=`, or neither | `Check::new` | VAL-237 | DK2-003 |
| `options.path_invalid` | An outgoing path that isn't a Reverse-path or a Forward-path | `Check::new` | VAL-238 | ADS-020, ADS-021 |
| `options.rcpt_to_limit` | More outgoing RCPT TO paths than `rcpt_to` | `Check::new` | VAL-239 | SAF-004 |
| `options.nonce_invalid` | An `n=` over 64 characters, or with a character outside printable ASCII or a `;` | `Check::new` | VAL-241 | DK2-003 |
| `options.revision_invalid` | A revision whose line ranges overlap or go backwards, or with both a body description and a null body recipe | `Check::new` | VAL-242 | DK2-014 |
| `limits.out_of_range` | A limit outside its range. Names the limit | `Limits` and `DocumentLimits` setters | VAL-202, VAL-204 | PAR-004 |
| `limits.conflict` | `header_field_size` over `header_section_size` | `Limits` setters | VAL-203 | PAR-004 |
| `results.buffer_empty` | A buffer of 0 bytes | `HeaderFieldOut::read_value` | VAL-301 | STR-002 |
| `results.offset_out_of_range` | An offset past the end of the value | `HeaderFieldOut::read_value` | VAL-302 | STR-002 |
| `key.invalid` | Key data that doesn't decode | `add_rsa_key` | VAL-401 | SGN-005 |
| `key.rsa_size` | An RSA key outside 1,024 to 16,384 bits | `add_rsa_key` | VAL-402 | DKS-001 |
| `key.duplicate` | A key the signer already holds | `add_rsa_key`, `add_ed25519_key`, `add_mldsa44_key` | VAL-403 to VAL-405 | SGN-005 |
| `signer.key_unknown` | No key for the request's statement | `MemorySigner::sign`, `Signer` | VAL-410 | SGN-005 |
| `signer.random_mismatch` | A random value missing for `mldsa44`, or given for another algorithm | `MemorySigner::sign` | VAL-411 | SGN-005 |
| `signer.random_unavailable` | An `mldsa44` request through the trait with no `RandomSource`, or one that failed | `Signer` | VAL-412 | SGN-005 |
| `trace.too_large` | A trace over `document_size` | `sort_trace` | VAL-501 | SAF-004 |
| `trace.record_malformed` | Bytes that aren't RS, one JSON text and LF. Gives the offset | `sort_trace`, `Event::parse` | VAL-502, VAL-510 | TRC-007 |
| `trace.event_malformed` | A record that isn't an event of the format | `sort_trace`, `Event::parse` | VAL-503, VAL-505, VAL-511, VAL-512 | TRC-007 |
| `trace.too_deep` | An event nested deeper than `document_depth` | `sort_trace`, `Event::parse` | VAL-503, VAL-511 | PAR-005 |
| `trace.version_unknown` | An event of a major version other than 1 | `sort_trace`, `Event::parse` | VAL-504, VAL-511 | TRC-007 |
| `trace.position_repeated` | Two events at one position | `sort_trace` | VAL-506 | TRC-012 |
| `canonicalizer.length_not_allowed` | An `l=` length in `dkim2` mode | `Canonicalizer::new` | VAL-601 | STR-012 |
| `canonicalizer.rebuild_not_allowed` | A rebuild outside `dkim2` mode | `Canonicalizer::new` | VAL-602 | STR-012 |
| `canonicalizer.instance_invalid` | A Message-Instance value that doesn't parse, or too many | `Canonicalizer::new` | VAL-603 | DK2-006 |
| `canonicalizer.instance_missing` | The version asked for, or an instance on the way to it, is missing | `Canonicalizer::new` | VAL-604 | DK2-006 |
| `canonicalizer.null_recipe` | A null body recipe on the way to the version asked for | `Canonicalizer::new` | VAL-605 | DK2-007 |
| `canonicalizer.recipe_invalid` | A recipe that breaks the order of draft-09 §5.2, a recipe limit, or the end of the body | `Canonicalizer::new`, `finish` | VAL-606, VAL-616 | DK2-007 |
| `canonicalizer.ended` | A chunk after `finish` | `canonicalize` | VAL-610 | STR-012 |
| `canonicalizer.buffer_empty` | An output buffer of 0 bytes | `canonicalize`, `finish` | VAL-611, VAL-615 | STR-003 |
| `authres.authserv_id_invalid` | An authserv-id that isn't a `value`, or holds a bad A-label | `AuthservId::new` | VAL-701, VAL-702 | AR-001, AR-005 |
| `authres.syntax` | A field value that doesn't parse, or reaches a limit. Gives the rule and the offset | `parse`, `carries_authserv_id` | VAL-710 to VAL-712, VAL-720 | HDR-009, PAR-001 |
| `driver.in_flight_out_of_range` | A count of pending futures outside 1 to 1,024 | `drive` | VAL-801 | CHK-005 |
| `driver.suffix_source_missing` | No suffix source for DMARC in RFC 7489 mode | `drive` | VAL-802 | DMC-010 |
| `driver.signer_missing` | No signer for a signing kind | `drive` | VAL-803 | SGN-001 |
| `driver.answer_refused` | The check refused an answer from a future. Carries the `CheckError` | `drive` | VAL-804 | CHK-004 |
| `stream.buffer_empty` | A buffer of 0 bytes | `io::read_message`, `io::write_trace` | VAL-810, VAL-820 | STR-004 |
| `stream.check_refused` | The check refused the message input. Carries the `CheckError` | `io::read_message` | VAL-811 | CHK-004 |
| `stream.source_failed` | The reader gave an error. Names its kind | `io::read_message` | VAL-812 | STR-006 |
| `stream.sink_failed` | The writer gave an error. Names its kind | `io::write_trace` | VAL-821 | STR-006 |
| `document.too_large` | A document or pasted text over its limit | `Snapshot::parse`, `PastedRecords::parse` | VAL-901, VAL-920 | SAF-004 |
| `document.syntax` | Bytes that aren't one I-JSON object. Gives the offset | `Snapshot::parse` | VAL-902 | PAR-001 |
| `document.too_deep` | A document nested deeper than `document_depth` | `Snapshot::parse` | VAL-903 | PAR-005 |
| `document.version_unknown` | A document of a major version other than 1 | `Snapshot::parse` | VAL-904 | TRC-007 |
| `document.type_mismatch` | A `type` other than `snapshot` | `Snapshot::parse` | VAL-905 | DNS-004 |
| `document.member_invalid` | A member missing, of the wrong shape or out of range. Names the member | `Snapshot::parse` | VAL-906, VAL-908 | DNS-004 |
| `document.order` | Answers out of order, or one type and name twice | `Snapshot::parse` | VAL-907 | DNS-004 |
| `document.trace_invalid` | A trace that breaks a rule of `sort_trace`. Carries the `TraceError` | `Snapshot::from_trace` | VAL-910 | TRC-007 |
| `document.trace_incomplete` | A trace below the full level, or cut short | `Snapshot::from_trace` | VAL-911 | TRC-008 |
| `document.trace_inconsistent` | Two steps record different answers to one question | `Snapshot::from_trace` | VAL-912 | DNS-007 |
| `document.line_invalid` | A pasted line that isn't a record of RFC 1035 §5.1 that hallmark reads. Gives the line and the offset | `PastedRecords::parse` | VAL-921 to VAL-923 | DNS-010 |

## Signing errors in the results

A `SigningError` stops one kind of signing. The other kinds still finish, and no header field of the stopped kind comes out (SGN-004).

| Code | Meaning | Raised at | Rules | Requirements | Scenario |
|---|---|---|---|---|---|
| `signing.signer_failed` | The signer answered with a `SignerFailure`. Names the signer and keeps its description | `answer_signing` | | SGN-004 | SIGN-008 |
| `signing.wrong_size` | The signature doesn't have the size its algorithm and key give. Names the signer and the size | `answer_signing` | VAL-142 | SGN-004 | SIGN-007 |
| `signing.not_verified` | The signature doesn't verify with the stated key. Names the signer | `answer_signing`, with `verify_signatures` on | VAL-143 | SGN-006 | SIGN-016 |
| `signing.header_limit` | The header section reached a header limit | The header section ends | | SAF-002 | SIGN-027 |
| `signing.exploded_first_signature` | The `exploded` flag on a message with no DKIM2-Signature yet | The header section ends | | draft-09 §12.5 | SIGN-026 |
| `signing.revision_mismatch` | The earlier version rebuilt from the revision has hashes other than those in the highest Message-Instance | The message ends | | DK2-014 | SIGN-022, SIGN-023 |

## Parse errors

A `ParseError` names the rule of the grammar that failed, the byte offset and the input that the parser read (PAR-001). It sits inside `authres.syntax`, `document.syntax`, `document.line_invalid` and `canonicalizer.instance_invalid`. Inside a check, parse errors of the message, the envelope and DNS records go into the trace and the results, never into a returned error (HDR-010).

## Stability

| Rule | Requirements |
|---|---|
| A code keeps its meaning within a major version of the crate. A minor version can add codes, and the error enums are non-exhaustive | STB-001 |
| The codes of DKIM2 features can change in any release while `unstable-dkim2` lasts | DK2-005 |
| Error codes are separate from trace codes, and the trace format's version doesn't cover them | STB-001 |
