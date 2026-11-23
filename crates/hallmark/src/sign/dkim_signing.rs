//! The options of DKIM signing.

use crate::{SignerStatement, Signing, SigningOptions};

/// One DKIM-Signature field to add (DKS-002).
///
/// The signing caller builds it with [`DkimSigning::new`] and sets the other
/// public fields, and [`SigningOptions::dkim`] holds it. [`Signing::new`]
/// checks it (VAL-220 to VAL-228). `t=` comes from [`SigningOptions::time`]
/// (DKS-004), and hallmark never writes `l=`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DkimSigning {
    /// `d=`, as UTF-8 text: A-labels, or U-labels that RFC 8616 §5 allows
    /// (RFC 6376 §3.5).
    pub domain: String,
    /// `s=` (RFC 6376 §3.1).
    pub selector: Vec<u8>,
    /// The signer's algorithm and public key (SGN-003).
    pub statement: SignerStatement,
    /// The names of the fields to sign, from RFC 6376 §5.4.1 (DKS-003).
    /// Default From, Reply-To, Subject, Date, To, Cc, Resent-Date,
    /// Resent-From, Resent-To, Resent-Cc, In-Reply-To, References, List-Id,
    /// List-Help, List-Unsubscribe, List-Subscribe, List-Post, List-Owner and
    /// List-Archive.
    pub headers: Vec<String>,
    /// Sign one more instance of each listed field than the message has,
    /// DKIM-Signature excepted. Default `false` (DKS-003, DKS-006).
    pub oversign: bool,
    /// `c=`. Default `relaxed/relaxed` (DKS-003).
    pub canonicalization: DkimCanonicalization,
    /// Seconds after `t=`: `x=` is `t=` plus this. Default `None`, so no `x=`
    /// (DKS-004).
    pub expiration: Option<u64>,
    /// Whether hallmark verifies the signature from the signer with the
    /// public key of `statement` before it writes the field. Default `false`,
    /// so hallmark checks only the signature's size (SGN-006).
    pub verify: bool,
}

impl DkimSigning {
    /// Makes one DKIM signature to add, with the other fields at their
    /// defaults.
    ///
    /// # Parameters
    ///
    /// - `domain`: `d=`, as UTF-8 text: A-labels, or U-labels that RFC 8616 §5
    ///   allows.
    /// - `selector`: `s=`, as bytes.
    /// - `statement`: the signer's algorithm and public key.
    ///
    /// Building the value checks nothing. [`Signing::new`] checks it.
    ///
    /// Requirements: DKS-002.
    pub fn new(domain: String, selector: Vec<u8>, statement: SignerStatement) -> DkimSigning {
        unimplemented!()
    }
}

/// The `c=` of a DKIM signature: the header and body canonicalizations
/// (RFC 6376 §3.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DkimCanonicalization {
    /// The header canonicalization.
    pub header: CanonAlgorithm,
    /// The body canonicalization.
    pub body: CanonAlgorithm,
}

/// A DKIM canonicalization algorithm (RFC 6376 §3.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanonAlgorithm {
    /// `simple`.
    Simple,
    /// `relaxed`.
    Relaxed,
}
