use frame_support::storage::migration::move_prefix;
use sp_runtime::runtime_logger::RuntimeLogger;

use super::*;

mod v6 {
    use scale_info::TypeInfo;

    use super::*;
    use polymesh_primitives::agent::AgentGroup;
    use polymesh_primitives::v6::Permissions;
    use polymesh_primitives::{Balance, CountryCode, Moment, PortfolioId, Ticker};

    #[derive(Encode, Decode, TypeInfo)]
    pub struct Claim2ndKey {
        pub issuer: IdentityId,
        pub scope: Option<Scope>,
    }

    #[derive(Encode, Decode, TypeInfo)]
    pub enum Scope {
        Identity(IdentityId),
        Ticker(Ticker),
        Custom(Vec<u8>),
    }

    #[derive(Encode, Decode, TypeInfo)]
    pub struct IdentityClaim {
        pub claim_issuer: IdentityId,
        pub issuance_date: Moment,
        pub last_update_date: Moment,
        pub expiry: Option<Moment>,
        pub claim: Claim,
    }

    #[derive(Encode, Decode, TypeInfo)]
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
    pub enum KeyRecord<AccountId> {
        PrimaryKey(IdentityId),
        SecondaryKey(IdentityId, Permissions),
        MultiSigSignerKey(AccountId),
    }

    #[derive(Encode, Decode, TypeInfo)]
    pub struct Authorization<AccountId, Moment> {
        pub authorization_data: AuthorizationData<AccountId>,
        pub authorized_by: IdentityId,
        pub expiry: Option<Moment>,
        pub auth_id: u64,
        pub count: u32,
    }

    #[derive(Encode, Decode, TypeInfo)]
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
            pub OldClaims: double_map hasher(twox_64_concat) Claim1stKey, hasher(blake2_128_concat) Claim2ndKey => Option<IdentityClaim>;

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

pub(crate) fn migrate_to_v7<T: Config>() {
    RuntimeLogger::init();

    // Removes all elements in the old storage and inserts it in the new storage
    let mut count = 0;
    log::info!("Updating types for the Claims storage");
    move_prefix(&Claims::final_prefix(), &v6::OldClaims::final_prefix());
    v6::OldClaims::drain().for_each(|(claim1key, claim2key, id_claim)| {
        Claims::insert(
            claim1key,
            Claim2ndKey::from(claim2key),
            IdentityClaim::from(id_claim),
        );
        count += 1;
    });
    log::info!("Migrated {:?} Identity.Claims entries.", count);

    let mut count = 0;
    log::info!("Updating types for the KeyRecords storage");
    v6::KeyRecords::<T>::drain().for_each(|(acc_id, key_record)| {
        let key_record = match key_record {
            v6::KeyRecord::PrimaryKey(did) => KeyRecord::PrimaryKey(did),
            v6::KeyRecord::SecondaryKey(did, perms) => {
                // Move secondary key permissions into split storage.
                KeyAssetPermissions::<T>::insert(&acc_id, AssetPermissions::from(perms.asset));
                KeyExtrinsicPermissions::<T>::insert(
                    &acc_id,
                    ExtrinsicPermissions::from(perms.extrinsic),
                );
                KeyPortfolioPermissions::<T>::insert(&acc_id, perms.portfolio);
                // KeyRecord no longer has the permissions.
                KeyRecord::SecondaryKey(did)
            }
            v6::KeyRecord::MultiSigSignerKey(acc) => KeyRecord::MultiSigSignerKey(acc),
        };
        KeyRecords::<T>::insert(acc_id, key_record);
        count += 1;
    });
    log::info!("Migrated {:?} Identity.KeyRecords entries.", count);

    let mut count = 0;
    log::info!("Updating types for the Authorizations storage");
    v6::Authorizations::<T>::drain().for_each(|(acc, n, auth)| {
        Authorizations::<T>::insert(acc, n, Authorization::from(auth));
        count += 1;
    });
    log::info!("Migrated {:?} Identity.Authorizations entries.", count);
}
