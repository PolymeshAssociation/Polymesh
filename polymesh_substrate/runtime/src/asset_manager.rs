/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references


/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs

use crate::asset;
use crate::general_tm;
use crate::percentage_tm;
use rstd::prelude::*;
use support::{dispatch::Result,decl_storage, decl_module, decl_event, ensure};
use system::{self, ensure_signed};

/// The module's configuration trait.
pub trait Trait: system::Trait + asset::Trait + general_tm::Trait + percentage_tm::Trait {
	// TODO: Add other types and constants required configure this module.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as AssetManager {

	}
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event<T>() = default;

        fn add_to_whitelist(origin, _ticker: Vec<u8>, _investor: T::AccountId, expiry: T::Moment) -> Result {
					  let ticker = Self::_toUpper(_ticker);
            let sender = ensure_signed(origin)?;
            ensure!(Self::is_owner(ticker.clone(),sender.clone()),"Sender must be the token owner");

            <general_tm::Module<T>>::add_to_whitelist(sender,ticker.clone(),_investor,expiry);

            Ok(())

        }

        pub fn toggle_maximum_percentage_restriction(origin, _ticker: Vec<u8>, enable:bool, max_percentage: u16) -> Result  {
						let ticker = Self::_toUpper(_ticker);
            let sender = ensure_signed(origin)?;
            ensure!(Self::is_owner(ticker.clone(),sender.clone()),"Sender must be the token owner");

            //PABLO: TODO: Move all the max % logic to a new module and call that one instead of holding all the different logics in just one module.
            <percentage_tm::Module<T>>::toggle_maximum_percentage_restriction(ticker.clone(),enable,max_percentage);

            Ok(())

        }

	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		// Just a dummy event.
		// Event `Something` is declared with a parameter of the type `u32` and `AccountId`
		// To emit this event, we call the deposit funtion, from our runtime funtions
		SomethingStored(u32, AccountId),
	}
);

impl<T: Trait> Module<T>{

    pub fn is_owner(_ticker: Vec<u8>, sender: T::AccountId) -> bool {
				let ticker = Self::_toUpper(_ticker);
        let token = <asset::Module<T>>::token_details(ticker.clone());
        token.owner == sender
    }

		fn _toUpper(_hexArray: Vec<u8>) -> Vec<u8> {
        let mut hexArray = _hexArray.clone();
        for i in &mut hexArray {
            if *i >= 97 && *i <= 122 {
                *i -= 32;
            }
        }
        return hexArray;
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
	type TemplateModule = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
	}

}
