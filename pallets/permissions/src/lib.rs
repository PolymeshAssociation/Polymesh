#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_module, decl_storage,
    dispatch::DispatchResult,
    storage::StorageValue,
    traits::{CallMetadata, EnsureOrigin, GetCallMetadata},
};
use sp_runtime::{
    traits::{DispatchInfoOf, PostDispatchInfoOf, SignedExtension},
    transaction_validity::{TransactionValidity, TransactionValidityError, ValidTransaction},
};
use sp_std::{fmt, marker::PhantomData, prelude::Vec, result::Result};

pub trait Trait: frame_system::Trait {}

decl_storage! {
    trait Store for Module<T: Trait> as Permissions {
        pub Who get(fn who): T::AccountId;
        pub PalletName get(fn pallet_name): Vec<u8>;
        pub FunctionName get(fn function_name): Vec<u8>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: <T as frame_system::Trait>::Origin {
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

impl<O, AccountId, T> EnsureOrigin<O> for EnsurePermissions<AccountId, T>
where
    O: Into<Result<RawOrigin<AccountId, T>, O>> + From<RawOrigin<AccountId, T>>,
    AccountId: Default,
    T: CheckAccountCallPermissions,
{
    type Success = AccountId;
    fn try_origin(o: O) -> Result<Self::Success, O> {
        o.into().and_then(|o| match o {
            RawOrigin {
                system_origin: frame_system::RawOrigin::Signed(who),
                call_metadata: Some(meta),
                _marker,
            } => {
                if !T::check_account_call_permissions(&meta) {
                    return Err(O::from(RawOrigin {
                        system_origin: frame_system::RawOrigin::Signed(who),
                        call_metadata: Some(meta),
                        _marker,
                    }));
                }
                // The origin has the required permissions. The rest is the same as in
                // `ensure_signed`.
                Ok(who)
            }
            u => Err(O::from(u)),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn successful_origin() -> O {
        O::from(RawOrigin {
            system_origin: frame_system::RawOrigin::Root,
            call_metadata: None,
            _marker: PhantomData::<T>::default(),
        })
    }
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
