use crate::asset::{self, AssetTrait};
use crate::identity::{self, InvestorList};
use crate::utils;

use rstd::prelude::*;
use srml_support::{
    decl_event, decl_module, decl_storage, dispatch::Result, ensure, StorageMap, StorageValue,
};
use system::{self, ensure_signed};

/// The module's configuration trait.
pub trait Trait: timestamp::Trait + system::Trait + utils::Trait + identity::Trait {
    // TODO: Add other types and constants required configure this module.

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Asset: asset::AssetTrait<Self::AccountId, Self::TokenBalance>;
}

#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct Whitelist<U, V> {
    investor: V,
    can_send_after: U,
    can_receive_after: U,
}

decl_storage! {
    trait Store for Module<T: Trait> as GeneralTM {

        // Tokens can have multiple whitelists that (for now) check entries individually within each other
        WhitelistsByToken get(whitelists_by_token): map (Vec<u8>, u32) => Vec<Whitelist<T::Moment, T::AccountId>>;

        WhitelistForTokenAndAddress get(whitelist_for_restriction): map (Vec<u8>, u32, T::AccountId) => Whitelist<T::Moment, T::AccountId>;

        WhitelistEntriesCount get(whitelist_entries_count): map (Vec<u8>,u32) => u64;
        WhitelistCount get(whitelist_count): u32;

    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event() = default;

        pub fn add_to_whitelist(origin, _ticker: Vec<u8>, whitelist_id: u32, _investor: T::AccountId, expiry: T::Moment) -> Result {
            let sender = ensure_signed(origin)?;
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            ensure!(Self::is_owner(ticker.clone(),sender.clone()),"Sender must be the token owner");

            let whitelist = Whitelist {
                investor: _investor.clone(),
                can_send_after:expiry.clone(),
                can_receive_after:expiry
            };

            //Get whitelist entries for this token + whitelistId
            let mut whitelists_for_token = Self::whitelists_by_token((ticker.clone(), whitelist_id.clone()));

            //Get how many entries this whiteslist has and increase it if we are adding a new entry
            let entries_count = Self::whitelist_entries_count((ticker.clone(), whitelist_id.clone()));

            // TODO: Make sure we are only increasing the count if it's a new entry and not just an update of an existing entry
            let new_entries_count = entries_count.checked_add(1).ok_or("overflow in calculating next entry count")?;
            <WhitelistEntriesCount>::insert((ticker.clone(), whitelist_id),new_entries_count);

            // If this is the first entry for this whitelist, increase the whitelists count so then we can loop through them.
            if new_entries_count == 1 {
                let whitelist_count = Self::whitelist_count();
                let new_whitelist_count = whitelist_count.checked_add(1).ok_or("overflow in calculating next whitelist count")?;
                <WhitelistCount>::put(new_whitelist_count);
            }

            whitelists_for_token.push(whitelist.clone());

            //PABLO: TODO: don't add the restriction to the array if it already exists
            <WhitelistsByToken<T>>::insert((ticker.clone(), whitelist_id.clone()), whitelists_for_token);

            <WhitelistForTokenAndAddress<T>>::insert((ticker.clone(), whitelist_id, _investor),whitelist);

            sr_primitives::print("Created restriction!!!");
            //<general_tm::Module<T>>::add_to_whitelist(sender,token_id,_investor,expiry);

            Ok(())
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        Example(u32, AccountId, AccountId),
    }
);

impl<T: Trait> Module<T> {
    pub fn is_owner(_ticker: Vec<u8>, sender: T::AccountId) -> bool {
        let ticker = utils::bytes_to_upper(_ticker.as_slice());
        T::Asset::is_owner(ticker.clone(), sender)
        // let token = T::Asset::token_details(token_id);
        // token.owner == sender
    }

    ///  Sender restriction verification
    pub fn verify_restriction(
        _ticker: Vec<u8>,
        from: T::AccountId,
        to: T::AccountId,
        _value: T::TokenBalance,
    ) -> Result {
        let ticker = utils::bytes_to_upper(_ticker.as_slice());
        let now = <timestamp::Module<T>>::get();
        // issuance case
        if from == T::AccountId::default() {
            ensure!(
                Self::_check_investor_status(to.clone()).is_ok(),
                "Account is not active"
            );
            ensure!(
                Self::is_whitelisted(_ticker.clone(), to).is_ok(),
                "to account is not whitelisted"
            );
            sr_primitives::print("GTM: Passed from the issuance case");
            return Ok(());
        } else if to == T::AccountId::default() {
            // burn case
            ensure!(
                Self::_check_investor_status(from.clone()).is_ok(),
                "Account is not active"
            );
            ensure!(
                Self::is_whitelisted(_ticker.clone(), from).is_ok(),
                "from account is not whitelisted"
            );
            sr_primitives::print("GTM: Passed from the burn case");
            return Ok(());
        } else {
            // loop through existing whitelists
            let whitelist_count = Self::whitelist_count();
            ensure!(
                Self::_check_investor_status(from.clone()).is_ok(),
                "Account is not active"
            );
            ensure!(
                Self::_check_investor_status(to.clone()).is_ok(),
                "Account is not active"
            );
            for x in 0..whitelist_count {
                let whitelist_for_from =
                    Self::whitelist_for_restriction((ticker.clone(), x, from.clone()));
                let whitelist_for_to =
                    Self::whitelist_for_restriction((ticker.clone(), x, to.clone()));

                if (whitelist_for_from.can_send_after > 0.into()
                    && now >= whitelist_for_from.can_send_after)
                    && (whitelist_for_to.can_receive_after > 0.into()
                        && now > whitelist_for_to.can_receive_after)
                {
                    return Ok(());
                }
            }
        }
        sr_primitives::print("GTM: Not going through the restriction");
        Err("Cannot Transfer: General TM restrictions not satisfied")
    }

    pub fn is_whitelisted(_ticker: Vec<u8>, holder: T::AccountId) -> Result {
        let ticker = utils::bytes_to_upper(_ticker.as_slice());
        let now = <timestamp::Module<T>>::get();
        ensure!(
            Self::_check_investor_status(holder.clone()).is_ok(),
            "Account is not active"
        );
        // loop through existing whitelists
        let whitelist_count = Self::whitelist_count();

        for x in 0..whitelist_count {
            let whitelist_for_holder =
                Self::whitelist_for_restriction((ticker.clone(), x, holder.clone()));

            if whitelist_for_holder.can_send_after > 0.into()
                && now >= whitelist_for_holder.can_send_after
            {
                return Ok(());
            }
        }
        Err("Not whitelisted")
    }

    fn _check_investor_status(holder: T::AccountId) -> Result {
        let investor = <InvestorList<T>>::get(holder.clone());
        ensure!(
            investor.active && investor.access_level == 1,
            "From account is not active"
        );
        Ok(())
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    /*
     *    use super::*;
     *
     *    use primitives::{Blake2Hasher, H256};
     *    use sr_io::with_externalities;
     *    use sr_primitives::{
     *        testing::{Digest, DigestItem, Header},
     *        traits::{BlakeTwo256, IdentityLookup},
     *        BuildStorage,
     *    };
     *    use srml_support::{assert_ok, impl_outer_origin};
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
     *    type TransferValidationModule = Module<Test>;
     *
     *    // This function basically just builds a genesis storage key/value store according to
     *    // our desired mockup.
     *    fn new_test_ext() -> sr_io::TestExternalities<Blake2Hasher> {
     *        system::GenesisConfig::<Test>::default()
     *            .build_storage()
     *            .unwrap()
     *            .0
     *            .into()
     *    }
     */
}
