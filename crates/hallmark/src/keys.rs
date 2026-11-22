//! The signer for keys held in memory. The private key types inside it stay
//! crate-private.

use crate::*;

/// The library's signer for keys held in memory, which tests and the demo
/// use (SGN-005,
/// [ADR 0042](https://github.com/photom/hallmark/blob/main/docs/adr/0042-signing-goes-through-a-signer-the-caller-supplies-and-the-caller-can-have-each-signature-verified.md)).
///
/// It implements [`Signer`]. It implements neither `Debug` nor `Clone`, and it
/// is `Send` and `Sync`
/// ([Concurrency](https://github.com/photom/hallmark/blob/main/docs/reference/design/concurrency.md#send-and-sync)).
///
/// | Rule | Requirements |
/// |---|---|
/// | It finds the key by the public key in the request's statement | SGN-003 |
/// | It draws no random number. ML-DSA-44 uses the hedged variant with a random value from the caller: the `random` parameter of [`sign_now`](Self::sign_now), or the [`RandomSource`] through the [`Signer`] trait | SGN-005, IO-001 |
/// | Its keys neither print nor format, and their memory is zeroed when the signer is dropped. No error holds a byte of a key | SEC-001, SEC-002 |
/// | It doesn't verify its own signatures at run time. Tests do | ADR 0042 |
pub struct MemorySigner {
    keys: (),
}

impl MemorySigner {
    /// Makes a signer with no keys.
    ///
    /// Through the [`Signer`] trait, it can't sign ML-DSA-44, which needs a
    /// random value (VAL-412).
    ///
    /// Requirements: SGN-005.
    pub fn new() -> MemorySigner {
        unimplemented!()
    }

    /// Makes a signer with no keys that can sign ML-DSA-44 through the
    /// [`Signer`] trait, with random values from `source`.
    ///
    /// Any `RandomSource` will do, so this function has no rule to check
    /// (SIGN-010).
    ///
    /// # Parameters
    ///
    /// - `source`: the caller's source of random values for ML-DSA-44.
    ///
    /// Requirements: SGN-005, IO-001.
    pub fn with_random(source: Box<dyn RandomSource>) -> MemorySigner {
        unimplemented!()
    }

    /// Adds an RSA private key, and gives its statement.
    ///
    /// # Parameters
    ///
    /// - `der`: an RSAPrivateKey in DER (RFC 8017 Appendix A.1.2). The signer
    ///   copies it, and the caller zeroes its own copy.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-401 | `der` | Decodes as an RSAPrivateKey whose parts agree | [`key.invalid`](KeyError::Invalid) | SGN-005, RFC 8017 Appendix A.1.2 | API-401 |
    /// | VAL-402 | `der` | The modulus has from 1,024 to 16,384 bits | [`key.rsa_size`](KeyError::RsaSize) | DKS-001, RFC 8301 §3.2 | API-402 |
    /// | VAL-403 | `der` | The signer holds no key with the same public key | [`key.duplicate`](KeyError::Duplicate) | SGN-005 | API-403 |
    ///
    /// Requirements: SGN-005, SEC-001.
    pub fn add_rsa_key(&mut self, der: &[u8]) -> Result<SignerStatement, KeyError> {
        unimplemented!()
    }

    /// Adds an Ed25519 private key, and gives its statement.
    ///
    /// The type takes exactly 32 octets, so no other rule applies.
    ///
    /// # Parameters
    ///
    /// - `key`: a private key (RFC 8032 §5.1.5).
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-404 | `key` | The signer holds no key with the same public key | [`key.duplicate`](KeyError::Duplicate) | SGN-005 | API-404 |
    ///
    /// Requirements: SGN-005, SEC-001.
    pub fn add_ed25519_key(&mut self, key: &[u8; 32]) -> Result<SignerStatement, KeyError> {
        unimplemented!()
    }

    /// Adds an ML-DSA-44 key pair made from a seed, and gives its statement.
    /// Available with the `unstable-dkim2` feature.
    ///
    /// # Parameters
    ///
    /// - `seed`: a seed for the ML-DSA-44 key pair.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-405 | `seed` | The signer holds no key with the same public key | [`key.duplicate`](KeyError::Duplicate) | SGN-005 | API-405 |
    ///
    /// Requirements: SGN-005, DK2-009.
    #[cfg(feature = "unstable-dkim2")]
    pub fn add_mldsa44_key(&mut self, seed: &[u8; 32]) -> Result<SignerStatement, KeyError> {
        unimplemented!()
    }

    /// Signs a request at once, with the random value of ML-DSA-44 as a
    /// parameter.
    ///
    /// [`Signer::sign`] does the same through the trait, and takes the random
    /// value from the signer's [`RandomSource`]. A broken rule gives a
    /// [`SignerFailure`] whose description names the code, so the signing
    /// caller can hand it to the signing as it is (SGN-004).
    ///
    /// # Parameters
    ///
    /// - `request`: a signing request from a signing.
    /// - `random`: for `Mldsa44`, the random value of the hedged variant
    ///   (draft-09 §3.4). `None` for the other algorithms.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-410 | `request` | The signer holds the key of the request's statement | `signer.key_unknown`, in a [`SignerFailure`] | SGN-005 | API-410 |
    /// | VAL-411 | `random` | `Some` when the algorithm is `Mldsa44`, and `None` otherwise | `signer.random_mismatch`, in a [`SignerFailure`] | SGN-005 | API-411 |
    ///
    /// Requirements: SGN-005.
    pub fn sign_now(
        &self,
        request: &SigningRequest,
        random: Option<&[u8; 32]>,
    ) -> Result<Signature, SignerFailure> {
        unimplemented!()
    }
}

impl Signer for MemorySigner {
    /// Signs a request, with a future that is ready at once.
    ///
    /// A broken rule gives a [`SignerFailure`] whose description names the
    /// code.
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-410 | `request` | The signer holds the key of the request's statement | `signer.key_unknown`, in a [`SignerFailure`] | SGN-005 | API-410 |
    /// | VAL-412 | The signer | An `Mldsa44` request needs a [`RandomSource`] whose `fill` succeeds | `signer.random_unavailable`, in a [`SignerFailure`] | SGN-005, IO-001 | API-412 |
    async fn sign(&self, request: SigningRequest) -> Result<Signature, SignerFailure> {
        unimplemented!()
    }
}

/// The caller's source of random values, for ML-DSA-44 signing through the
/// [`Signer`] trait (SGN-005, IO-001).
///
/// The trait requires `Send` and `Sync`, so a [`MemorySigner`] that holds one
/// stays `Send` and `Sync`.
pub trait RandomSource: Send + Sync {
    /// Writes 32 random octets into `value`.
    ///
    /// The caller implements it, and a failure stops that signature with
    /// `signer.random_unavailable` (VAL-412).
    fn fill(&self, value: &mut [u8; 32]) -> Result<(), RandomFailure>;
}

/// Why a [`RandomSource`] gave no value (SGN-005).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RandomFailure {
    /// Why the source gave no value.
    pub description: String,
}

impl RandomFailure {
    /// Makes a failure of a [`RandomSource`].
    ///
    /// # Parameters
    ///
    /// - `description`: text for the caller's logs.
    ///
    /// Requirements: SGN-005.
    pub fn new(description: String) -> RandomFailure {
        unimplemented!()
    }
}

/// The error of the methods of [`MemorySigner`] that add a key (SGN-005).
///
/// No error holds a byte of a key (SEC-002).
///
/// | Code | Variant | Given by | Rules | Requirements |
/// |---|---|---|---|---|
/// | `key.invalid` | [`Invalid`](Self::Invalid) | [`MemorySigner::add_rsa_key`] | VAL-401 | SGN-005 |
/// | `key.rsa_size` | [`RsaSize`](Self::RsaSize) | [`MemorySigner::add_rsa_key`] | VAL-402 | DKS-001 |
/// | `key.duplicate` | [`Duplicate`](Self::Duplicate) | [`MemorySigner::add_rsa_key`], [`MemorySigner::add_ed25519_key`], `MemorySigner::add_mldsa44_key` | VAL-403 to VAL-405 | SGN-005 |
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyError {
    /// `key.invalid`: key data that doesn't decode.
    Invalid,
    /// `key.rsa_size`: an RSA key outside 1,024 to 16,384 bits.
    RsaSize,
    /// `key.duplicate`: a key the signer already holds.
    Duplicate,
}

impl KeyError {
    /// The error's code, such as `key.duplicate` (STB-001).
    pub fn code(&self) -> &'static str {
        unimplemented!()
    }
}

impl core::fmt::Display for KeyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for KeyError {}
