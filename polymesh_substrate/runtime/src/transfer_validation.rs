use crate::asset_manager;
use rstd::prelude::*;
use parity_codec::Codec;
use support::{dispatch::Result, Parameter, StorageMap, StorageValue, decl_storage, decl_module, decl_event, ensure};
use runtime_primitives::traits::{CheckedSub, CheckedAdd, Member, SimpleArithmetic, As};
use system::{self, ensure_signed};

/// The module's configuration trait.
pub trait Trait: timestamp::Trait + system::Trait {
	// TODO: Add other types and constants required configure this module.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[derive(parity_codec::Encode, parity_codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct Whitelist<U,V> {
    investor: V,
    canSendAfter: U,
    canReceiveAfter: U
}

/// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as TransferValidation {

        WhitelistsByToken get(whitelists_by_token): map u32 => Vec<Whitelist<T::Moment, T::AccountId>>;
        
        WhitelistForTokenAndAddress get(whitelist_for_restriction): map (u32,T::AccountId) => Whitelist<T::Moment, T::AccountId>;
	}
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event<T>() = default;

	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
        Example(u32, AccountId, AccountId),
	}
);

impl<T: Trait> Module<T> {

        pub fn add_to_whitelist(sender: T::AccountId, token_id:u32, _investor: T::AccountId, expiry: T::Moment){
            //let mut now = <timestamp::Module<T>>::get();

            let whitelist = Whitelist {
                investor: _investor.clone(),
                canSendAfter:expiry.clone(),
                canReceiveAfter:expiry
            };

            let mut whitelists_for_token = Self::whitelists_by_token(token_id);
            whitelists_for_token.push(whitelist.clone());

            //PABLO: TODO: don't add the restriction to the array if it already exists
            <WhitelistsByToken<T>>::insert(token_id,whitelists_for_token);

            <WhitelistForTokenAndAddress<T>>::insert((token_id,_investor),whitelist);

            runtime_io::print("Created restriction!!!");
        }

        pub fn verifyWhitelistRestriction(token_id: u32, from: T::AccountId, to: T::AccountId) -> (bool,&'static str) {
            let mut _can_transfer = false;
            let now = <timestamp::Module<T>>::get();
            let whitelistForFrom = Self::whitelist_for_restriction((token_id,from));
            let whitelistForTo = Self::whitelist_for_restriction((token_id,to));
            if (whitelistForFrom.canSendAfter > T::Moment::sa(0) && now >= whitelistForFrom.canSendAfter) && (whitelistForTo.canReceiveAfter > T::Moment::sa(0) && now > whitelistForTo.canReceiveAfter) {
                _can_transfer = true;
            }
            (_can_transfer, "Transfer failed: simple restriction in place")
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
