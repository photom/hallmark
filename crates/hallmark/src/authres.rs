//! Authentication-Results header fields (RFC 8601).

use crate::*;

/// The caller's authserv-id (AR-001).
///
/// [`AuthservId::new`] checks it, and the field is private.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuthservId {
    id: Vec<u8>,
}

impl AuthservId {
    /// Makes the caller's authserv-id.
    ///
    /// # Parameters
    ///
    /// - `id`: an authserv-id: a `value` of RFC 2045 §5.1, a token or a
    ///   quoted-string, with the UTF-8 of RFC 6532.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-701 | `id` | Matches `value` of RFC 2045 §5.1, a token or a quoted-string, with the UTF-8 of RFC 6532 | [`authres.authserv_id_invalid`](AuthResError::AuthservIdInvalid) | AR-001, HDR-009, RFC 8601 §2.2 | API-701 |
    /// | VAL-702 | `id` | Each A-label in it converts to a U-label | [`authres.authserv_id_invalid`](AuthResError::AuthservIdInvalid) | AR-005, RFC 8601 §5 | API-702 |
    ///
    /// Requirements: AR-001.
    pub fn new(id: &[u8]) -> Result<AuthservId, AuthResError> {
        unimplemented!()
    }

    /// The authserv-id as [`AuthservId::new`] took it.
    ///
    /// Requirements: AR-001.
    pub fn as_bytes(&self) -> &[u8] {
        unimplemented!()
    }
}

/// An Authentication-Results header field (RFC 8601): writing one from
/// results, reading one, and telling whether one carries the caller's
/// authserv-id.
///
/// [`AuthenticationResults::parse`] builds the value, and the caller reads its
/// fields. Comments don't appear in these types.
///
/// | Rule | Requirements |
/// |---|---|
/// | [`generate`](Self::generate) writes one result for each SPF identity, each DKIM signature, DKIM2 and each DMARC author domain, and maps each typed reason to one result value | AR-006, [ADR 0056](https://github.com/photom/hallmark/blob/main/docs/adr/0056-authentication-results-map-each-typed-reason-to-one-result-value.md) |
/// | [`generate`](Self::generate) builds each `reason` and comment from the typed reasons, so it works at every trace level | AR-002 |
/// | [`generate`](Self::generate) writes `dkim2=none` for a message without DKIM2 fields, when the results hold DKIM2 | AR-004 |
/// | [`generate`](Self::generate) adds a comment that says time checks were off, when they were | TIM-005 |
/// | [`carries_authserv_id`](Self::carries_authserv_id) compares after converting A-labels to U-labels, as RFC 8601 §5 asks for internationalized messages, so a field that names the caller's ID in either form matches | AR-005 |
/// | hallmark deletes no field. The caller decides whether to delete a field that carries its authserv-id | AR-005, GEN-006 |
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct AuthenticationResults {
    /// The authserv-id, without the quotes of a quoted-string (HDR-009).
    pub authserv_id: Vec<u8>,
    /// `authres-version`, as its digits (RFC 8601 §2.2).
    pub version: Option<String>,
    /// Each result, in order. Empty for `none` (RFC 8601 §2.2).
    pub results: Vec<MethodResult>,
}

impl AuthenticationResults {
    /// Writes an Authentication-Results field from the results of a check.
    ///
    /// [Authentication-Results](https://github.com/photom/hallmark/blob/main/docs/reference/design/authentication-results.md)
    /// gives the order of the results, the result value and `reason` for each
    /// typed reason, the properties, the comments, and how values are quoted
    /// and folded.
    ///
    /// Each parameter is valid by its type, so the function has no rule to
    /// check (ARH-008).
    ///
    /// # Parameters
    ///
    /// - `results`: the results of a check.
    /// - `id`: the caller's authserv-id.
    /// - `form`: the form of the value.
    ///
    /// Requirements: AR-001, AR-002, AR-004, AR-006, TIM-005, STR-009.
    pub fn generate(results: &CheckResults, id: &AuthservId, form: FieldForm) -> HeaderFieldOut {
        unimplemented!()
    }

    /// Reads one Authentication-Results field.
    ///
    /// # Parameters
    ///
    /// - `value`: the value of one Authentication-Results field: the octets
    ///   after the colon, with any folds, without the line ending that ends
    ///   the field.
    /// - `limits`: the limits that apply: `header_field_size` and
    ///   `comment_depth`.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-710 | `value` | At most `header_field_size` octets | [`authres.syntax`](AuthResError::Syntax), naming the limit | SAF-002 | API-710 |
    /// | VAL-711 | `value` | Matches `authres-payload` of RFC 8601 §2.2 | [`authres.syntax`](AuthResError::Syntax), with the rule and the offset | HDR-009, PAR-001 | ARH-001 to ARH-007 |
    /// | VAL-712 | `value` | Comments nest at most `comment_depth` levels | [`authres.syntax`](AuthResError::Syntax), naming the limit | HDR-011 | API-712 |
    ///
    /// Requirements: AR-003, HDR-009.
    pub fn parse(value: &[u8], limits: &Limits) -> Result<AuthenticationResults, AuthResError> {
        unimplemented!()
    }

    /// Tells whether one Authentication-Results field carries the caller's
    /// authserv-id.
    ///
    /// A field whose authserv-id doesn't parse can't be matched. The error
    /// says so, and the caller decides what to do with the field (GEN-006).
    ///
    /// # Parameters
    ///
    /// - `value`: the value of one Authentication-Results field: the octets
    ///   after the colon, with any folds, without the line ending that ends
    ///   the field.
    /// - `id`: the caller's authserv-id.
    /// - `limits`: the limits that apply: `header_field_size` and
    ///   `comment_depth`.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-720 | `value` | The rules of VAL-710 to VAL-712, up to the authserv-id | [`authres.syntax`](AuthResError::Syntax) | AR-005 | API-720 |
    ///
    /// Requirements: AR-005.
    pub fn carries_authserv_id(
        value: &[u8],
        id: &AuthservId,
        limits: &Limits,
    ) -> Result<bool, AuthResError> {
        unimplemented!()
    }
}

/// One result of an Authentication-Results field (RFC 8601 §2.2).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct MethodResult {
    /// The method, such as `dkim`.
    pub method: String,
    /// The method version, as its digits.
    pub method_version: Option<String>,
    /// The result value, such as `pass`.
    pub result: String,
    /// The reason's value, without the quotes of a quoted-string (AR-002).
    pub reason: Option<Vec<u8>>,
    /// Each property, in order.
    pub properties: Vec<Property>,
}

/// One property of a result (RFC 8601 §2.2).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Property {
    /// The property type, such as `header`.
    pub ptype: String,
    /// The property, such as `d`.
    pub property: String,
    /// The value, without the quotes of a quoted-string.
    pub value: Vec<u8>,
}

/// The error of [`AuthservId::new`], [`AuthenticationResults::parse`] and
/// [`AuthenticationResults::carries_authserv_id`] (AR-003, AR-005).
///
/// | Code | Variant | Given by | Rules | Requirements |
/// |---|---|---|---|---|
/// | `authres.authserv_id_invalid` | [`AuthservIdInvalid`](Self::AuthservIdInvalid) | [`AuthservId::new`] | VAL-701, VAL-702 | AR-001, AR-005 |
/// | `authres.syntax` | [`Syntax`](Self::Syntax) | [`AuthenticationResults::parse`], [`AuthenticationResults::carries_authserv_id`] | VAL-710 to VAL-712, VAL-720 | HDR-009, PAR-001 |
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AuthResError {
    /// `authres.authserv_id_invalid`: an authserv-id that isn't a `value`, or
    /// holds a bad A-label.
    AuthservIdInvalid,
    /// `authres.syntax`: a field value that doesn't parse, or reaches a limit.
    Syntax(ParseError),
}

impl AuthResError {
    /// The error's code, such as `authres.syntax` (STB-001).
    pub fn code(&self) -> &'static str {
        unimplemented!()
    }
}

impl core::fmt::Display for AuthResError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for AuthResError {}
