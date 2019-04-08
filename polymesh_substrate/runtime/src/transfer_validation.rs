use crate::asset;
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

// struct to store the token details
#[derive(parity_codec::Encode, parity_codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct Restriction {
    name: Vec<u8>,
    token_id: u32,
    can_transfer: bool
}

#[derive(parity_codec::Encode, parity_codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct Whitelist<U> {
    canSendAfter: U,
    canReceiveAfter: U
}

/// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as TransferValidation {
		// Just a dummy storage item. 
		// Here we are declaring a StorageValue, `Something` as a Option<u32>
		// `get(something)` is the default getter which returns either the stored `u32` or `None` if nothing stored
        RestrictionId get(restriction_id): u32;
		Something get(something): Option<u32>;
        Restrictions get(transfer_restrictions): map u32 => Restriction<>;
        RestrictionsByToken get(restriction_by_token): map u32 => Vec<Restriction>;
        
        //WhitelistForRestriction get(whitelist_for_restriction): map u32 => T::AccountId => Vec<Whitelist<T::Moment>>;
	}
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event<T>() = default;

        //Creates a new restriction for a token
        //Can only be done by token owner TBD
        fn create_restriction(origin, name:Vec<u8>, token_id:u32) -> Result {
            let sender = ensure_signed(origin)?;

            let restriction_id = Self::restriction_id();
            let next_restriction_id = restriction_id.checked_add(1).ok_or("overflow in calculating next restriction id")?;
            <RestrictionId<T>>::put(next_restriction_id);

            let restriction = Restriction {
                name,
                token_id,
                can_transfer:false,
            };

            let mut restrictions_for_token = Self::restriction_by_token(token_id);
            restrictions_for_token.push(restriction.clone());

            <Restrictions<T>>::insert(restriction_id, restriction);
            <RestrictionsByToken<T>>::insert(token_id,restrictions_for_token);

            runtime_io::print("Created restriction!!!");

            Ok(())
        }
        
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
        Example(u32, AccountId, AccountId),
	}
);

impl<T: Trait> Module<T> {

        pub fn verifyRestrictions(token_id: u32, from: T::AccountId, to: T::AccountId) -> (bool,&'static str) {
            let mut _can_transfer = true;
            let restrictions_for_token = Self::restriction_by_token(token_id);
            for i in 0..restrictions_for_token.len() {
                if !restrictions_for_token[i].can_transfer {_can_transfer = false;}
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
