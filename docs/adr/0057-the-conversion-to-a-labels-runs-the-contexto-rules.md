# ADR 0057: The conversion to A-labels runs the CONTEXTO rules

- Status: Accepted
- Date: 2026-11-21
- Affects: ADS-002, the `idna` module, `crates/hallmark-tables`, the rule keywords of the trace, and the domain name scenarios

## Context

RFC 5891 §5.4 has lookup reject a label that holds a CONTEXTO code point "for which no such rule appears in the table of rules". It requires no more: lookup applications "are not required to test contextual rules for CONTEXTO characters, only to verify that a rule is defined (although they MAY make such tests to provide better protection or give better information to the user)". The registry in RFC 5892 Appendix A marks each CONTEXTO rule with "Lookup: False".

Every CONTEXTO code point comes from the exceptions of RFC 5892 §2.6: U+00B7 MIDDLE DOT, U+0375 GREEK LOWER NUMERAL SIGN, U+05F3 and U+05F4 HEBREW PUNCTUATION GERESH and GERSHAYIM, U+30FB KATAKANA MIDDLE DOT, and the digits U+0660 to U+0669 and U+06F0 to U+06F9. Appendix A.3 to A.9 gives each one a rule, so the check that §5.4 requires never fails.

hallmark's trace names the rule that a domain name broke (ADS-002), and [ADR 0012](0012-hallmark-converts-domain-names-to-a-labels-with-its-own-rfc-5891-code.md) already runs the CONTEXTJ rules of Appendix A.1 and A.2.

## Options considered

- (a) Run the CONTEXTO rules of RFC 5892 Appendix A.3 to A.9, and reject a label that breaks one.
- (b) Only check that each CONTEXTO code point has a rule, which always holds.

## Decision

Option (a).

- The conversion runs the rule of each CONTEXTO code point in a label, as RFC 5892 Appendix A.3 to A.9 states it, and a label that breaks one gets no lookup form.
- The trace names the rule `contexto`, with the label and the code point ([Trace codes](../reference/trace-codes.md#domain-name-rules)).
- The generator in `crates/hallmark-tables` extracts the Script property values that the rules read: Greek, Hebrew, Hiragana, Katakana and Han.

Option (b) looks up a name such as `a·b.invalid`, whose MIDDLE DOT isn't between two `l` characters. A registry can't hold that name, and the trace could only say that the lookup found nothing.

## Consequences

Positive:
- A name that no registry could have registered gets no DNS question, and the trace says which rule it broke.
- CONTEXTJ and CONTEXTO code points get the same treatment.

Negative:
- hallmark rejects names that RFC 5891 §5.4 lets lookup pass. A resolver that follows only the MUST of §5.4 looks such a name up, and hallmark doesn't.
- The tables grow by the Script values of five scripts, which the generator reads from the Unicode Character Database files of version 12.0.0 ([ADR 0055](0055-the-idna-tables-use-unicode-12-0-0.md)).

## References

- [RFC 5891 §5.4](https://www.rfc-editor.org/rfc/rfc5891.html#section-5.4)
- [RFC 5892 §2.6 and Appendix A](https://www.rfc-editor.org/rfc/rfc5892.html#appendix-A)
- [ADR 0012](0012-hallmark-converts-domain-names-to-a-labels-with-its-own-rfc-5891-code.md), hallmark's own RFC 5891 code
- [ADR 0055](0055-the-idna-tables-use-unicode-12-0-0.md), the Unicode version of the tables
- [requirements.md: Addresses and domain names](../reference/requirements.md#addresses-and-domain-names)
