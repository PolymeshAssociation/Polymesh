use rstd::prelude::*;
//use parity_codec::Codec;
use runtime_primitives::traits::{As, CheckedAdd, CheckedSub};
use support::{
    decl_event, decl_module, decl_storage, dispatch::Result, ensure, StorageMap, StorageValue,
};
use system::{self, ensure_signed};

#[derive(parity_codec::Encode, parity_codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct Issuer<U> {
    account: U,
    access_level: u16,
    active: bool,
}

#[derive(parity_codec::Encode, parity_codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct Investor<U> {
    pub account: U,
    pub access_level: u16,
    pub active: bool,
    pub jurisdiction: u16,
}

/// The module's configuration trait.
pub trait Trait: system::Trait {
    // TODO: Add other types and constants required configure this module.

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as identity {

        Owner get(owner) config(): T::AccountId;

        ERC20IssuerList get(erc20_issuer_list): map T::AccountId => Issuer<T::AccountId>;
        IssuerList get(issuer_list): map T::AccountId => Issuer<T::AccountId>;
        pub InvestorList get(investor_list): map T::AccountId => Investor<T::AccountId>;

    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event<T>() = default;

        fn create_issuer(origin,_issuer: T::AccountId) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(Self::owner() == sender,"Sender must be the identity module owner");

            Self::do_create_issuer(_issuer)
        }

        fn create_erc20_issuer(origin,_issuer: T::AccountId) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(Self::owner() == sender,"Sender must be the identity module owner");

            Self::do_create_erc20_issuer(_issuer)
        }

        fn create_investor(origin,_investor: T::AccountId) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(Self::owner() == sender,"Sender must be the identity module owner");

            Self::do_create_investor(_investor)
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        // Just a dummy event.
        // Event `Something` is declared with a parameter of the type `u32` and `AccountId`
        // To emit this event, we call the deposit funtion, from our runtime funtions
        SomethingStored(u32, AccountId),
    }
);

pub trait IdentityTrait<T> {
    fn investor_data(who: T) -> Investor<T>;
}

impl<T: Trait> IdentityTrait<T::AccountId> for Module<T> {
    fn investor_data(sender: T::AccountId) -> Investor<T::AccountId> {
        let _investor = Self::investor_list(sender);
        _investor
    }
}

impl<T: Trait> Module<T> {
    /// Add a new issuer. Warning: No identity module ownership checks are performed
    pub fn do_create_issuer(_issuer: T::AccountId) -> Result {
        let new_issuer = Issuer {
            account: _issuer.clone(),
            access_level: 1,
            active: true,
        };

        <IssuerList<T>>::insert(_issuer, new_issuer);
        Ok(())
    }

    /// Add a new ERC20 issuer. Warning: No identity module ownership checks are performed
    pub fn do_create_erc20_issuer(_erc20_issuer: T::AccountId) -> Result {
        let new_erc20_issuer = Issuer {
            account: _erc20_issuer.clone(),
            access_level: 1,
            active: true,
        };

        <ERC20IssuerList<T>>::insert(_erc20_issuer, new_erc20_issuer);
        Ok(())
    }

    /// Add a new investor. Warning: No identity module ownership checks are performed
    pub fn do_create_investor(_investor: T::AccountId) -> Result {
        let new_investor = Investor {
            account: _investor.clone(),
            access_level: 1,
            active: true,
            jurisdiction: 1,
        };

        <InvestorList<T>>::insert(_investor, new_investor);
        Ok(())
    }

    pub fn is_issuer(_user: T::AccountId) -> bool {
        let user = Self::issuer_list(_user.clone());
        user.account == _user && user.access_level == 1 && user.active
    }

    pub fn is_erc20_issuer(_user: T::AccountId) -> bool {
        let user = Self::erc20_issuer_list(_user.clone());
        user.account == _user && user.access_level == 1 && user.active
    }

    pub fn is_investor(_user: T::AccountId) -> bool {
        let user = Self::investor_list(_user.clone());
        user.account == _user && user.access_level == 1 && user.active
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    /*
     *    use super::*;
     *
     *    use primitives::{Blake2Hasher, H256};
     *    use runtime_io::with_externalities;
     *    use runtime_primitives::{
     *        testing::{Digest, DigestItem, Header},
     *        traits::{BlakeTwo256, IdentityLookup},
     *        BuildStorage,
     *    };
     *    use support::{assert_ok, impl_outer_origin};
     *
     *    impl_outer_origin! {
     *        pub enum Origin for Test {}
     *    }
     *
     *    // For testing the module, we construct most of a mock runtime. This means
     *    // first constructing a configuration type (`Test`) which `impl`s each of the
     *    // configuration traits of modules we want to use.
     *    #[derive(Clone, Eq, PartialEq)]
     *    pub struct Test;
     *    impl system::Trait for Test {
     *        type Origin = Origin;
     *        type Index = u64;
     *        type BlockNumber = u64;
     *        type Hash = H256;
     *        type Hashing = BlakeTwo256;
     *        type Digest = Digest;
     *        type AccountId = u64;
     *        type Lookup = IdentityLookup<Self::AccountId>;
     *        type Header = Header;
     *        type Event = ();
     *        type Log = DigestItem;
     *    }
     *    impl Trait for Test {
     *        type Event = ();
     *    }
     *    type identity = Module<Test>;
     *
     *    // This function basically just builds a genesis storage key/value store according to
     *    // our desired mockup.
     *    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
     *        system::GenesisConfig::<Test>::default()
     *            .build_storage()
     *            .unwrap()
     *            .0
     *            .into()
     *    }
     *
     *    #[test]
     *    fn it_works_for_default_value() {
     *        with_externalities(&mut new_test_ext(), || {
     *            // Just a dummy test for the dummy funtion `do_something`
     *            // calling the `do_something` function with a value 42
     *            assert_ok!(identity::do_something(Origin::signed(1), 42));
     *            // asserting that the stored value is equal to what we stored
     *            assert_eq!(identity::something(), Some(42));
     *        });
     *    }
     */
}
