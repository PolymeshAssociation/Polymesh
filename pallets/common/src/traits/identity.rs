// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2020 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use crate::{
    traits::{
        group::GroupTrait,
        multisig::MultiSigSubTrait,
        portfolio::PortfolioSubTrait,
        transaction_payment::{CddAndFeeDetails, ChargeTxFee},
        CommonConfig,
    },
    ChargeProtocolFee, SystematicIssuers,
};

use codec::{Decode, Encode};
use frame_support::{
    decl_event,
    dispatch::PostDispatchInfo,
    traits::{Currency, EnsureOrigin, Get, GetCallMetadata},
    weights::{GetDispatchInfo, Weight},
    Parameter,
};
use polymesh_primitives::{
    secondary_key::api::SecondaryKey, AuthorizationData, IdentityClaim, IdentityId, InvestorUid,
    Permissions, Signatory, Ticker,
};
use sp_core::H512;
use sp_runtime::traits::{Dispatchable, IdentifyAccount, Member, Verify};
use sp_std::vec::Vec;

pub type AuthorizationNonce = u64;

/// It represents an authorization that any account could sign to allow operations related with a
/// target identity.
///
/// # Safety
///
/// Please note, that `nonce` has been added to avoid **replay attack** and it should be the current
/// value of nonce of primary key of `target_id`. See `System::account_nonce`.
/// In this way, the authorization is delimited to an specific transaction (usually the next one)
/// of primary key of target identity.
#[derive(codec::Encode, codec::Decode, Clone, PartialEq, Eq, Debug)]
pub struct TargetIdAuthorization<Moment> {
    /// Target identity which is authorized to make an operation.
    pub target_id: IdentityId,
    /// It HAS TO be `target_id` authorization nonce: See `Identity::offchain_authorization_nonce`
    pub nonce: AuthorizationNonce,
    pub expires_at: Moment,
}

/// It is a secondary item with authorization of that secondary key (off-chain operation) to be added
/// to an identity.
/// `auth_signature` is the signature, generated by secondary item, of `TargetIdAuthorization`.
///
/// # TODO
///  - Replace `H512` type by a template type which represents explicitly the relation with
///  `TargetIdAuthorization`.
#[derive(codec::Encode, codec::Decode, Clone, PartialEq, Eq, Debug)]
pub struct SecondaryKeyWithAuth<AccountId> {
    /// Secondary key to be added.
    pub secondary_key: SecondaryKey<AccountId>,
    /// Off-chain authorization signature.
    pub auth_signature: H512,
}

pub trait WeightInfo {
    fn cdd_register_did(i: u32) -> Weight;
    fn invalidate_cdd_claims() -> Weight;
    fn remove_secondary_keys(i: u32) -> Weight;
    fn accept_primary_key() -> Weight;
    fn change_cdd_requirement_for_mk_rotation() -> Weight;
    fn join_identity_as_key() -> Weight;
    fn leave_identity_as_key() -> Weight;
    fn add_claim() -> Weight;
    fn revoke_claim() -> Weight;
    fn set_permission_to_signer() -> Weight;
    fn freeze_secondary_keys() -> Weight;
    fn unfreeze_secondary_keys() -> Weight;
    fn add_authorization() -> Weight;
    fn remove_authorization() -> Weight;
    fn add_secondary_keys_with_authorization(n: u32) -> Weight;
    fn add_investor_uniqueness_claim() -> Weight;
    fn add_investor_uniqueness_claim_v2() -> Weight;
    fn revoke_claim_by_index() -> Weight;
}

/// The module's configuration trait.
pub trait Config: CommonConfig + pallet_timestamp::Config + crate::traits::base::Config {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    /// An extrinsic call.
    type Proposal: Parameter
        + Dispatchable<Origin = <Self as frame_system::Config>::Origin, PostInfo = PostDispatchInfo>
        + GetCallMetadata
        + GetDispatchInfo
        + From<frame_system::Call<Self>>;
    /// MultiSig module
    type MultiSig: MultiSigSubTrait<Self::AccountId>;
    /// Portfolio module. Required to accept portfolio custody transfers.
    type Portfolio: PortfolioSubTrait<Self::AccountId>;
    /// Group module
    type CddServiceProviders: GroupTrait<Self::Moment>;
    /// Balances module
    type Balances: Currency<Self::AccountId>;
    /// Charges fee for forwarded call
    type ChargeTxFeeTarget: ChargeTxFee;
    /// Used to check and update CDD
    type CddHandler: CddAndFeeDetails<Self::AccountId, <Self as frame_system::Config>::Call>;

    type Public: IdentifyAccount<AccountId = Self::AccountId>;
    type OffChainSignature: Verify<Signer = Self::Public> + Member + Decode + Encode;
    type ProtocolFee: ChargeProtocolFee<Self::AccountId>;

    /// Origin for Governance Committee voting majority origin.
    type GCVotingMajorityOrigin: EnsureOrigin<Self::Origin>;

    /// Weight information for extrinsics in the identity pallet.
    type WeightInfo: WeightInfo;

    /// Identity functions
    type IdentityFn: IdentityFnTrait<Self::AccountId>;

    /// A type for identity-mapping the `Origin` type. Used by the scheduler.
    type SchedulerOrigin: From<frame_system::RawOrigin<Self::AccountId>>;

    /// POLYX given to primary keys of all new Identities
    type InitialPOLYX: Get<<Self::Balances as Currency<Self::AccountId>>::Balance>;
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
        Moment = <T as pallet_timestamp::Config>::Moment,
    {
        /// DID, primary key account ID, secondary keys
        DidCreated(IdentityId, AccountId, Vec<SecondaryKey<AccountId>>),

        /// DID, new keys
        SecondaryKeysAdded(IdentityId, Vec<SecondaryKey<AccountId>>),

        /// DID, the keys that got removed
        SecondaryKeysRemoved(IdentityId, Vec<Signatory<AccountId>>),

        /// A signer left their identity. (did, signer)
        SignerLeft(IdentityId, Signatory<AccountId>),

        /// DID, updated secondary key, previous permissions, new permissions
        SecondaryKeyPermissionsUpdated(
            IdentityId,
            SecondaryKey<AccountId>,
            Permissions,
            Permissions,
        ),

        /// DID, old primary key account ID, new ID
        PrimaryKeyUpdated(IdentityId, AccountId, AccountId),

        /// DID, claims
        ClaimAdded(IdentityId, IdentityClaim),

        /// DID, ClaimType, Claim Issuer
        ClaimRevoked(IdentityId, IdentityClaim),

        /// Asset DID
        AssetDidRegistered(IdentityId, Ticker),

        /// New authorization added.
        /// (authorised_by, target_did, target_key, auth_id, authorization_data, expiry)
        AuthorizationAdded(
            IdentityId,
            Option<IdentityId>,
            Option<AccountId>,
            u64,
            AuthorizationData<AccountId>,
            Option<Moment>,
        ),

        /// Authorization revoked by the authorizer.
        /// (authorized_identity, authorized_key, auth_id)
        AuthorizationRevoked(Option<IdentityId>, Option<AccountId>, u64),

        /// Authorization rejected by the user who was authorized.
        /// (authorized_identity, authorized_key, auth_id)
        AuthorizationRejected(Option<IdentityId>, Option<AccountId>, u64),

        /// Authorization consumed.
        /// (authorized_identity, authorized_key, auth_id)
        AuthorizationConsumed(Option<IdentityId>, Option<AccountId>, u64),

        /// Off-chain Authorization has been revoked.
        /// (Target Identity, Signatory)
        OffChainAuthorizationRevoked(IdentityId, Signatory<AccountId>),

        /// CDD requirement for updating primary key changed. (new_requirement)
        CddRequirementForPrimaryKeyUpdated(bool),

        /// CDD claims generated by `IdentityId` (a CDD Provider) have been invalidated from
        /// `Moment`.
        CddClaimsInvalidated(IdentityId, Moment),

        /// All Secondary keys of the identity ID are frozen.
        SecondaryKeysFrozen(IdentityId),

        /// All Secondary keys of the identity ID are unfrozen.
        SecondaryKeysUnfrozen(IdentityId),

        /// Mocked InvestorUid created.
        MockInvestorUIDCreated(IdentityId, InvestorUid),
    }
);

pub trait IdentityFnTrait<AccountId> {
    fn get_identity(key: &AccountId) -> Option<IdentityId>;
    fn current_identity() -> Option<IdentityId>;
    fn set_current_identity(id: Option<IdentityId>);
    fn current_payer() -> Option<AccountId>;
    fn set_current_payer(payer: Option<AccountId>);

    fn is_key_authorized(did: IdentityId, key: &AccountId) -> bool;
    fn is_primary_key(did: &IdentityId, key: &AccountId) -> bool;

    /// It adds a systematic CDD claim for each `target` identity.
    ///
    /// It is used when we add a new member to CDD providers or Governance Committee.
    fn add_systematic_cdd_claims(targets: &[IdentityId], issuer: SystematicIssuers);

    /// It removes the systematic CDD claim for each `target` identity.
    ///
    /// It is used when we remove a member from CDD providers or Governance Committee.
    fn revoke_systematic_cdd_claims(targets: &[IdentityId], issuer: SystematicIssuers);

    /// Provides the DID status for the given DID
    fn has_valid_cdd(target_did: IdentityId) -> bool;
}
