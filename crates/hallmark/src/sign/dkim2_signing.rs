//! The options of DKIM2 signing. The whole module exists only with the
//! `unstable-dkim2` feature.

use crate::{SignerStatement, Signing, SigningOptions};

/// The DKIM2-Signature field to add, and the Message-Instance field that a
/// Reviser needs (DK2-003, DK2-014). Available with the `unstable-dkim2`
/// feature.
///
/// The signing caller builds it with [`Dkim2Signing::new`] and sets the other
/// public fields, and [`SigningOptions::dkim2`] holds it. [`Signing::new`]
/// checks it (VAL-230 to VAL-242).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Dkim2Signing {
    /// `d=` (draft-09 §10.8).
    pub domain: String,
    /// `s=`: a group passes when all its signers pass, and the field passes
    /// when any group does (draft-09 §10.9, DK2-009).
    pub groups: Vec<Vec<Dkim2Signer>>,
    /// `mf=`: the reverse-path of the outgoing SMTP transaction, with its
    /// angle brackets. Default `None` (draft-09 §10.5).
    pub mail_from: Option<Vec<u8>>,
    /// `rt=`: the forward-paths of the outgoing SMTP transaction, each with
    /// its angle brackets. Default empty (draft-09 §10.6).
    pub rcpt_to: Vec<Vec<u8>>,
    /// `nd=`, which replaces `mf=` and `rt=`. Default `None` (draft-09 §10.7).
    pub next_domain: Option<String>,
    /// `n=`. Default `None` (draft-09 §10.3).
    pub nonce: Option<String>,
    /// `f=`. Default every flag `false` (draft-09 §10.10).
    pub flags: Dkim2Flags,
    /// The changes a Reviser made. Default `None`, so the message is unchanged
    /// (DK2-014).
    pub revision: Option<Revision>,
    /// Whether hallmark verifies each signature from the signers with the
    /// public key of its [`Dkim2Signer`] before it writes the field. Default
    /// `false`, so hallmark checks only each signature's size (SGN-006).
    pub verify: bool,
}

impl Dkim2Signing {
    /// Makes the DKIM2 signature to add, with the other fields at their
    /// defaults.
    ///
    /// # Parameters
    ///
    /// - `domain`: `d=`, as UTF-8 text.
    /// - `groups`: `s=`: each group a list of signers. A group passes when all
    ///   its signers pass, and the field passes when any group does.
    ///
    /// Building the value checks nothing. [`Signing::new`] checks it.
    ///
    /// Requirements: DK2-003.
    pub fn new(domain: String, groups: Vec<Vec<Dkim2Signer>>) -> Dkim2Signing {
        unimplemented!()
    }
}

/// One signer of a DKIM2 group (DK2-009). Available with the
/// `unstable-dkim2` feature.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Dkim2Signer {
    /// The signer's selector (dns-03 §3).
    pub selector: Vec<u8>,
    /// The signer's algorithm and public key (SGN-003).
    pub statement: SignerStatement,
}

impl Dkim2Signer {
    /// Makes one signer of a DKIM2 group.
    ///
    /// # Parameters
    ///
    /// - `selector`: the signer's selector, as bytes.
    /// - `statement`: the signer's algorithm and public key.
    ///
    /// Requirements: DK2-009.
    pub fn new(selector: Vec<u8>, statement: SignerStatement) -> Dkim2Signer {
        unimplemented!()
    }
}

/// The flags of `f=` (draft-09 §10.10). Available with the `unstable-dkim2`
/// feature.
///
/// The default has every flag `false`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub struct Dkim2Flags {
    /// The `feedback` flag.
    pub feedback: bool,
    /// The `feedhere` flag.
    pub feedhere: bool,
    /// The `exploded` flag.
    pub exploded: bool,
}

/// The changes a Reviser made to the message (DK2-014). Available with the
/// `unstable-dkim2` feature.
///
/// hallmark writes the Message-Instance and its recipes from this description
/// (draft-09 §5, §12.1). It never compares two versions of the body, which
/// would need both in memory (STR-001). The default holds no body change and
/// no field change.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct Revision {
    /// The change to the body, or `None` when the body is unchanged
    /// (DK2-014).
    pub body: Option<BodyRevision>,
    /// One entry for each field name whose fields the Reviser changed, added
    /// or removed (DK2-014).
    pub header_fields: Vec<FieldRevision>,
}

/// The change to the body (DK2-014). Available with the `unstable-dkim2`
/// feature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BodyRevision {
    /// The earlier body, from its first line to its last (DK2-014, draft-09
    /// §5.2).
    Earlier(Vec<EarlierLines>),
    /// The earlier body can't be rebuilt, and hallmark writes a null body
    /// recipe (DK2-007, draft-09 §5.2).
    NullRecipe,
}

/// A run of lines of the earlier body (DK2-014). Available with the
/// `unstable-dkim2` feature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EarlierLines {
    /// Lines `first` to `last` of the new body came unchanged from the earlier
    /// body. Lines count from 1, as DK2-008 counts them.
    Unchanged {
        /// The first line of the run, in the new body.
        first: u64,
        /// The last line of the run, in the new body.
        last: u64,
    },
    /// Lines of the earlier body that the new body doesn't hold, each without
    /// its CRLF.
    Replaced(Vec<Vec<u8>>),
}

/// The earlier values of the fields with one name (draft-09 §5.1). Available
/// with the `unstable-dkim2` feature.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct FieldRevision {
    /// The field name.
    pub name: String,
    /// The values of the fields with this name in the earlier header section,
    /// each without its final CRLF, from the bottom of the header section up.
    /// Empty when the earlier header section had none.
    pub earlier_values: Vec<Vec<u8>>,
}
