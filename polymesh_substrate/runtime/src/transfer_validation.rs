use crate::utils;
use rstd::prelude::*;
use support::{StorageMap, StorageValue, decl_storage, decl_module, decl_event, ensure};
use runtime_primitives::traits::{As};
use system::{self};

/// The module's configuration trait.
pub trait Trait: timestamp::Trait + system::Trait + utils::Trait {
	// TODO: Add other types and constants required configure this module.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[derive(parity_codec::Encode, parity_codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct Restriction {
    name: Vec<u8>,
    restriction_type: u16,
    active: bool
}

#[derive(parity_codec::Encode, parity_codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct Whitelist<U,V> {
    investor: V,
    can_send_after: U,
    can_receive_after: U
}

decl_storage! {
	trait Store for Module<T: Trait> as TransferValidation {

        //PABLO: TODO: Idea here is to have a mapping/array of restrictions with a type and then loop through them applying their type of restriction. Whitelist would be associated to restriction instead of token.
        //RestrictionsForToken get(restrictions_for_token): map u32 => Vec<Restriction>;

        WhitelistsByToken get(whitelists_by_token): map u32 => Vec<Whitelist<T::Moment, T::AccountId>>;
        
        WhitelistForTokenAndAddress get(whitelist_for_restriction): map (u32,T::AccountId) => Whitelist<T::Moment, T::AccountId>;

        MaximumPercentageEnabledForToken get(maximum_percentage_enabled_for_token): map u32 => (bool,u16);
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
                can_send_after:expiry.clone(),
                can_receive_after:expiry
            };

            let mut whitelists_for_token = Self::whitelists_by_token(token_id);
            whitelists_for_token.push(whitelist.clone());

            //PABLO: TODO: don't add the restriction to the array if it already exists
            <WhitelistsByToken<T>>::insert(token_id,whitelists_for_token);

            <WhitelistForTokenAndAddress<T>>::insert((token_id,_investor),whitelist);

            runtime_io::print("Created restriction!!!");
        }

        pub fn toggle_maximum_percentage_restriction(token_id:u32, enable:bool, max_percentage: u16) {
            <MaximumPercentageEnabledForToken<T>>::insert(token_id,(enable,max_percentage));

            if enable{
                runtime_io::print("Maximum percentage restriction enabled!");
            }else{ 
                runtime_io::print("Maximum percentage restriction disabled!");
            }
        }

        // Transfer restriction verification logic

        pub fn verify_whitelist_restriction(token_id: u32, from: T::AccountId, to: T::AccountId, value: T::TokenBalance) -> (bool,&'static str) {
            let mut _can_transfer = false;
            let now = <timestamp::Module<T>>::get();
            let whitelist_for_from = Self::whitelist_for_restriction((token_id,from));
            let whitelist_for_to = Self::whitelist_for_restriction((token_id,to));
            if (whitelist_for_from.can_send_after > T::Moment::sa(0) && now >= whitelist_for_from.can_send_after) && (whitelist_for_to.can_receive_after > T::Moment::sa(0) && now > whitelist_for_to.can_receive_after) {
                _can_transfer = true;
            }
            (_can_transfer, "Transfer failed: simple restriction in place")
        }

        pub fn verify_totalsupply_percentage(token_id: u32, from: T::AccountId, to: T::AccountId, value: T::TokenBalance, totalSupply: T::TokenBalance) -> (bool,&'static str) {
            let mut _can_transfer = Self::maximum_percentage_enabled_for_token(token_id);
            let enabled = _can_transfer.0;
            // If the restriction is enabled, then we need to make the calculations, otherwise all good
            if enabled {
                (false, "Transfer failed: percentage of total supply surpassed")
            }else{
                (true,"") 
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
