use sp_std::prelude::*;
use sp_std::vec::Vec;

use polymesh_primitives::{
    Claim, Condition, ConditionType, CountryCode, Scope, TargetIdentity, Ticker, TrustedIssuer,
};

use crate::traits::compliance_manager::{ComplianceFnConfig, Config};

/// Adds a compliance rule that will require `trusted_claims_calls`, `id_fetch_claim_calls` and `external_agents_calls`
/// reads to the `TrustedClaimIssuer`, `Claims` and `GroupOfAgent` storage, and sets `trusted_issuer` as a trusted
/// issuer for the asset.
pub fn setup_compliance<T>(
    sender_origin: T::RuntimeOrigin,
    ticker: Ticker,
    trusted_issuer: TrustedIssuer,
    trusted_claims_calls: u32,
    id_fetch_claim_calls: u32,
    external_agents_calls: u32,
) where
    T: Config,
{
    add_trusted_issuer::<T>(sender_origin.clone(), ticker, trusted_issuer.clone());

    add_compliance_rule::<T>(
        sender_origin,
        ticker,
        trusted_claims_calls,
        id_fetch_claim_calls,
        external_agents_calls,
        vec![trusted_issuer.clone()],
    );
}

/// Adds one trusted issuer for ticker.
fn add_trusted_issuer<T>(origin: T::RuntimeOrigin, ticker: Ticker, trusted_issuer: TrustedIssuer)
where
    T: Config,
{
    T::ComplianceFn::add_default_trusted_claim_issuer(origin, ticker, trusted_issuer).unwrap();
}

/// Adds a compliance rule for `ticker`. The complexity of the rule will be dependent on the
/// number of calls to the `TrustedClaimIssuer`, `Claims`, and `GroupOfAgent` storage.
fn add_compliance_rule<T>(
    sender_origin: T::RuntimeOrigin,
    ticker: Ticker,
    trusted_claims_calls: u32,
    id_fetch_claim_calls: u32,
    external_agents_calls: u32,
    trusted_issuers: Vec<TrustedIssuer>,
) where
    T: Config,
{
    let (sender_conditions, receiver_conditions) = {
        if trusted_claims_calls == 0 {
            let sender_conditions = setup_conditions(
                0,
                id_fetch_claim_calls,
                external_agents_calls - 1,
                Some(trusted_issuers),
            );
            let receiver_conditions = setup_conditions(0, 0, 1, None);
            (sender_conditions, receiver_conditions)
        } else {
            let sender_conditions = setup_conditions(
                0,
                id_fetch_claim_calls - 1,
                external_agents_calls - 1,
                Some(trusted_issuers),
            );
            let receiver_conditions = setup_conditions(1, 1, 1, None);
            (sender_conditions, receiver_conditions)
        }
    };

    T::ComplianceFn::add_compliance_requirement(
        sender_origin,
        ticker.clone(),
        sender_conditions.clone(),
        receiver_conditions,
    )
    .unwrap();
}

/// Creates a `Vec<Condition>` that will require `trusted_claims_calls` lookups to `TrustedClaimIssuer`,
/// `id_fetch_claim_calls` calls to the `Claims` storage, and `external_agents_calls` calls to the `GroupOfAgent` storage.
fn setup_conditions(
    trusted_claims_calls: u32,
    id_fetch_claim_calls: u32,
    external_agents_calls: u32,
    trusted_issuers: Option<Vec<TrustedIssuer>>,
) -> Vec<Condition> {
    let conditions: Vec<Condition> = {
        if id_fetch_claim_calls == 0 {
            Vec::new()
        } else if trusted_claims_calls == 0 {
            vec![Condition::new(
                ConditionType::IsNoneOf(
                    (0..id_fetch_claim_calls)
                        .map(|i| {
                            Claim::Jurisdiction(CountryCode::BR, Scope::Custom(vec![(i + 1) as u8]))
                        })
                        .collect(),
                ),
                trusted_issuers.unwrap(),
            )]
        } else {
            vec![Condition::new(
                ConditionType::IsAbsent(Claim::Jurisdiction(
                    CountryCode::BR,
                    Scope::Custom(vec![0]),
                )),
                Vec::new(),
            )]
        }
    };

    let is_identity_conditions: Vec<Condition> = (0..external_agents_calls)
        .map(|_| {
            Condition::new(
                ConditionType::IsIdentity(TargetIdentity::ExternalAgent),
                Vec::new(),
            )
        })
        .collect();

    [conditions, is_identity_conditions].concat()
}
