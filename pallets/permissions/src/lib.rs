#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_module, decl_storage,
    dispatch::DispatchResult,
    storage::StorageValue,
    traits::{CallMetadata, EnsureOrigin, GetCallMetadata},
};
use frame_system::{self as system, ensure_signed};
use sp_runtime::{
    traits::{BadOrigin, DispatchInfoOf, PostDispatchInfoOf, SignedExtension},
    transaction_validity::{TransactionValidity, TransactionValidityError, ValidTransaction},
};
use sp_std::{fmt, marker::PhantomData, prelude::Vec, result::Result};

pub trait Trait: frame_system::Trait {
    /// The origin that can be used with [`frame_system::ensure_signed`].
    type Origin: Into<Result<frame_system::RawOrigin<Self::AccountId>, <Self as Trait>::Origin>>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Permissions {
        pub Who get(fn who): T::AccountId;
        pub PalletName get(fn pallet_name): Vec<u8>;
        pub FunctionName get(fn function_name): Vec<u8>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: <T as Trait>::Origin {
        #[weight = 1]
        fn test(origin) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            Ok(())
        }
    }
}

/// A type of origin that wraps the Substrate system origin while adding call
pub struct RawOrigin<AccountId, T> {
    system_origin: frame_system::RawOrigin<AccountId>,
    call_metadata: Option<CallMetadata>,
    _marker: PhantomData<T>,
}

pub type Origin<T> = RawOrigin<<T as frame_system::Trait>::AccountId, T>;

pub trait CheckAccountCallPermissions {
    fn check_account_call_permissions(call_metadata: &CallMetadata) -> bool;
}

pub struct EnsurePermissions<AccountId, T>(PhantomData<(AccountId, T)>);

impl<OuterOrigin, AccountId, T> EnsureOrigin<OuterOrigin> for EnsurePermissions<AccountId, T>
where
    OuterOrigin: Into<Result<RawOrigin<AccountId, T>, OuterOrigin>> + From<RawOrigin<AccountId, T>>,
    AccountId: Default,
    T: CheckAccountCallPermissions,
{
    type Success = AccountId;

    fn try_origin(o: OuterOrigin) -> Result<Self::Success, OuterOrigin> {
        o.into().and_then(|o| match o {
            RawOrigin {
                system_origin: frame_system::RawOrigin::Signed(who),
                call_metadata: Some(meta),
                _marker,
            } => {
                if !T::check_account_call_permissions(&meta) {
                    return Err(OuterOrigin::from(RawOrigin {
                        system_origin: frame_system::RawOrigin::Signed(who),
                        call_metadata: Some(meta),
                        _marker,
                    }));
                }
                // The origin has the required permissions. The rest is the same as in
                // `ensure_signed`.
                Ok(who)
            }
            u => Err(OuterOrigin::from(u)),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn successful_origin() -> OuterOrigin {
        OuterOrigin::from(RawOrigin {
            system_origin: frame_system::RawOrigin::Root,
            call_metadata: None,
            _marker: PhantomData::<T>::default(),
        })
    }
}

/// Ensure that the origin `o` has permissions to call the extrinsic by calling the check
/// `T::check_account_call_permissions`. In case of success, returns a wrapped signer
/// `AccountId`. Otherwise returns `BadOrigin`.
pub fn ensure_permissions<OuterOrigin, AccountId, T>(o: OuterOrigin) -> Result<AccountId, BadOrigin>
where
    AccountId: Default,
    OuterOrigin: Into<Result<RawOrigin<AccountId, T>, OuterOrigin>> + From<RawOrigin<AccountId, T>>,
    T: CheckAccountCallPermissions,
{
    EnsurePermissions::ensure_origin(o)
}

#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct CheckPermissions<T: Trait + Send + Sync>(PhantomData<T>);

impl<T: Trait + Send + Sync> Default for CheckPermissions<T> {
    fn default() -> Self {
        CheckPermissions(PhantomData::<T>::default())
    }
}

impl<T: Trait + Send + Sync> fmt::Debug for CheckPermissions<T> {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CheckPermissions<{:?}>", self.0)
    }

    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

impl<T: Trait + Send + Sync> CheckPermissions<T>
where
    T::Call: GetCallMetadata,
{
    pub fn new() -> Self {
        Self::default()
    }

    fn set_call_context(who: &T::AccountId, pallet_name: &str, function_name: &str) {
        <Who<T>>::put(who);
        <PalletName>::put(pallet_name.as_bytes());
        <FunctionName>::put(function_name.as_bytes());
    }

    fn clear_call_context() {
        <Who<T>>::kill();
        <PalletName>::kill();
        <FunctionName>::kill();
    }
}

impl<T: Trait + Send + Sync> SignedExtension for CheckPermissions<T>
where
    T::Call: GetCallMetadata,
{
    const IDENTIFIER: &'static str = "CheckPermissions";
    type AccountId = T::AccountId;
    type Call = T::Call;
    type AdditionalSigned = ();
    type Pre = ();

    fn additional_signed(&self) -> Result<(), TransactionValidityError> {
        Ok(())
    }

    fn validate(
        &self,
        _who: &Self::AccountId,
        _call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> TransactionValidity {
        Ok(ValidTransaction::default())
    }

    fn pre_dispatch(
        self,
        who: &Self::AccountId,
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        let metadata = call.get_call_metadata();
        Self::set_call_context(who, metadata.pallet_name, metadata.function_name);
        Ok(())
    }

    fn post_dispatch(
        _pre: Self::Pre,
        _info: &DispatchInfoOf<Self::Call>,
        _post_info: &PostDispatchInfoOf<Self::Call>,
        _len: usize,
        _result: &DispatchResult,
    ) -> Result<(), TransactionValidityError> {
        Self::clear_call_context();
        Ok(())
    }
}
