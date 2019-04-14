use crate::utils;
use crate::asset;
use crate::asset::HasOwner;

use rstd::prelude::*;
use support::{dispatch::Result, StorageMap, StorageValue, decl_storage, decl_module, decl_event, ensure};
use runtime_primitives::traits::{As};
use system::{self, ensure_signed};

/// The module's configuration trait.
pub trait Trait: timestamp::Trait + system::Trait + utils::Trait {
	// TODO: Add other types and constants required configure this module.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type Asset: asset::HasOwner<Self::AccountId>;
}


decl_storage! {
	trait Store for Module<T: Trait> as PercentageTM {
        MaximumPercentageEnabledForToken get(maximum_percentage_enabled_for_token): map u32 => (bool,u16);
	}
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event<T>() = default;

		fn toggle_maximum_percentage_restriction(origin, token_id:u32, enable:bool, max_percentage: u16) -> Result  {
			let sender = ensure_signed(origin)?;
			ensure!(Self::is_owner(token_id, sender.clone()),"Sender must be the token owner");

			//PABLO: TODO: Move all the max % logic to a new module and call that one instead of holding all the different logics in just one module.
			<MaximumPercentageEnabledForToken<T>>::insert(token_id,(enable,max_percentage));

			if enable{
				runtime_io::print("Maximum percentage restriction enabled!");
			}else{ 
				runtime_io::print("Maximum percentage restriction disabled!");
			}

			Ok(())
		}
	}

}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
        Example(u32, AccountId, AccountId),
	}
);

impl<T: Trait> Module<T>{
    pub fn is_owner(token_id:u32, sender: T::AccountId) -> bool {
		T::Asset::is_owner(token_id, sender)
        // let token = T::Asset::token_details(token_id);
        // token.owner == sender
    }

	// Transfer restriction verification logic
	pub fn verify_restriction(token_id: u32, from: T::AccountId, to: T::AccountId, value: T::TokenBalance) -> Result {
		let mut _can_transfer = Self::maximum_percentage_enabled_for_token(token_id);
		let enabled = _can_transfer.0;
		// If the restriction is enabled, then we need to make the calculations, otherwise all good
		if enabled {
			Err("Cannot Transfer: Percentage TM restrictions not satisfied")
		}else{
			Ok(())
		}                    
	}


}

/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::with_externalities;
	use primitives::{H256, Blake2Hasher};
	use support::{impl_outer_origin, assert_ok};
	use runtime_primitives::{
		BuildStorage,
		traits::{BlakeTwo256, IdentityLookup},
		testing::{Digest, DigestItem, Header}
	};

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	impl system::Trait for Test {
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type Digest = Digest;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type Log = DigestItem;
	}
	impl Trait for Test {
		type Event = ();
	}
	type TransferValidationModule = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
	}
}
