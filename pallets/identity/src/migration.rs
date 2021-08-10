use crate::types::{Claim1stKey, Claim2ndKey};

use super::*;
use confidential_identity::mocked::make_investor_uid;
use polymesh_primitives::{
    agent::AgentGroup, Authorization, Balance, CddId, Claim, ClaimType, IdentityClaim, IdentityId,
    PortfolioId, Ticker,
};

/// Migrate claim.
pub fn migrate_claim(
    k1: Claim1stKey,
    _k2: Claim2ndKey,
    id_claim: IdentityClaim,
) -> Option<IdentityClaim> {
    match &k1.claim_type {
        ClaimType::CustomerDueDiligence => migrate_cdd_claim(k1.target, id_claim),
        _ => Some(id_claim),
    }
}

/// CDD claims are going to be mocked, where the Investor UID is the hash of its `IdentityId`.
fn migrate_cdd_claim(target: IdentityId, mut id_claim: IdentityClaim) -> Option<IdentityClaim> {
    let uid = make_investor_uid(target.as_bytes()).into();
    let cdd_id = CddId::new_v1(target, uid);

    id_claim.claim = Claim::CustomerDueDiligence(cdd_id);
    Some(id_claim)
}

/// Authorization V1 data for two step processes.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum AuthDataV1<AccountId> {
    AttestPrimaryKeyRotation(IdentityId),
    RotatePrimaryKey(IdentityId),
    TransferTicker(Ticker),
    TransferPrimaryIssuanceAgent(Ticker),
    AddMultiSigSigner(AccountId),
    TransferAssetOwnership(Ticker),
    JoinIdentity(Permissions),
    PortfolioCustody(PortfolioId),
    Custom(Ticker),
    NoData,
    TransferCorporateActionAgent(Ticker),
    BecomeAgent(Ticker, AgentGroup),
    AddRelayerPayingKey(AccountId, AccountId, Balance),
}

/// Authorization v1 struct.
#[derive(Encode, Decode, Clone, PartialEq, Debug)]
pub struct AuthV1<AccountId, Moment> {
    pub authorization_data: AuthDataV1<AccountId>,
    pub authorized_by: IdentityId,
    pub expiry: Option<Moment>,
    pub auth_id: u64,
}

/// Migrate `AuthorizationData::RotatePrimaryKey`.
pub fn migrate_auth_v1<AccountId, Moment>(
    _k1: Signatory<AccountId>,
    _k2: u64,
    auth: AuthV1<AccountId, Moment>,
) -> Result<Authorization<AccountId, Moment>, ()> {
    let authorization_data = match auth.authorization_data {
        AuthDataV1::AttestPrimaryKeyRotation(did) => {
            AuthorizationData::AttestPrimaryKeyRotation(did)
        }
        AuthDataV1::RotatePrimaryKey(_) => AuthorizationData::RotatePrimaryKey,
        AuthDataV1::TransferTicker(ticker) => AuthorizationData::TransferTicker(ticker),
        AuthDataV1::TransferPrimaryIssuanceAgent(ticker) => {
            AuthorizationData::TransferPrimaryIssuanceAgent(ticker)
        }
        AuthDataV1::AddMultiSigSigner(acc) => AuthorizationData::AddMultiSigSigner(acc),
        AuthDataV1::TransferAssetOwnership(ticker) => {
            AuthorizationData::TransferAssetOwnership(ticker)
        }
        AuthDataV1::JoinIdentity(perms) => AuthorizationData::JoinIdentity(perms),
        AuthDataV1::PortfolioCustody(portfolio) => AuthorizationData::PortfolioCustody(portfolio),
        AuthDataV1::Custom(ticker) => AuthorizationData::Custom(ticker),
        AuthDataV1::NoData => AuthorizationData::NoData,
        AuthDataV1::TransferCorporateActionAgent(ticker) => {
            AuthorizationData::TransferCorporateActionAgent(ticker)
        }
        AuthDataV1::BecomeAgent(ticker, group) => AuthorizationData::BecomeAgent(ticker, group),
        AuthDataV1::AddRelayerPayingKey(user, payer, limit) => {
            AuthorizationData::AddRelayerPayingKey(user, payer, limit)
        }
    };

    Ok(Authorization {
        authorization_data,
        authorized_by: auth.authorized_by,
        expiry: auth.expiry,
        auth_id: auth.auth_id,
    })
}
