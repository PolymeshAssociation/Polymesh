use parity_scale_codec::{Decode, Encode};

/// Identity permissions.
/// # TODO
/// 2. Review documents:
///     - [MESH-235](https://polymath.atlassian.net/browse/MESH-235)
///     - [Polymesh: permissions/Permissions](https://docs.google.com/document/d/12u-rMavow4fvidsFlLcLe7DAXuqWk8XUHOBV9kw05Z8/)
#[allow(missing_docs)]
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Identitypermission {
    Issuer,
    SimpleTokenIssuer,
    Validator,
    ClaimIssuer,
    // From MESH-235
    Investor,
    NodeRunner,
    PM,
    KYCAMLClaimIssuer,
    AccreditedInvestorClaimIssuer,
    VerifiedIdentityClaimIssuer,
    Custom(u8),
}
