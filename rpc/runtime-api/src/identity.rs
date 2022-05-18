use codec::Codec;
use pallet_identity::types::{
    AssetDidResult, CddStatus, DidStatus, KeyIdentityData, RpcDidRecords,
};
use polymesh_primitives::{Authorization, AuthorizationType, Signatory};
use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
    /// Identity runtime API.
    #[api_version(2)]
    pub trait IdentityApi<IdentityId, Ticker, AccountId, Moment> where
        IdentityId: Codec,
        Ticker: Codec,
        AccountId: Codec,
        Moment: Codec
    {
        /// Returns CDD status of an identity
        fn is_identity_has_valid_cdd(did: IdentityId, buffer_time: Option<u64>) -> CddStatus;

        /// Returns DID of an asset
        fn get_asset_did(ticker: Ticker) -> AssetDidResult;

        /// Retrieve DidRecord for a given `did`.
        #[deprecated]
        fn get_did_records(did: IdentityId) -> RpcDidRecords<AccountId>;

        /// Retrieve DidRecord for a given `did`.
        ///
        /// Old v1 call for `Signatory` based secondary keys.
        #[changed_in(2)]
        fn get_did_records(did: IdentityId) -> pallet_identity::types::v1::RpcDidRecords<AccountId>;

        /// Retrieve list of a authorization for a given signatory
        fn get_filtered_authorizations(
            signatory: Signatory<AccountId>,
            allow_expired: bool,
            auth_type: Option<AuthorizationType>
        ) -> Vec<Authorization<AccountId, Moment>>;

        /// Retrieve the status of the DID
        fn get_did_status(dids: Vec<IdentityId>) -> Vec<DidStatus>;

        /// Provide the `KeyIdentityData` from a given `AccountId`, including:
        /// - the corresponding DID,
        /// - whether the `AccountId` is a primary or secondary key,
        /// - any permissions related to the key.
        ///
        /// This is an aggregate call provided for UX convenience.
        fn get_key_identity_data(acc: AccountId) -> Option<KeyIdentityData<IdentityId>>;
    }
}
