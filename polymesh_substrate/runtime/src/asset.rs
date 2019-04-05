/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references


/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs

use rstd::prelude::*;
use parity_codec::Codec;
use runtime_io;
use runtime_primitives::traits::{As, Member,SimpleArithmetic, CheckedSub, CheckedAdd, CheckedDiv, CheckedMul, Hash};
use support::{Parameter,decl_module, decl_storage, decl_event, StorageMap, StorageValue, dispatch::Result, ensure};
use system::ensure_signed;

/// The module's configuration trait.
pub trait Trait: system::Trait {
	// TODO: Add other types and constants required configure this module.

	/// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type TokenBalance: Parameter + Member + SimpleArithmetic + Codec + Default + Copy + As<usize> + As<u64>;
}

/// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as Asset {
        Init get(is_init): bool;
       
        Symbol: Vec<u8>;
        // total supply of the token
        //TotalSupply get(total_supply): T::TokenBalance;
        TotalSupply get(total_supply): T::TokenBalance;
        // mapping of balances to accounts
        //BalanceOf get(balance_of): map T::AccountId => T::TokenBalance;
        BalanceOf get(balance_of): map T::AccountId => T::TokenBalance;
    }
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event<T>() = default;

        // initialize the token
        // transfers the total_supply amout to the caller
        // the token becomes usable
        // not part of ERC20 standard interface
        // similar to the ERC20 smart contract constructor
        pub fn init(origin, sender: T::AccountId, initial_supply: T::TokenBalance) -> Result {
            let _sender = ensure_signed(origin)?;
            ensure!(Self::is_init() == false, "Token already initialized.");

            <BalanceOf<T>>::insert(sender, initial_supply);
            <Init<T>>::put(true);
            <TotalSupply<T>>::put(initial_supply);

            runtime_io::print("Initialized!!!");

            Ok(())
        }

        // transfer tokens from one account to another
        pub fn transfer(origin, to: T::AccountId, #[compact] value: T::TokenBalance) -> Result {
            let _sender = ensure_signed(origin)?;
            Self::_transfer(_sender, to, value)
        }

        pub fn set_symbol(origin, value: Vec<u8>) -> Result {
            let _sender = ensure_signed(origin)?;

            <Symbol<T>>::put(value);

            // some bytes, in a vector
            let polymesh = vec![80, 111, 108, 121, 109, 101, 115, 104];
            <Symbol<T>>::put(polymesh);

            Ok(())
        }   
	}
}

decl_event!(
    pub enum Event<T> where AccountId = <T as system::Trait>::AccountId, TokenBalance = <T as self::Trait>::TokenBalance {
        // event for transfer of tokens
        // from, to, value
        Transfer(AccountId, AccountId, TokenBalance),
        // event when an approval is made
        // owner, spender, value
        //Approval(AccountId, AccountId, TokenBalance),
    }
);

/// All functions in the decl_module macro become part of the public interface of the module
/// If they are there, they are accessible via extrinsics calls whether they are public or not
/// However, in the impl module section (this, below) the functions can be public and private
/// Private functions are internal to this module e.g.: _transfer
/// Public functions can be called from other modules e.g.: lock and unlock (being called from the tcr module)
/// All functions in the impl module section are not part of public interface because they are not part of the Call enum
impl<T: Trait> Module<T> {

    // internal transfer function for ERC20 interface
    fn _transfer(
        from: T::AccountId,
        to: T::AccountId,
        value: T::TokenBalance,
    ) -> Result {
        ensure!(<BalanceOf<T>>::exists(from.clone()), "Account does not own this token");
        let sender_balance = Self::balance_of(from.clone());
        ensure!(sender_balance >= value, "Not enough balance.");
        let updated_from_balance = sender_balance.checked_sub(&value).ok_or("overflow in calculating balance")?;
        let receiver_balance = Self::balance_of(to.clone());
        let updated_to_balance = receiver_balance.checked_add(&value).ok_or("overflow in calculating balance")?;
        
        // reduce sender's balance
        <BalanceOf<T>>::insert(from.clone(), updated_from_balance);

        // increase receiver's balance
        <BalanceOf<T>>::insert(to.clone(), updated_to_balance);

        Self::deposit_event(RawEvent::Transfer(from, to, value));
        Ok(())
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
	type asset = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
	}

	//#[test]
	// fn it_works_for_default_value() {
	// 	with_externalities(&mut new_test_ext(), || {
	// 		// Just a dummy test for the dummy funtion `do_something`
	// 		// calling the `do_something` function with a value 42
	// 		assert_ok!(asset::do_something(Origin::signed(1), 42));
	// 		// asserting that the stored value is equal to what we stored
	// 		assert_eq!(asset::something(), Some(42));
	// 	});
	// }
}
