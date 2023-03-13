// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
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
    ChargeProtocolFee,
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
    secondary_key::{v1, SecondaryKey},
    AuthorizationData, Balance, CustomClaimTypeId, IdentityClaim, IdentityId, Permissions,
    Signatory, Ticker,
};
use scale_info::TypeInfo;
use sp_core::H512;
use sp_runtime::traits::{Dispatchable, IdentifyAccount, Member, Verify};
use sp_std::convert::TryFrom;
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
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
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
#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Debug)]
pub struct SecondaryKeyWithAuth<AccountId> {
    /// Secondary key to be added.
    pub secondary_key: SecondaryKey<AccountId>,
    /// Off-chain authorization signature.
    pub auth_signature: H512,
}

#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SecondaryKeyWithAuthV1<AccountId> {
    secondary_key: v1::SecondaryKey<AccountId>,
    auth_signature: H512,
}

impl<AccountId> TryFrom<SecondaryKeyWithAuthV1<AccountId>> for SecondaryKeyWithAuth<AccountId> {
    type Error = ();
    fn try_from(auth: SecondaryKeyWithAuthV1<AccountId>) -> Result<Self, Self::Error> {
        match auth.secondary_key.signer {
            Signatory::Account(key) => Ok(Self {
                secondary_key: SecondaryKey {
                    key,
                    permissions: auth.secondary_key.permissions,
                },
                auth_signature: auth.auth_signature,
            }),
            _ => {
                // Unsupported `Signatory::Identity`.
                Err(())
            }
        }
    }
}

pub trait WeightInfo {
    fn has_valid_cdd() -> Weight;
    fn child_identity_has_valid_cdd() -> Weight;
    fn cdd_register_did(i: u32) -> Weight;
    fn invalidate_cdd_claims() -> Weight;
    fn remove_secondary_keys(i: u32) -> Weight;
    fn accept_primary_key() -> Weight;
    fn rotate_primary_key_to_secondary() -> Weight;
    fn change_cdd_requirement_for_mk_rotation() -> Weight;
    fn join_identity_as_key() -> Weight;
    fn leave_identity_as_key() -> Weight;
    fn add_claim() -> Weight;
    fn revoke_claim() -> Weight;
    fn set_secondary_key_permissions() -> Weight;
    /// Complexity Parameters:
    /// `a` = Number of (A)ssets
    /// `p` = Number of (P)ortfolios
    /// `l` = Number of pa(L)lets
    /// `e` = Number of (E)xtrinsics
    fn permissions_cost(a: u32, p: u32, l: u32, e: u32) -> Weight;

    fn permissions_cost_perms(perms: &Permissions) -> Weight {
        let (assets, portfolios, pallets, extrinsics) = perms.counts();
        Self::permissions_cost(assets, portfolios, pallets, extrinsics)
    }

    fn freeze_secondary_keys() -> Weight;
    fn unfreeze_secondary_keys() -> Weight;
    fn add_authorization() -> Weight;
    fn remove_authorization() -> Weight;
    fn add_secondary_keys_with_authorization(n: u32) -> Weight;
    fn add_investor_uniqueness_claim() -> Weight;
    fn add_investor_uniqueness_claim_v2() -> Weight;
    fn revoke_claim_by_index() -> Weight;
    fn register_custom_claim_type(n: u32) -> Weight;

    /// Add complexity cost of Permissions to `add_secondary_keys_with_authorization` extrinsic.
    fn add_secondary_keys_full_v1<AccountId>(
        additional_keys: &[SecondaryKeyWithAuthV1<AccountId>],
    ) -> Weight {
        Self::add_secondary_keys_perms_cost(
            additional_keys
                .iter()
                .map(|auth| &auth.secondary_key.permissions),
        )
    }

    /// Add complexity cost of Permissions to `add_secondary_keys_with_authorization` extrinsic.
    fn add_secondary_keys_full<AccountId>(
        additional_keys: &[SecondaryKeyWithAuth<AccountId>],
    ) -> Weight {
        Self::add_secondary_keys_perms_cost(
            additional_keys
                .iter()
                .map(|auth| &auth.secondary_key.permissions),
        )
    }

    /// Add complexity cost of Permissions to `add_secondary_keys_with_authorization` extrinsic.
    fn add_secondary_keys_perms_cost<'a>(
        perms: impl ExactSizeIterator<Item = &'a Permissions>,
    ) -> Weight {
        let len_cost = Self::add_secondary_keys_with_authorization(perms.len() as u32);
        perms.fold(len_cost, |cost, key| {
            cost.saturating_add(Self::permissions_cost_perms(key))
        })
    }

    /// Add complexity cost of Permissions to `add_authorization` extrinsic.
    fn add_authorization_full<AccountId>(data: &AuthorizationData<AccountId>) -> Weight {
        let perm_cost = match data {
            AuthorizationData::JoinIdentity(perms) => Self::permissions_cost_perms(perms),
            _ => 0,
        };
        perm_cost.saturating_add(Self::add_authorization())
    }

    /// Add complexity cost of Permissions to `set_secondary_key_permissions` extrinsic.
    fn set_secondary_key_permissions_full(perms: &Permissions) -> Weight {
        Self::permissions_cost_perms(perms).saturating_add(Self::set_secondary_key_permissions())
    }
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
    type Balances: Currency<Self::AccountId, Balance = Balance>;
    /// Charges fee for forwarded call
    type ChargeTxFeeTarget: ChargeTxFee;
    /// Used to check and update CDD
    type CddHandler: CddAndFeeDetails<Self::AccountId, <Self as frame_system::Config>::Call>;

    type Public: IdentifyAccount<AccountId = Self::AccountId>;
    type OffChainSignature: Verify<Signer = Self::Public> + Member + Decode + Encode + TypeInfo;
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

    /// Only allow MultiSig primary/secondary keys to be removed from an identity
    /// if its POLYX balance is below this limit.
    type MultiSigBalanceLimit: Get<<Self::Balances as Currency<Self::AccountId>>::Balance>;
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
        Moment = <T as pallet_timestamp::Config>::Moment,
    {
        /// Identity created.
        ///
        /// (DID, primary key, secondary keys)
        DidCreated(IdentityId, AccountId, Vec<SecondaryKey<AccountId>>),

        /// Secondary keys added to identity.
        ///
        /// (DID, new keys)
        SecondaryKeysAdded(IdentityId, Vec<SecondaryKey<AccountId>>),

        /// Secondary keys removed from identity.
        ///
        /// (DID, the keys that got removed)
        SecondaryKeysRemoved(IdentityId, Vec<AccountId>),

        /// A secondary key left their identity.
        ///
        /// (DID, secondary key)
        SecondaryKeyLeftIdentity(IdentityId, AccountId),

        /// Secondary key permissions updated.
        ///
        /// (DID, updated secondary key, previous permissions, new permissions)
        SecondaryKeyPermissionsUpdated(IdentityId, AccountId, Permissions, Permissions),

        /// Primary key of identity changed.
        ///
        /// (DID, old primary key account ID, new ID)
        PrimaryKeyUpdated(IdentityId, AccountId, AccountId),

        /// Claim added to identity.
        ///
        /// (DID, claim)
        ClaimAdded(IdentityId, IdentityClaim),

        /// Claim revoked from identity.
        ///
        /// (DID, claim)
        ClaimRevoked(IdentityId, IdentityClaim),

        /// Asset's identity registered.
        ///
        /// (Asset DID, ticker)
        AssetDidRegistered(IdentityId, Ticker),

        /// New authorization added.
        ///
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
        ///
        /// (authorized_identity, authorized_key, auth_id)
        AuthorizationRevoked(Option<IdentityId>, Option<AccountId>, u64),

        /// Authorization rejected by the user who was authorized.
        ///
        /// (authorized_identity, authorized_key, auth_id)
        AuthorizationRejected(Option<IdentityId>, Option<AccountId>, u64),

        /// Authorization consumed.
        ///
        /// (authorized_identity, authorized_key, auth_id)
        AuthorizationConsumed(Option<IdentityId>, Option<AccountId>, u64),

        /// Accepting Authorization retry limit reached.
        ///
        /// (authorized_identity, authorized_key, auth_id)
        AuthorizationRetryLimitReached(Option<IdentityId>, Option<AccountId>, u64),

        /// CDD requirement for updating primary key changed.
        ///
        /// (new_requirement)
        CddRequirementForPrimaryKeyUpdated(bool),

        /// CDD claims generated by `IdentityId` (a CDD Provider) have been invalidated from
        /// `Moment`.
        ///
        /// (CDD provider DID, disable from date)
        CddClaimsInvalidated(IdentityId, Moment),

        /// All Secondary keys of the identity ID are frozen.
        ///
        /// (DID)
        SecondaryKeysFrozen(IdentityId),

        /// All Secondary keys of the identity ID are unfrozen.
        ///
        /// (DID)
        SecondaryKeysUnfrozen(IdentityId),

        /// A new CustomClaimType was added.
        ///
        /// (DID, id, Type)
        CustomClaimTypeAdded(IdentityId, CustomClaimTypeId, Vec<u8>),

        /// Child identity created.
        ///
        /// (Parent DID, Child DID, primary key)
        ChildDidCreated(IdentityId, IdentityId, AccountId),

        /// Child identity unlinked from parent identity.
        ///
        /// (Caller DID, Parent DID, Child DID)
        ChildDidUnlinked(IdentityId, IdentityId, IdentityId),
    }
);

pub trait IdentityFnTrait<AccountId> {
    fn get_identity(key: &AccountId) -> Option<IdentityId>;
    fn current_identity() -> Option<IdentityId>;
    fn set_current_identity(id: Option<IdentityId>);
    fn current_payer() -> Option<AccountId>;
    fn set_current_payer(payer: Option<AccountId>);

    /// Provides the DID status for the given DID
    fn has_valid_cdd(target_did: IdentityId) -> bool;
}
