#![cfg_attr(not(feature = "std"), no_std)]

use rstd::prelude::*;

use crate::identity;
use srml_support::{decl_module, decl_storage};
use system::{ensure_root};
use sr_primitives::{
    traits::{
        StaticLookup
    }
};
use staking;

/// The configuration trait.
pub trait Trait: staking::Trait + identity::Trait {}

decl_storage! {
    trait Store for Module<T: Trait> as Staking {
    }
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		fn deposit_event() = default;

		fn bond(origin,
			controller: <T::Lookup as StaticLookup>::Source,
			#[compact] value: staking::BalanceOf<T>,
			payee: staking::RewardDestination
		) {
			let stash = ensure_signed(origin)?;

			if <Bonded<T>>::exists(&stash) {
				return Err("stash already bonded")
			}

			let controller = T::Lookup::lookup(controller)?;

			if <Ledger<T>>::exists(&controller) {
				return Err("controller already paired")
			}

			// reject a bond which is considered to be _dust_.
			if value < T::Currency::minimum_balance() {
				return Err("can not bond with value less than minimum balance")
			}

			// You're auto-bonded forever, here. We might improve this by only bonding when
			// you actually validate/nominate and remove once you unbond __everything__.
			<Bonded<T>>::insert(&stash, &controller);
			<Payee<T>>::insert(&stash, payee);

			let stash_balance = T::Currency::free_balance(&stash);
			let value = value.min(stash_balance);
			let item = StakingLedger { stash, total: value, active: value, unlocking: vec![] };
			Self::update_ledger(&controller, &item);

            //<staking::Module<T>>::bond(origin, controller, payee);
		}

		fn bond_extra(origin, #[compact] max_additional: staking::BalanceOf<T>) {
		    <staking::Module<T>>::bond_extra(origin, max_additional);
		}

		fn unbond(origin, #[compact] value: staking::BalanceOf<T>) {
		    <staking::Module<T>>::bond(origin, value);
		}

		fn withdraw_unbonded(origin) {
            <staking::Module<T>>::withdraw_unbonded(origin);
		}

		fn validate(origin, prefs: staking::ValidatorPrefs<staking::BalanceOf<T>>) {
            <staking::Module<T>>::validate(origin, prefs);
		}

		fn nominate(origin, targets: Vec<<T::Lookup as StaticLookup>::Source>) {
            <staking::Module<T>>::validate(origin, targets);
		}

		fn chill(origin) {
            <staking::Module<T>>::chill(origin);
		}

		fn set_payee(origin, payee: staking::RewardDestination) {
            <staking::Module<T>>::chill(origin, payee);
		}

		fn set_controller(origin, controller: <T::Lookup as StaticLookup>::Source) {
            <staking::Module<T>>::set_controller(origin, controller);
		}

		fn set_validator_count(origin, #[compact] new: u32) {
            <staking::Module<T>>::set_controller(origin, new);
		}

		// ----- Root calls.

		/// Force there to be no new eras indefinitely.
		///
		/// # <weight>
		/// - No arguments.
		/// # </weight>
		fn force_no_eras(origin) {
			ensure_root(origin)?;
            <staking::Module<T>>::force_no_eras(origin);
		}

		/// Force there to be a new era at the end of the next session. After this, it will be
		/// reset to normal (non-forced) behaviour.
		///
		/// # <weight>
		/// - No arguments.
		/// # </weight>
		fn force_new_era(origin) {
			ensure_root(origin)?;
			<staking::Module<T>>::force_new_era(origin);
		}

		/// Set the validators who cannot be slashed (if any).
		fn set_invulnerables(origin, validators: Vec<T::AccountId>) {
			ensure_root(origin)?;
			<staking::Module<T>>::set_invulnerables(origin, validators);
		}
	}
}

impl<T: Trait> Module<T> {}

/// tests for this module
#[cfg(test)]
mod tests {
    use sr_io::with_externalities;
    use sr_primitives::weights::Weight;
    use sr_primitives::Perbill;
    use sr_primitives::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
    };
    use support::{assert_ok, impl_outer_origin, parameter_types};

    use primitives::{Blake2Hasher, H256};

    use super::*;

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }
    impl system::Trait for Test {
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type WeightMultiplierUpdate = ();
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
    }

    impl Trait for Test {
        type Event = ();
    }

    type CuratedValidation = Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap()
            .into()
    }

    #[test]
    fn it_works_for_default_value() {
        with_externalities(&mut new_test_ext(), || {
            assert_ok!(CuratedValidation::do_something(Origin::signed(1), 42));
        });

    }
}