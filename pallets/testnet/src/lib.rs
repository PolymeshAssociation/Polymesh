#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, weights::Weight,
};
use frame_system::{ensure_signed, RawOrigin};
use pallet_identity::PermissionedCallOriginData;
use polymesh_common_utilities::{
    protocol_fee::ProtocolOp, traits::identity::Trait as IdentityTrait,
};
use polymesh_primitives::{
    secondary_key::api::SecondaryKey, CddId, Claim, IdentityId, InvestorUid,
};
use sp_std::{prelude::*, vec};

type Identity<T> = pallet_identity::Module<T>;
type CallPermissions<T> = pallet_permissions::Module<T>;

pub trait WeightInfo {
    fn register_did(i: u32) -> Weight;
    fn mock_cdd_register_did() -> Weight;
    fn get_my_did() -> Weight;
    fn get_cdd_of() -> Weight;
}

pub trait Trait: IdentityTrait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    /// Weight information for extrinsics in the identity pallet.
    type WeightInfo: WeightInfo;
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        /// A new mocked `InvestorUid` has been created for the given Identity.
        /// (Target DID, New InvestorUid)
        MockInvestorUIDCreated(IdentityId, InvestorUid),
        /// Emits the `IdentityId` and the `AccountId` of the caller.
        /// (Caller DID, Caller account)
        DidStatus(IdentityId, AccountId),
        /// Shows the `DID` associated to the `AccountId`, and a flag indicates if that DID has a
        /// valid CDD claim.
        /// (Target DID, Target Account, a valid CDD claim exists)
        CddStatus(Option<IdentityId>, AccountId, bool),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as testnet {
    }
}
decl_error! {
    pub enum Error for Module<T: Trait> {
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        type Error = Error<T>;

        fn deposit_event() = default;

        /// Generates a new `IdentityID` for the caller, and issues a self-generated CDD claim.
        ///
        /// The caller account will be the primary key of that identity.
        /// For each account of `secondary_keys`, a new `JoinIdentity` authorization is created, so
        /// each of them will need to accept it before become part of this new `IdentityID`.
        ///
        /// # Errors
        /// - `AlreadyLinked` if the caller account or if any of the given `secondary_keys` has already linked to an `IdentityID`
        /// - `SecondaryKeysContainPrimaryKey` if `secondary_keys` contains the caller account.
        /// - `DidAlreadyExists` if auto-generated DID already exists.
        #[weight = <T as Trait>::WeightInfo::register_did(secondary_keys.len() as u32)]
        pub fn register_did(
            origin,
            uid: InvestorUid,
            secondary_keys: Vec<SecondaryKey<T::AccountId>>,
        ) {
            let sender = ensure_signed(origin)?;
            Identity::<T>::_register_did(sender.clone(), secondary_keys, Some(ProtocolOp::IdentityRegisterDid))?;

            // Add CDD claim
            let did = Identity::<T>::get_identity(&sender).ok_or("DID Self-register failed")?;
            let cdd_claim = Claim::CustomerDueDiligence(CddId::new(did, uid));
            Identity::<T>::base_add_claim(did, cdd_claim, did, None);
        }

        /// Registers a new Identity for the `target_account` and issues a CDD claim to it.
        /// The Investor UID is generated deterministically by the hash of the generated DID and
        /// then we fix it to be compliant with UUID v4.
        ///
        /// # See
        /// - [RFC 4122: UUID](https://tools.ietf.org/html/rfc4122)
        ///
        /// # Failure
        /// - `origin` has to be a active CDD provider. Inactive CDD providers cannot add new
        /// claims.
        /// - `target_account` (primary key of the new Identity) can be linked to just one and only
        /// one identity.
        ///
        /// # Weight
        /// `7_000_000_000
        #[weight = <T as Trait>::WeightInfo::mock_cdd_register_did()]
        pub fn mock_cdd_register_did(origin, target_account: T::AccountId) {
            let cdd_id = Identity::<T>::ensure_perms(origin)?;
            let target_did = Identity::<T>::base_cdd_register_did(cdd_id, target_account, vec![])?;
            let target_uid = confidential_identity::mocked::make_investor_uid(target_did.as_bytes());

            // Add CDD claim for the target
            let cdd_claim = Claim::CustomerDueDiligence(CddId::new(target_did, target_uid.clone().into()));
            Identity::<T>::base_add_claim(target_did, cdd_claim, cdd_id, None);

            Self::deposit_event(RawEvent::MockInvestorUIDCreated(target_did, target_uid.into()));
        }

        /// Emits an event with caller's identity.
        #[weight = <T as Trait>::WeightInfo::get_my_did()]
        pub fn get_my_did(origin) {
            let PermissionedCallOriginData {
                sender,
                primary_did: did,
                ..
            } = Identity::<T>::ensure_origin_call_permissions(origin)?;
            Self::deposit_event(RawEvent::DidStatus(did, sender));
        }

        /// Emits an event with caller's identity and CDD status.
        #[weight = <T as Trait>::WeightInfo::get_cdd_of()]
        pub fn get_cdd_of(origin, of: T::AccountId) {
            let sender = ensure_signed(origin)?;
            CallPermissions::<T>::ensure_call_permissions(&sender)?;
            let did_opt = Identity::<T>::get_identity(&of);
            let has_cdd = did_opt.map(Identity::<T>::has_valid_cdd).unwrap_or_default();

            Self::deposit_event(RawEvent::CddStatus(did_opt, of, has_cdd));
        }
    }
}

impl<T: Trait> polymesh_common_utilities::TestnetFn<T::AccountId> for Module<T> {
    fn register_did(
        target: T::AccountId,
        investor: InvestorUid,
        secondary_keys: Vec<SecondaryKey<T::AccountId>>,
    ) -> DispatchResult {
        Self::register_did(RawOrigin::Signed(target).into(), investor, secondary_keys)
    }
}
