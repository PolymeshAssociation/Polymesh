#![cfg_attr(not(feature = "std"), no_std)]

use rstd::prelude::*;

use support::{decl_event, decl_module, decl_storage, ensure, dispatch::Result};
use system::{ensure_signed, ensure_root};
use codec::{Encode, Decode};

/// The configuration trait.
pub trait Trait: system::Trait + session::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

/// Requirement level for a control point
#[repr(u32)]
#[derive(Encode, Decode, Clone, PartialEq, Debug)]
enum Condition {
    MinorMust,
    MajorMust,
}

/// Represents a requirement that must be met to be eligible to become a validator
#[derive(Encode, Decode, Clone, PartialEq, Debug)]
pub struct ControlPoint {
    what: Vec<u8>,
    condition: Condition,
    complies: bool,
}

decl_storage! {
    trait Store for Module<T: Trait> as CuratedValidation {
        /// Validator -> Compliance criteria
        AllValidators: map T::AccountId => Vec<ControlPoint>;

    }
}

decl_event!(
    pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
        // Validator candidate added. (candidate account)
        CandidateAdded(AccountId),

        // Validator removed from active pool
        ValidatorRemoved(AccountId),
    }
);

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        fn deposit_event() = default;

        // Add a potential validator account. It marks the beginning of compliance process.
        // Add compliance control points that must be met by this validator
        // candidate. Only after passing all control points will a candidate
        // be promoted to a validator pool.
        pub fn add_candidate(origin, account_id: T::AccountId, requirements: Vec<ControlPoint>) -> Result {
            ensure_root(origin)?;

            ensure!(!<AllValidators<T>>::exists(account_id.clone()), "Already a validator.");

            let mut copies = requirements
                .iter()
                .cloned()
                .map(|cp| ControlPoint {
                    what: cp.what.clone(),
                    condition: cp.condition.clone(),
                    complies: false,
                })
                .collect();

            <AllValidators<T>>::insert(account_id.clone(), &mut copies);

            Ok(())
        }

        // Sign and mark whether compliance criteria has been met.
        pub fn report_compliance(origin) -> Result {
            ensure_root(origin)?;



            Ok(())
        }

        // Attempt to promote the candidate to validator pool.
        pub fn promote_candidate(origin, account_id: T::AccountId) -> Result {
            ensure_root(origin)?;



            Self::add_validator(account_id)?;

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {

    // Add new validator and trigger a new session
    fn add_validator(account_id: T::AccountId) -> Result {
        let mut current_validators = <session::Module<T>>::validators();



        Self::deposit_event(RawEvent::CandidateAdded(account_id));
        Ok(())
    }
}


/// tests for this module
#[cfg(test)]
mod tests {
    use sr_io::with_externalities;
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