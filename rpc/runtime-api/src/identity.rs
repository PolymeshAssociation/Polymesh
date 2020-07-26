use codec::Codec;
use pallet_identity::types::{AssetDidResult, CddStatus, DidRecords, DidStatus};
use polymesh_primitives::{Authorization, AuthorizationType};
use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
    pub trait IdentityApi<IdentityId, Ticker, AccountId, SigningKey, Signatory, Moment> where
        IdentityId: Codec,
        Ticker: Codec,
        AccountId: Codec,
        SigningKey: Codec,
        Signatory: Codec,
        Moment: Codec
    {
        /// Returns CDD status of an identity
        fn is_identity_has_valid_cdd(did: IdentityId, buffer_time: Option<u64>) -> CddStatus;

        /// Returns DID of an asset
        fn get_asset_did(ticker: Ticker) -> AssetDidResult;

        /// Retrieve DidRecord for a given `did`.
        fn get_did_records(did: IdentityId) -> DidRecords<AccountId, SigningKey>;

        /// Retrieve list of a authorization for a given signatory
        fn get_filtered_authorizations(
            signatory: Signatory,
            allow_expired: bool,
            auth_type: Option<AuthorizationType>
        ) -> Vec<Authorization<AccountId, Moment>>;

        /// Retrieve the status of the DID
        fn get_did_status(dids: Vec<IdentityId>) -> Vec<DidStatus>;
    }
}
