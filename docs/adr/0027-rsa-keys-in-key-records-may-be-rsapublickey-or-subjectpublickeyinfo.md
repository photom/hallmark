# ADR 0027: RSA keys in key records may be RSAPublicKey or SubjectPublicKeyInfo

- Status: Accepted
- Date: 2026-11-14
- Affects: the DKIM and DKIM2 key record parsers

## Context

RFC 6376 §3.6.1 says that the `rsa` key type "indicates that an ASN.1 DER-encoded [ITU-X660-1997] RSAPublicKey (see [RFC3447], Sections 3.1 and A.1.1) is being used in the "p=" tag." An RSAPublicKey is a SEQUENCE of the modulus and the public exponent. RFC 8017 §3.1 and Appendix A.1.1 now define it, and erratum 4287 corrects the reference to ITU-T X.690.

The procedure in RFC 6376 Appendix C produces a different structure. Its sample key is a SubjectPublicKeyInfo (RFC 5280 §4.1): an AlgorithmIdentifier for `rsaEncryption`, followed by a BIT STRING that holds the RSAPublicKey. Three errata report this. Erratum 3017 (Held for Document Update) would allow an RSAPublicKey that "MAY be contained in a SubjectPublicKeyInfo". Errata 6674 and 7001 are Reported.

dns-03 settles the question for both DKIM and DKIM2. Its §4.2 says that for `rsa`, "p= is an ASN.1 DER-encoded [ITU-X660-1997] RSAPublicKey ([RFC8017], Sections 3.1 and A.1.1) or a SubjectPublicKeyInfo containing one ([RFC5280], [RFC4055])". Its §6 lists this as an update to RFC 6376 that "resolves errata 3017, 6674 and 7001". RFC 4055 §1.2 says that `rsaEncryption` in a SubjectPublicKeyInfo has parameters that "MUST contain NULL".

## Options considered

- (a) RSAPublicKey only, as RFC 6376 §3.6.1 says.
- (b) SubjectPublicKeyInfo only, as RFC 6376 Appendix C produces.
- (c) Both, as dns-03 §4.2 says.

## Decision

Option (c), for DKIM and DKIM2.

- `p=` decodes from base64 (HDR-007) to DER. The first element of the outer SEQUENCE tells the two forms apart: an INTEGER starts an RSAPublicKey, and a SEQUENCE starts the AlgorithmIdentifier of a SubjectPublicKeyInfo.
- A SubjectPublicKeyInfo must name `rsaEncryption` with NULL parameters (RFC 4055 §1.2), and its BIT STRING must hold an RSAPublicKey with no unused bits. Any other algorithm, such as `id-RSASSA-PSS`, makes the key malformed.
- Bytes after the outer structure, or a structure that DER doesn't allow, make the key malformed: PERMFAIL (key syntax error) for DKIM (RFC 6376 §6.1.2), and PERMERROR "is malformed" for DKIM2 (draft-09 §14.5).
- The key size rules of DKV-003 and DK2-010 apply to the modulus.
- The trace records which form the key used.

## Consequences

Positive:
- hallmark reads keys made with the procedure of RFC 6376 Appendix C, and keys in the form that §3.6.1 names.
- DKIM and DKIM2 share one RSA key parser.

Negative:
- hallmark accepts a form that RFC 6376 §3.6.1 doesn't name, on the strength of an Internet-Draft and a Held erratum.
- A verifier that accepts only one form fails keys in the other form that hallmark accepts.

## References

- [RFC 6376 §3.6.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.6.1), [§6.1.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-6.1.2) and [Appendix C](https://www.rfc-editor.org/rfc/rfc6376.html#appendix-C)
- [RFC 6376 errata 3017](https://www.rfc-editor.org/errata/eid3017), [4287](https://www.rfc-editor.org/errata/eid4287), [6674](https://www.rfc-editor.org/errata/eid6674) and [7001](https://www.rfc-editor.org/errata/eid7001)
- [RFC 8017 §3.1](https://www.rfc-editor.org/rfc/rfc8017.html#section-3.1) and [Appendix A.1.1](https://www.rfc-editor.org/rfc/rfc8017.html#appendix-A.1.1)
- [RFC 5280 §4.1](https://www.rfc-editor.org/rfc/rfc5280.html#section-4.1) and [RFC 4055 §1.2](https://www.rfc-editor.org/rfc/rfc4055.html#section-1.2)
- [draft-ietf-dkim-dkim2-dns-03 §4.2](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-dns-03#section-4.2) and [§6](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-dns-03#section-6)
- [draft-ietf-dkim-dkim2-spec-09 §14.5](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-14.5)
- [requirements.md: DNS records](../reference/requirements.md#dns-records)
