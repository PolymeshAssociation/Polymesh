use codec::Codec;
use pallet_identity::types::{DidStatus, KeyIdentityData, RpcDidRecords};
use polymesh_primitives::{Authorization, AuthorizationType, Signatory};
use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
    /// Identity runtime API.
    #[api_version(4)]
    pub trait IdentityApi<IdentityId, Ticker, AccountId, Moment> where
        IdentityId: Codec,
        Ticker: Codec,
        AccountId: Codec,
        Moment: Codec
    {
        /// Retrieve DidRecord for a given `did`.
        fn get_did_records(did: IdentityId) -> RpcDidRecords<AccountId>;

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
