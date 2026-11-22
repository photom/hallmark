//! The values that pass between a signing and the signing caller's signer.

use crate::*;

/// A signature algorithm.
///
/// `Mldsa44` exists only with the `unstable-dkim2` feature, so the enum is
/// `#[non_exhaustive]`: a crate that turns the feature on can't break a
/// `match` elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Algorithm {
    /// `rsa-sha256` (DKS-001).
    RsaSha256,
    /// `ed25519-sha256` (DKS-001, RFC 8463).
    Ed25519Sha256,
    /// `mldsa44` (DK2-009). Available with the `unstable-dkim2` feature.
    #[cfg(feature = "unstable-dkim2")]
    Mldsa44,
}

/// What a signer states before it signs: its algorithm and public key
/// (SGN-003).
///
/// The signing caller gets it from its signer, or from [`MemorySigner`], and
/// puts it in [`DkimSigning`] or a DKIM2 signer. [`Signing::new`] checks it
/// (VAL-222 to VAL-225, VAL-234 to VAL-236).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SignerStatement {
    /// The signer's algorithm (SGN-003).
    pub algorithm: Algorithm,
    /// The signer's public key, in the form that [`SignerStatement::new`]
    /// takes (SGN-003).
    pub public_key: Vec<u8>,
}

impl SignerStatement {
    /// Makes the statement of a signer that the signing caller supplies.
    ///
    /// # Parameters
    ///
    /// - `algorithm`: the algorithm the signer signs with.
    /// - `public_key`: the public key. For RSA, an RSAPublicKey or a
    ///   SubjectPublicKeyInfo that holds one, in DER. For Ed25519, 32 octets.
    ///   For ML-DSA-44, 1,312 octets.
    ///
    /// Building a statement checks nothing. [`Signing::new`] checks it.
    ///
    /// Requirements: SGN-003.
    pub fn new(algorithm: Algorithm, public_key: Vec<u8>) -> SignerStatement {
        unimplemented!()
    }
}

/// The kind of signature a signing request is for (SGN-002).
///
/// `Dkim2` exists only with the `unstable-dkim2` feature, so the enum is
/// `#[non_exhaustive]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SigningKind {
    /// A DKIM-Signature field.
    Dkim,
    /// A DKIM2-Signature field. Available with the `unstable-dkim2` feature.
    #[cfg(feature = "unstable-dkim2")]
    Dkim2,
}

/// A signature that a signing needs from the signing caller's signer
/// (SGN-002).
///
/// [`Signing::take_requests`] gives it, and the signing caller hands the
/// signer's answer to [`Signing::answer`].
/// [Lifecycle](https://github.com/photom/hallmark/blob/main/docs/reference/design/lifecycle.md#signing)
/// gives the states of one signature. The signing builds the request, and the
/// signing caller reads its fields.
///
/// The data to sign, and the size of the signature that comes back, depend on
/// the algorithm:
///
/// | Algorithm | `data` | Signature size | Requirements |
/// |---|---|---|---|
/// | `RsaSha256` | The SHA-256 hash, for RSASSA-PKCS1-v1_5 | The modulus size in octets | SGN-002, RFC 8017 §8.2.1 |
/// | `Ed25519Sha256` | The SHA-256 hash, which Ed25519 takes as its message | 64 octets | SGN-002, RFC 8032 §5.1.6 |
/// | `Mldsa44` | The whole signing input, unhashed, signed with the context string `DKIM2` | 2,420 octets | SGN-002, draft-09 §3.4 |
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SigningRequest {
    /// Names the request in [`Signing::answer`] (CHK-004).
    pub id: RequestId,
    /// The kind of signature (SGN-002).
    pub kind: SigningKind,
    /// The statement the request was made for (SGN-003).
    pub statement: SignerStatement,
    /// `d=`, in lookup form (SGN-002).
    pub domain: String,
    /// `s=` (SGN-002).
    pub selector: Vec<u8>,
    /// What the signer signs, which depends on the algorithm (SGN-002).
    pub data: Vec<u8>,
}

/// The ID of a signing request, unique within its signing (CHK-004).
///
/// The signing caller gets IDs only from requests: the type has no public
/// field or constructor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RequestId(u64);

/// A signature that a signer made, for [`Signing::answer`], which checks it
/// (VAL-140 to VAL-143, SGN-004).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Signature {
    /// The signature value, before base64 (SGN-004).
    pub bytes: Vec<u8>,
}

impl Signature {
    /// Makes a signature to hand to [`Signing::answer`].
    ///
    /// # Parameters
    ///
    /// - `bytes`: the signature value, before base64.
    ///
    /// Requirements: SGN-004.
    pub fn new(bytes: Vec<u8>) -> Signature {
        unimplemented!()
    }
}

/// Why a signer couldn't sign (SGN-004).
///
/// The signing caller hands it to [`Signing::answer`], and signing stops with
/// [`SignatureError::SignerFailed`] in the results. The signing keeps the
/// description only in that error, and the trace says that the signer failed,
/// without the description (SEC-002). [`MemorySigner`] gives one for a broken
/// rule, whose description names the code: `signer.key_unknown`,
/// `signer.random_mismatch` or `signer.random_unavailable`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SignerFailure {
    /// Why the signer couldn't sign (SGN-004).
    pub description: String,
}

impl SignerFailure {
    /// Makes a failure to hand to [`Signing::answer`].
    ///
    /// # Parameters
    ///
    /// - `description`: text for the signing caller's logs. The signing keeps
    ///   it out of the trace.
    ///
    /// Requirements: SGN-004.
    pub fn new(description: String) -> SignerFailure {
        unimplemented!()
    }
}
