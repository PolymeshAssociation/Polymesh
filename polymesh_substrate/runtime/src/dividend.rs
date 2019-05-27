use rstd::prelude::*;
/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs
use runtime_primitives::traits::{As, CheckedDiv, CheckedMul, CheckedSub};
use support::{
    decl_event, decl_module, decl_storage,
    dispatch::Result,
    ensure,
    traits::{Currency, ExistenceRequirement, WithdrawReason},
    StorageMap,
};
use system::ensure_signed;

use crate::{asset, utils};

/// The module's configuration trait.
pub trait Trait: asset::Trait + balances::Trait + system::Trait + utils::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[derive(parity_codec::Encode, parity_codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct Dividend<U> {
    /// Total amount to be distributed
    amount: U,
    /// Whether claiming dividends is already enabled
    payout_started: bool,
    /// The payout currency ticker. None means POLY
    payout_currency: Option<Vec<u8>>,
    /// The checkpoint
    checkpoint_id: u32,
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as dividend {
        // Dividend records; (ticker, checkpoint ID) => dividend entries, index is dividend ID
        Dividends get(dividends): map (Vec<u8>, u32) => Vec<Dividend<T::TokenBalance>>;
        // Payout flags, decide whether a user already was paid their dividend
        // (who, ticker, checkpoint_id, dividend_id)
        PayoutCompleted get(payout_completed): map (T::AccountId, Vec<u8>, u32, u32) => bool;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event<T>() = default;

        /// Creates a new dividend entry without payout
        pub fn new(origin,
                   amount: T::TokenBalance,
                   ticker: Vec<u8>,
                   payout_currency: Option<Vec<u8>>,
                   checkpoint_id: u32
                  ) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender owns the asset token
            ensure!(<asset::Module<T>>::_is_owner(ticker.clone(), sender.clone()), "User is not the owner of the asset");

            // Check if sender has enough funds in payout currency
            let balance = if let Some(payout_ticker) = payout_currency.as_ref() {
                // Check for token
                <asset::BalanceOf<T>>::get((payout_ticker.clone(), sender.clone()))
            } else {
                // Check for POLY
                <T::TokenBalance as As<T::Balance>>::sa(<balances::FreeBalance<T>>::get(sender.clone()))
            };
            ensure!(balance >= amount, "Insufficient funds for payout");

            // Check if checkpoint exists
            ensure!(<asset::Module<T>>::total_checkpoints_of(ticker.clone()) > checkpoint_id,
                    "Checkpoint for dividend does not exist");

            // Subtract the amount
            if let Some(payout_ticker) = payout_currency.as_ref() {
                let new_balance = balance.checked_sub(&amount).ok_or("Overflow calculating new owner balance")?;
                <asset::BalanceOf<T>>::insert((payout_ticker.clone(), sender.clone()), new_balance);
            } else {
                let _imbalance = <balances::Module<T> as Currency<_>>::withdraw(
                    &sender,
                    <T::TokenBalance as As<T::Balance>>::as_(amount),
                    WithdrawReason::Reserve,
                    ExistenceRequirement::KeepAlive)?;
            }

            // Insert dividend entry into storage
            let new_dividend = Dividend {
                amount,
                payout_started: false,
                payout_currency: payout_currency.clone(),
                checkpoint_id,
            };

            let dividend_id = Self::add_dividend_entry((ticker.clone(), checkpoint_id),new_dividend)?;

            // Dispatch event
            Self::deposit_event(RawEvent::DividendCreated(ticker, amount, checkpoint_id, dividend_id));

            Ok(())
        }

        /// Enables withdrawal of dividend funds for asset `ticker`.
        pub fn start_payout(origin, ticker: Vec<u8>, checkpoint_id: u32, dividend_id: u32) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender owns the asset token
            ensure!(<asset::Module<T>>::_is_owner(ticker.clone(), sender.clone()), "User is not the owner of the asset");

            // Check that the dividend exists
            ensure!(<Dividends<T>>::exists((ticker.clone(), checkpoint_id)), "No dividend entries for supplied ticker and checkpoint");

            // Flip `payout_started`
            <Dividends<T>>::mutate((ticker.clone(), checkpoint_id), |entries| {
                if let Some(entry) = entries.get_mut(dividend_id as usize) {
                    entry.payout_started = true;
                    Ok(())
                } else {
                    Err("No dividend entry for supplied dividend id")
                }
            })?;

            // Dispatch event
            Self::deposit_event(RawEvent::DividendPayoutStarted(ticker.clone(), checkpoint_id, dividend_id));

            Ok(())
        }

        /// Withdraws from a dividend the adequate share of the `amount` field. All dividend shares
        /// are rounded by truncation (down to first integer below)
        pub fn claim_payout(origin, ticker: Vec<u8>, checkpoint_id: u32, dividend_id: u32) -> Result {
            let sender = ensure_signed(origin)?;

            // Check if sender wasn't already paid their share
            ensure!(!<PayoutCompleted<T>>::get((sender.clone(), ticker.clone(), checkpoint_id, dividend_id)), "User was already paid their share");

            // Check if sender owned the token at checkpoint
            let balance_at_checkpoint =
                <asset::Module<T>>::balance_at_checkpoint((ticker.clone(), sender.clone(), checkpoint_id))
                .ok_or("Sender did not own the token at checkpoint")?;

            // Look dividend entry up
            let dividend = Self::get_dividend(ticker.clone(), checkpoint_id, dividend_id).ok_or("Dividend not found")?;

            // Compute the share
            ensure!(<asset::Tokens<T>>::exists(ticker.clone()), "Dividend token entry not found");
            let token = <asset::Tokens<T>>::get(ticker.clone());

            let balance_amount_product = balance_at_checkpoint
                .checked_mul(&dividend.amount)
                .ok_or("multiplying balance and total payout amount failed")?;

            let share = balance_amount_product
                .checked_div(&token.total_supply)
                .ok_or("balance_amount_product division failed")?;

            // Perform the payout in designated tokens or base currency depending on setting
            if let Some(payout_ticker) = dividend.payout_currency.as_ref() {
                <asset::Module<T>>::_mint(payout_ticker.clone(), sender.clone(), share)?;
            } else {
                // Convert to balances::Trait::Balance
                let share = <T::TokenBalance as As<T::Balance>>::as_(share);
                let _imbalance = <balances::Module<T> as Currency<_>>::deposit_into_existing(&sender, share)?;
            }
            // Create payout entry
            <PayoutCompleted<T>>::insert((sender.clone(), ticker.clone(), checkpoint_id, dividend_id), true);

            // Dispatch event
            Self::deposit_event(RawEvent::DividendPaidOut(sender.clone(), ticker.clone(), checkpoint_id, dividend_id, share));
            Ok(())
        }

    }
}

decl_event!(
    pub enum Event<T>
    where
        TokenBalance = <T as utils::Trait>::TokenBalance,
        AccountId = <T as system::Trait>::AccountId,
    {
        // ticker, amount, checkpoint ID, dividend ID
        DividendCreated(Vec<u8>, TokenBalance, u32, u32),

        // ticker, checkpoint ID, dividend ID
        DividendPayoutStarted(Vec<u8>, u32, u32),

        // who, ticker, checkpoint_id, dividend_id, share
        DividendPaidOut(AccountId, Vec<u8>, u32, u32, TokenBalance),
    }
);

impl<T: Trait> Module<T> {
    /// A helper method for dividend creation. The `mutate` return type would become pretty ugly
    /// otherwise.
    #[inline]
    fn add_dividend_entry(
        key: (Vec<u8>, u32),
        d: Dividend<T::TokenBalance>,
    ) -> core::result::Result<u32, &'static str> {
        <Dividends<T>>::mutate(key, |entries| {
            entries.push(d);
            Ok((entries.len() - 1) as u32)
        })
    }
    /// Quick lookup of a dividend entry
    pub fn get_dividend(
        ticker: Vec<u8>,
        checkpoint_id: u32,
        dividend_id: u32,
    ) -> Option<Dividend<T::TokenBalance>> {
        let entries = <Dividends<T>>::get((ticker, checkpoint_id));

        entries.get(dividend_id as usize).map(|d| d.clone())
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
     *    type dividend = Module<Test>;
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
     *            assert_ok!(dividend::do_something(Origin::signed(1), 42));
     *            // asserting that the stored value is equal to what we stored
     *            assert_eq!(dividend::something(), Some(42));
     *        });
     *    }
     */
}
