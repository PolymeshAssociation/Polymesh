use rstd::prelude::*;

use srml_support::{
    decl_event, decl_module, decl_storage, dispatch::Result,
    traits::{ChangeMembers, InitializeMembers},
};
use sr_primitives::{
    weights::{SimpleDispatchInfo},
};
use session::{historical::OnSessionEnding, SelectInitialValidators};
use sr_staking_primitives::{
    SessionIndex,
    offence::{OnOffenceHandler, OffenceDetails, Offence, ReportOffence},
};
use system::{self, ensure_signed};
use staking;

/// The module's configuration trait.
pub trait Trait: staking::Trait {
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
		/// An entity has issued a candidacy. See the transaction for who.
		ValidatorAdded(AccountId),
		/// The given member was removed. See the transaction for who.
		ValidatorRemoved(AccountId),
	}
);

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		fn deposit_event() = default;

        /// Add a potential new validator to the pool of validators.
        /// Staking module checks `Members` to ensure validators have
        /// completed KYB compliance
		#[weight = SimpleDispatchInfo::FixedNormal(50_000)]
		fn add_validator_candidate(origin, member: T::AccountId) {
			let mut members = <Members<T>>::get();
			let index = members.binary_search(&member).err().ok_or("already a member")?;
			members.insert(index, member.clone());
			<Members<T>>::put(&members);

			// T::MembershipChanged::change_members_sorted(&[who], &[], &members[..]);

			Self::deposit_event(RawEvent::ValidatorAdded(member));
		}

        /// Removes a validator from the pool of validators. This can
        /// happen when a validator loses KYB compliance
		#[weight = SimpleDispatchInfo::FixedNormal(50_000)]
		fn remove_validator(origin, member: T::AccountId) {
			let mut members = <Members<T>>::get();
			let index = members.binary_search(&member).ok().ok_or("not a member")?;
			members.remove(index);
			<Members<T>>::put(&members);

			// T::MembershipChanged::change_members_sorted(&[], &[who], &members[..]);

			Self::deposit_event(RawEvent::ValidatorRemoved(member));
		}
	}
}

//impl<T: Trait> session::OnSessionEnding<T::AccountId> for Module<T> {
//    fn on_session_ending(_ending: SessionIndex, start_session: SessionIndex) -> Option<Vec<T::AccountId>> {
//        staking::new_session(start_session - 1).map(|(new, _old)| new)
//    }
//}

impl<T: Trait> Module<T> {
    pub fn is_compliant() -> bool {

        false
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
}
