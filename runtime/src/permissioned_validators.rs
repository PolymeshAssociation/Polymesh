use rstd::prelude::*;

use srml_support::{
    decl_event, decl_module, decl_storage, dispatch::Result
};
use sr_primitives::{
    weights::{DispatchInfo, SimpleDispatchInfo},
    transaction_validity::{
        ValidTransaction, TransactionValidityError,
        InvalidTransaction, TransactionValidity,
    },
    traits::{
        SignedExtension,
    },
};
use system::{self, ensure_signed};

/// The module's configuration trait.
pub trait Trait: system::Trait {
     /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as PermissionedValidators {
        Members get(members): Vec<T::AccountId>;
    }
}

decl_event!(
	pub enum Event<T> where
	    AccountId = <T as system::Trait>::AccountId
	{
		/// The given member was removed. See the transaction for who.
		MemberRemoved(AccountId),
		/// An entity has issued a candidacy. See the transaction for who.
		CandidateAdded(AccountId),
	}
);

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		fn deposit_event() = default;

		pub fn add_member(origin, member: T::AccountId) -> Result {
			let who = ensure_signed(origin)?;

			// here we are raising the Something event
			Self::deposit_event(RawEvent::CandidateAdded(member));
			Ok(())
		}
	}
}

#[derive(codec::Encode, codec::Decode, Clone, Eq, PartialEq)]
pub struct CheckValidatorPermission<T: Trait + Send + Sync>(rstd::marker::PhantomData<T>);

#[cfg(feature = "std")]
impl<T: Trait + Send + Sync> std::fmt::Debug for CheckValidatorPermission<T> {
    fn fmt(&self, _: &mut std::fmt::Formatter) -> std::fmt::Result {
        Ok(())
    }
}

impl<T: Trait + Send + Sync> SignedExtension for CheckValidatorPermission<T> {
    type AccountId = T::AccountId;
    type Call = T::Call;
    type AdditionalSigned = ();
    type Pre = ();

    fn additional_signed(&self) -> rstd::result::Result<(), TransactionValidityError> { Ok(()) }

    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        _: DispatchInfo,
        _: usize,
    ) -> TransactionValidity {

         return Ok(ValidTransaction::default());
    }
}



/// tests for this module
#[cfg(test)]
mod tests {
}
