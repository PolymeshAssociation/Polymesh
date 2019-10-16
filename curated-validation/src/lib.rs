#![cfg_attr(not(feature = "std"), no_std)]

use support::{decl_event, decl_module, decl_storage, dispatch::Result};
use system::ensure_signed;

/// The configuration trait.
pub trait Trait: system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as CuratedValidation {
        pub ValidatorCount get(validator_count) config(): u32;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;
    }
}

decl_event!(
    pub enum Event<T> where
    AccountId = <T as system::Trait>::AccountId {
        ValidatorAdded(AccountId),

        ValidatorRemoved(AccountId),
    }
);

/// tests for this module
#[cfg(test)]
mod tests {
    use runtime_io::with_externalities;
    use sr_primitives::{testing::Header, traits::{BlakeTwo256, IdentityLookup}};
    use sr_primitives::Perbill;
    use sr_primitives::weights::Weight;
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
        system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
    }

    #[test]
    fn it_works_for_default_value() {
        with_externalities(&mut new_test_ext(), || {
            assert_ok!(CuratedValidation::do_something(Origin::signed(1), 42));
        });
    }
}