# ADR 0060: A check fixes its evaluations when created, and each input once it arrives

- Status: Accepted
- Date: 2026-11-22
- Affects: the `check` module, a new kernel module `write_once`, INV-003, INV-016 and INV-017, the [domain model](../reference/design/domain-model.md#the-check), [Lifecycle](../reference/design/lifecycle.md) and [Modules](../reference/design/modules.md)

## Context

A check takes its input over many calls: the envelope, the header fields, the body chunks, the answers and the signatures (CHK-001). It runs only the kinds of check that its options list (GEN-001), and the options never change after it is created (INV-015).

If the check keeps its inputs and its evaluations in fields that any of its methods can assign, the type says nothing about when a field gets its value or whether it can change later. A reader has to go through every method to find out. An `Option` field there means two things at once: the caller didn't ask for this kind of check, or its input hasn't arrived yet.

Two rules already ask for values that are set once. A second envelope, or a second end of the header fields, gives the caller an error and the check stays as it was (CHK-004). A second answer to a question gives an error, and the first answer stays (INV-003).

Each protocol takes different input. SPF reads only the envelope (DNS-005), DKIM reads the header section and body hashes, and DMARC reads the From field, the authenticated identifiers and public suffix answers (DMC-005, DMC-010).

## Options considered

- (a) Separate values with private fields. The set of evaluations is fixed when the check is created, each input sits in a slot that can be set only once, and the check calls each protocol through a typed field of its own.
- (b) Fields of `Check` that its methods assign as input arrives, with `Option` for what hasn't arrived or doesn't run.
- (c) A crate-private trait that every protocol implements, and a list of trait objects in the check.

## Decision

Option (a).

| Value | Holds | Rules |
|---|---|---|
| `Evaluations` | One field for each kind that verifies: SPF for HELO, SPF for MAIL FROM, DKIM, DKIM2 and DMARC | Private fields. `new`, from the options, is the only place a field becomes `Some` or `None`, and `None` means the caller didn't ask for that kind. Accessors give `Option<&T>`. Input methods pass each input to the evaluations that exist and give back what they now need. No method gives `&mut Option<T>` |
| `Signings` | One field for DKIM signing and one for DKIM2 signing | The same rules as `Evaluations` |
| `WriteOnce<T>`, in the kernel module `write_once` | One value or none | `set` stores the value if the slot is empty, and otherwise gives an error and keeps the first value. `get` gives `Option<&T>`. The type has no other method, so nothing can take, replace or change a value once set |
| `Inputs` | A write-once slot for the envelope as the caller passed it, one for the parsed envelope, and one for the header section | The check sets each slot when that input is complete. Header fields that arrive one at a time (STR-008) collect in the check until the end of the header fields, and only the finished header section goes into its slot. A second envelope or a second end of the header fields is the slot's error, which the check gives as the error of CHK-004 |
| An entry of the question table | A write-once slot for its answer | A second answer is the slot's error, and the first answer stays (INV-003). Public suffix questions and signing requests follow the same rule |

- An evaluation that the caller asked for exists from the start. If it still waits for input, such as DKIM before the end of the header fields, it is in the first state of its own state machine. It is never `None` for that reason.
- An input value that hallmark builds, such as the parsed envelope or the header section, has private fields, a constructor that checks it, and only methods that take `&self`. It has no method that takes `&mut self`, not even inside its own module. A value that the caller built, such as `Envelope`, is reached only through the `&` that its slot gives.
- A protocol reads inputs through shared references, such as `&HeaderSection` and `&Answer`. It never owns or changes one.
- A body chunk is borrowed for one call and never stored (STR-007, INV-004).
- A protocol asks for a DNS record by its question type and lookup name, and never sees a `QuestionId`. The question table turns those needs into requests and numbers them (INV-001, INV-002), and passes each answer to the evaluations that asked for it.
- The caller-built public types, such as `Envelope`, `Options` and `DkimSigning`, keep the public fields that [ADR 0054](0054-the-public-api-follows-one-set-of-conventions.md) gives them. The rules above start when the check takes the value.

Option (b) leaves every field open to any method of `Check`, and its `Option` fields mean two things.

Option (c) makes every protocol take the same input calls, whether it reads that input or not, and calls each one through dynamic dispatch. A new protocol would bring inputs of its own, so the shared trait would grow with each one, and every protocol would take the new calls.

## Consequences

Positive:
- The types show the lifecycle. The check's own states follow the input order. Which evaluations exist is fixed when the check is created. Each evaluation's states live in its protocol context.
- The errors of CHK-004 and INV-003 come from the write-once slots, not from checks spread over the methods of `Check`.
- An evaluation can't read an input that another one changed, because no one can change an input.

Negative:
- A new protocol means a new field in `Evaluations` or `Signings`, a line in `new`, an accessor and a case in each input method that it reads. The compiler doesn't point at the input methods, because each one names only the fields it calls.
- `WriteOnce` is a type of hallmark's own. `std::cell::OnceCell` is close, but its `take` and `get_mut` change a value that is already set.

## References

- [ADR 0003](0003-rust-code-is-written-in-a-procedural-style.md), the procedural style
- [ADR 0047](0047-one-check-type-runs-any-set-of-checks-and-shares-their-dns-answers.md), one check type for any set of checks
- [ADR 0054](0054-the-public-api-follows-one-set-of-conventions.md), the public API conventions
- [ADR 0059](0059-the-modules-of-the-hallmark-crate-are-bounded-contexts-in-five-layers.md), the layers and contexts
- [requirements.md: Running a check](../reference/requirements.md#running-a-check)
