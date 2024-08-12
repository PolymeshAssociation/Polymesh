use sp_runtime::runtime_logger::RuntimeLogger;
use sp_std::collections::btree_map::BTreeMap;

use super::*;
use polymesh_primitives::Scope;

mod v0 {
    use codec::{Decode, Encode};
    use scale_info::TypeInfo;

    use super::*;
    use polymesh_primitives::{CddId, CountryCode, CustomClaimTypeId, Ticker};

    #[derive(Encode, Decode, TypeInfo, Default, Clone, PartialEq, Eq)]
    pub struct AssetCompliance {
        pub paused: bool,
        pub requirements: Vec<ComplianceRequirement>,
    }

    #[derive(Encode, Decode, TypeInfo, Default, Clone, PartialEq, Eq, Debug)]
    pub struct ComplianceRequirement {
        pub sender_conditions: Vec<Condition>,
        pub receiver_conditions: Vec<Condition>,
        pub id: u32,
    }

    #[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Debug, Hash)]
    pub struct Condition {
        pub condition_type: ConditionType,
        pub issuers: Vec<TrustedIssuer>,
    }

    #[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Debug, Hash)]
    pub enum ConditionType {
        IsPresent(Claim),
        IsAbsent(Claim),
        IsAnyOf(Vec<Claim>),
        IsNoneOf(Vec<Claim>),
        IsIdentity(TargetIdentity),
    }

    #[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Debug, Hash)]
    pub enum Claim {
        Accredited(Scope),
        Affiliate(Scope),
        BuyLockup(Scope),
        SellLockup(Scope),
        CustomerDueDiligence(CddId),
        KnowYourCustomer(Scope),
        Jurisdiction(CountryCode, Scope),
        Exempted(Scope),
        Blocked(Scope),
        Custom(CustomClaimTypeId, Option<Scope>),
    }

    #[derive(Encode, Decode, TypeInfo)]
    #[derive(Clone, PartialEq, Eq, Debug, PartialOrd, Ord, Hash)]
    pub enum Scope {
        Identity(IdentityId),
        Ticker(Ticker),
        Custom(Vec<u8>),
    }

    decl_storage! {
        trait Store for Module<T: Config> as ComplianceManager {
            // This storage changed the Ticker key to AssetID.
            // The Scope type, which is inside the compliance codition, has also been changed.
            pub AssetCompliances get(fn asset_compliance):
                map hasher(blake2_128_concat) Ticker => AssetCompliance;

            // This storage changed the Ticker key to AssetID.
            pub TrustedClaimIssuer get(fn trusted_claim_issuer):
                map hasher(blake2_128_concat) Ticker => Vec<TrustedIssuer>;
        }
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin { }
    }
}

impl From<v0::Scope> for Scope {
    fn from(v0_scope: v0::Scope) -> Self {
        match v0_scope {
            v0::Scope::Identity(did) => Scope::Identity(did),
            v0::Scope::Ticker(ticker) => Scope::Asset(AssetID::from(ticker)),
            v0::Scope::Custom(bytes) => Scope::Custom(bytes),
        }
    }
}

impl From<v0::Claim> for Claim {
    fn from(v0_claim: v0::Claim) -> Self {
        match v0_claim {
            v0::Claim::Accredited(scope) => Claim::Accredited(scope.into()),
            v0::Claim::Affiliate(scope) => Claim::Affiliate(scope.into()),
            v0::Claim::BuyLockup(scope) => Claim::BuyLockup(scope.into()),
            v0::Claim::SellLockup(scope) => Claim::SellLockup(scope.into()),
            v0::Claim::CustomerDueDiligence(cdd_id) => Claim::CustomerDueDiligence(cdd_id),
            v0::Claim::KnowYourCustomer(scope) => Claim::KnowYourCustomer(scope.into()),
            v0::Claim::Jurisdiction(cc, scope) => Claim::Jurisdiction(cc, scope.into()),
            v0::Claim::Exempted(scope) => Claim::Exempted(scope.into()),
            v0::Claim::Blocked(scope) => Claim::Blocked(scope.into()),
            v0::Claim::Custom(id, option_scope) => {
                Claim::Custom(id, option_scope.map(|scope| scope.into()))
            }
        }
    }
}

impl From<v0::ConditionType> for ConditionType {
    fn from(v0_condition_type: v0::ConditionType) -> Self {
        match v0_condition_type {
            v0::ConditionType::IsPresent(claim) => ConditionType::IsPresent(claim.into()),
            v0::ConditionType::IsAbsent(claim) => ConditionType::IsAbsent(claim.into()),
            v0::ConditionType::IsAnyOf(claims) => {
                ConditionType::IsAnyOf(claims.into_iter().map(|claim| claim.into()).collect())
            }
            v0::ConditionType::IsNoneOf(claims) => {
                ConditionType::IsNoneOf(claims.into_iter().map(|claim| claim.into()).collect())
            }
            v0::ConditionType::IsIdentity(target_did) => ConditionType::IsIdentity(target_did),
        }
    }
}

impl From<v0::Condition> for Condition {
    fn from(v0_condition: v0::Condition) -> Self {
        Condition {
            condition_type: v0_condition.condition_type.into(),
            issuers: v0_condition.issuers,
        }
    }
}

impl From<v0::ComplianceRequirement> for ComplianceRequirement {
    fn from(v0_compliance_req: v0::ComplianceRequirement) -> Self {
        ComplianceRequirement {
            sender_conditions: v0_compliance_req
                .sender_conditions
                .into_iter()
                .map(|condition| condition.into())
                .collect(),
            receiver_conditions: v0_compliance_req
                .receiver_conditions
                .into_iter()
                .map(|condition| condition.into())
                .collect(),
            id: v0_compliance_req.id,
        }
    }
}

impl From<v0::AssetCompliance> for AssetCompliance {
    fn from(v0_asset_compliance: v0::AssetCompliance) -> Self {
        AssetCompliance {
            paused: v0_asset_compliance.paused,
            requirements: v0_asset_compliance
                .requirements
                .into_iter()
                .map(|req| req.into())
                .collect(),
        }
    }
}

pub(crate) fn migrate_to_v1<T: Config>() {
    RuntimeLogger::init();
    let mut ticker_to_asset_id = BTreeMap::new();

    let mut count = 0;
    log::info!("Updating types for the AssetCompliances storage");
    v0::AssetCompliances::drain().for_each(|(ticker, compliance)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        AssetCompliances::insert(asset_id, AssetCompliance::from(compliance));
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the TrustedClaimIssuer storage");
    v0::TrustedClaimIssuer::drain().for_each(|(ticker, trusted_issuers)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        TrustedClaimIssuer::insert(asset_id, trusted_issuers);
    });
    log::info!("{:?} items migrated", count);
}
