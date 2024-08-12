use sp_runtime::runtime_logger::RuntimeLogger;

use super::*;

mod v6 {
    use scale_info::TypeInfo;
    use sp_std::collections::btree_set::BTreeSet;

    use super::*;
    use polymesh_primitives::{
        agent::AgentGroup, Balance, CountryCode, ExtrinsicPermissions, Moment, PortfolioId,
        PortfolioPermissions, Ticker,
    };

    #[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
    pub struct Claim2ndKey {
        pub issuer: IdentityId,
        pub scope: Option<Scope>,
    }

    #[derive(Encode, Decode, TypeInfo)]
    #[derive(Clone, PartialEq, Eq, Debug, PartialOrd, Ord, Hash)]
    pub enum Scope {
        Identity(IdentityId),
        Ticker(Ticker),
        Custom(Vec<u8>),
    }

    #[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq)]
    pub struct IdentityClaim {
        pub claim_issuer: IdentityId,
        pub issuance_date: Moment,
        pub last_update_date: Moment,
        pub expiry: Option<Moment>,
        pub claim: Claim,
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
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum KeyRecord<AccountId> {
        PrimaryKey(IdentityId),
        SecondaryKey(IdentityId, Permissions),
        MultiSigSignerKey(AccountId),
    }

    #[derive(Decode, Encode, TypeInfo)]
    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Permissions {
        pub asset: AssetPermissions,
        pub extrinsic: ExtrinsicPermissions,
        pub portfolio: PortfolioPermissions,
    }

    pub type AssetPermissions = SubsetRestriction;

    #[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum SubsetRestriction {
        Whole,
        These(BTreeSet<Ticker>),
        Except(BTreeSet<Ticker>),
    }

    #[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Debug)]
    pub struct Authorization<AccountId, Moment> {
        pub authorization_data: AuthorizationData<AccountId>,
        pub authorized_by: IdentityId,
        pub expiry: Option<Moment>,
        pub auth_id: u64,
        pub count: u32,
    }

    #[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
    pub enum AuthorizationData<AccountId> {
        AttestPrimaryKeyRotation(IdentityId),
        RotatePrimaryKey,
        TransferTicker(Ticker),
        AddMultiSigSigner(AccountId),
        TransferAssetOwnership(Ticker),
        JoinIdentity(Permissions),
        PortfolioCustody(PortfolioId),
        BecomeAgent(Ticker, AgentGroup),
        AddRelayerPayingKey(AccountId, AccountId, Balance),
        RotatePrimaryKeyToSecondary(Permissions),
    }

    decl_storage! {
        trait Store for Module<T: Config> as Identity {
            // This storage changed the Ticker key to AssetID.
            pub Claims: double_map hasher(twox_64_concat) Claim1stKey, hasher(blake2_128_concat) Claim2ndKey => Option<IdentityClaim>;

            pub KeyRecords get(fn key_records):
                map hasher(twox_64_concat) T::AccountId => Option<KeyRecord<T::AccountId>>;

            pub Authorizations get(fn authorizations):
                double_map hasher(blake2_128_concat) Signatory<T::AccountId>, hasher(twox_64_concat) u64 => Option<Authorization<T::AccountId, T::Moment>>;
        }
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin { }
    }
}

impl From<v6::Claim2ndKey> for Claim2ndKey {
    fn from(v6_claim2key: v6::Claim2ndKey) -> Self {
        Claim2ndKey {
            issuer: v6_claim2key.issuer,
            scope: v6_claim2key.scope.map(|v| v.into()),
        }
    }
}

impl From<v6::Scope> for Scope {
    fn from(v6_scope: v6::Scope) -> Self {
        match v6_scope {
            v6::Scope::Identity(did) => Scope::Identity(did),
            v6::Scope::Ticker(ticker) => Scope::Asset(ticker.into()),
            v6::Scope::Custom(bytes) => Scope::Custom(bytes),
        }
    }
}

impl From<v6::Claim> for Claim {
    fn from(v6_claim: v6::Claim) -> Self {
        match v6_claim {
            v6::Claim::Accredited(scope) => Claim::Accredited(scope.into()),
            v6::Claim::Affiliate(scope) => Claim::Affiliate(scope.into()),
            v6::Claim::BuyLockup(scope) => Claim::BuyLockup(scope.into()),
            v6::Claim::SellLockup(scope) => Claim::SellLockup(scope.into()),
            v6::Claim::CustomerDueDiligence(cdd_id) => Claim::CustomerDueDiligence(cdd_id),
            v6::Claim::KnowYourCustomer(scope) => Claim::KnowYourCustomer(scope.into()),
            v6::Claim::Jurisdiction(cc, scope) => Claim::Jurisdiction(cc, scope.into()),
            v6::Claim::Exempted(scope) => Claim::Exempted(scope.into()),
            v6::Claim::Blocked(scope) => Claim::Blocked(scope.into()),
            v6::Claim::Custom(id, option_scope) => {
                Claim::Custom(id, option_scope.map(|scope| scope.into()))
            }
        }
    }
}

impl From<v6::IdentityClaim> for IdentityClaim {
    fn from(v6_id_claim: v6::IdentityClaim) -> Self {
        IdentityClaim {
            claim_issuer: v6_id_claim.claim_issuer,
            issuance_date: v6_id_claim.issuance_date,
            last_update_date: v6_id_claim.last_update_date,
            expiry: v6_id_claim.expiry,
            claim: v6_id_claim.claim.into(),
        }
    }
}

use polymesh_primitives::AssetPermissions;

impl From<v6::AssetPermissions> for AssetPermissions {
    fn from(v6_asset_perms: v6::AssetPermissions) -> Self {
        match v6_asset_perms {
            v6::AssetPermissions::Whole => AssetPermissions::Whole,
            v6::AssetPermissions::These(tickers) => {
                AssetPermissions::These(tickers.into_iter().map(|t| t.into()).collect())
            }
            v6::AssetPermissions::Except(tickers) => {
                AssetPermissions::Except(tickers.into_iter().map(|t| t.into()).collect())
            }
        }
    }
}

impl From<v6::Permissions> for Permissions {
    fn from(v6_perms: v6::Permissions) -> Self {
        Permissions {
            asset: v6_perms.asset.into(),
            extrinsic: v6_perms.extrinsic,
            portfolio: v6_perms.portfolio,
        }
    }
}

impl<T> From<v6::KeyRecord<T>> for KeyRecord<T> {
    fn from(v6_key_record: v6::KeyRecord<T>) -> Self {
        match v6_key_record {
            v6::KeyRecord::PrimaryKey(did) => KeyRecord::PrimaryKey(did),
            v6::KeyRecord::SecondaryKey(did, permissions) => {
                KeyRecord::SecondaryKey(did, permissions.into())
            }
            v6::KeyRecord::MultiSigSignerKey(acc) => KeyRecord::MultiSigSignerKey(acc),
        }
    }
}

impl<T> From<v6::AuthorizationData<T>> for AuthorizationData<T> {
    fn from(v6_auth_data: v6::AuthorizationData<T>) -> Self {
        match v6_auth_data {
            v6::AuthorizationData::AttestPrimaryKeyRotation(did) => {
                AuthorizationData::AttestPrimaryKeyRotation(did)
            }
            v6::AuthorizationData::RotatePrimaryKey => AuthorizationData::RotatePrimaryKey,
            v6::AuthorizationData::TransferTicker(ticker) => {
                AuthorizationData::TransferTicker(ticker)
            }
            v6::AuthorizationData::AddMultiSigSigner(acc) => {
                AuthorizationData::AddMultiSigSigner(acc)
            }
            v6::AuthorizationData::TransferAssetOwnership(ticker) => {
                AuthorizationData::TransferAssetOwnership(ticker.into())
            }
            v6::AuthorizationData::JoinIdentity(perms) => {
                AuthorizationData::JoinIdentity(perms.into())
            }
            v6::AuthorizationData::PortfolioCustody(portfolio_id) => {
                AuthorizationData::PortfolioCustody(portfolio_id)
            }
            v6::AuthorizationData::BecomeAgent(ticker, ag) => {
                AuthorizationData::BecomeAgent(ticker.into(), ag)
            }
            v6::AuthorizationData::AddRelayerPayingKey(acc1, acc2, balance) => {
                AuthorizationData::AddRelayerPayingKey(acc1, acc2, balance)
            }
            v6::AuthorizationData::RotatePrimaryKeyToSecondary(perms) => {
                AuthorizationData::RotatePrimaryKeyToSecondary(perms.into())
            }
        }
    }
}

impl<T, S> From<v6::Authorization<T, S>> for Authorization<T, S> {
    fn from(v6_auth: v6::Authorization<T, S>) -> Self {
        Authorization {
            authorization_data: v6_auth.authorization_data.into(),
            authorized_by: v6_auth.authorized_by,
            expiry: v6_auth.expiry,
            auth_id: v6_auth.auth_id,
            count: v6_auth.count,
        }
    }
}

#[allow(dead_code)]
pub(crate) fn migrate_to_v7<T: Config>() {
    RuntimeLogger::init();

    // Removes all elements in the old storage and inserts it in the new storage
    log::info!("Updating types for the Claims storage");
    v6::Claims::drain().for_each(|(claim1key, claim2key, id_claim)| {
        Claims::insert(
            claim1key,
            Claim2ndKey::from(claim2key),
            IdentityClaim::from(id_claim),
        );
    });

    log::info!("Updating types for the KeyRecords storage");
    v6::KeyRecords::<T>::drain().for_each(|(acc_id, key_record)| {
        KeyRecords::<T>::insert(acc_id, KeyRecord::from(key_record));
    });

    log::info!("Updating types for the Authorizations storage");
    v6::Authorizations::<T>::drain().for_each(|(acc, n, auth)| {
        Authorizations::<T>::insert(acc, n, Authorization::from(auth));
    });
}
