use rstd::prelude::*;
/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs
use runtime_primitives::traits::{As, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};
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
pub trait Trait:
    asset::Trait + balances::Trait + system::Trait + utils::Trait + timestamp::Trait
{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[derive(parity_codec::Encode, parity_codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct Dividend<U, V> {
    /// Total amount to be distributed
    amount: U,
    /// Whether claiming dividends is enabled
    active: bool,
    /// An optional timestamp of payout start
    maturates_at: Option<V>,
    /// An optional timestamp for payout end
    expires_at: Option<V>,
    /// The payout currency ticker. None means POLY
    payout_currency: Option<Vec<u8>>,
    /// The checkpoint
    checkpoint_id: u32,
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as dividend {
        // Dividend records; (ticker, checkpoint ID) => dividend entries, index is dividend ID
        Dividends get(dividends): map (Vec<u8>, u32) => Vec<Dividend<T::TokenBalance, T::Moment>>;
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
                   maturates_at: Option<T::Moment>,
                   expires_at: Option<T::Moment>,
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

            let now = <timestamp::Module<T>>::get();

            // Check maturation/expiration dates
            match (maturates_at.as_ref(), expires_at.as_ref()) {
                (Some(start), Some(end))=> {
                    // Ends in the future
                    ensure!(*end > now, "Dividend payout should end in the future");
                    // Ends after start
                    ensure!(*end > *start, "Dividend payout must end after it starts");
                },
                (Some(_start), None) => {
                },
                (None, Some(end)) => {
                    // Ends in the future
                    ensure!(*end > now, "Dividend payout should end in the future");
                },
                (None, None) => {}
            }

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
                active: false,
                maturates_at,
                expires_at,
                payout_currency: payout_currency.clone(),
                checkpoint_id,
            };

            let dividend_id = Self::add_dividend_entry((ticker.clone(), checkpoint_id),new_dividend)?;

            // Dispatch event
            Self::deposit_event(RawEvent::DividendCreated(ticker, amount, checkpoint_id, dividend_id));

            Ok(())
        }

        /// Enables withdrawal of dividend funds for asset `ticker`.
        pub fn activate(origin, ticker: Vec<u8>, checkpoint_id: u32, dividend_id: u32) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender owns the asset token
            ensure!(<asset::Module<T>>::_is_owner(ticker.clone(), sender.clone()), "User is not the owner of the asset");

            // Check that the dividend exists
            ensure!(<Dividends<T>>::exists((ticker.clone(), checkpoint_id)), "No dividend entries for supplied ticker and checkpoint");

            // Flip `active`
            <Dividends<T>>::mutate((ticker.clone(), checkpoint_id), |entries| {
                if let Some(entry) = entries.get_mut(dividend_id as usize) {
                    entry.active = true;
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
        pub fn claim(origin, ticker: Vec<u8>, checkpoint_id: u32, dividend_id: u32) -> Result {
            let sender = ensure_signed(origin)?;

            // Check if sender wasn't already paid their share
            ensure!(!<PayoutCompleted<T>>::get((sender.clone(), ticker.clone(), checkpoint_id, dividend_id)), "User was already paid their share");

            let balance_at_checkpoint =
                <asset::Module<T>>::get_balance_at(ticker.clone(), sender.clone(), checkpoint_id);

            // Look dividend entry up
            let dividend = Self::get_dividend(ticker.clone(), checkpoint_id, dividend_id).ok_or("Dividend not found")?;

            // Check if the dividend is active
            ensure!(dividend.active, "Dividend not active");

            let now = <timestamp::Module<T>>::get();

            // Check if the current time is within maturation/expiration bounds
            if let Some(start) = dividend.maturates_at.as_ref() {
                ensure!(now > *start, "Attempted payout before maturation");
            }

            if let Some(end) = dividend.expires_at.as_ref() {
                ensure!(*end > now, "Attempted payout after expiration");
            }

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
                <asset::BalanceOf<T>>::mutate(
                    (payout_ticker.clone(), sender.clone()),
                    |balance| -> Result {
                        *balance = balance
                            .checked_add(&share)
                            .ok_or("Could not add share to sender balance")?;
                        Ok(())
                    })?;

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
        d: Dividend<T::TokenBalance, T::Moment>,
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
    ) -> Option<Dividend<T::TokenBalance, T::Moment>> {
        let entries = <Dividends<T>>::get((ticker, checkpoint_id));

        entries.get(dividend_id as usize).map(|d| d.clone())
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{prelude::*, Duration};
    use lazy_static::lazy_static;
    use primitives::{Blake2Hasher, H256};
    use runtime_io::with_externalities;
    use runtime_primitives::{
        testing::{Digest, DigestItem, Header, UintAuthorityId},
        traits::{BlakeTwo256, Convert, IdentityLookup},
        BuildStorage,
    };
    use support::{assert_ok, impl_outer_origin};

    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use crate::{asset::SecurityToken, general_tm, identity, percentage_tm};

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    pub struct CurrencyToBalanceHandler;

    impl Convert<u128, u128> for CurrencyToBalanceHandler {
        fn convert(x: u128) -> u128 {
            x
        }
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

    impl asset::Trait for Test {
        type Event = ();
        type Currency = balances::Module<Test>;
        type TokenFeeCharge = ();
        type CurrencyToBalance = CurrencyToBalanceHandler;
    }

    impl balances::Trait for Test {
        type Balance = u128;
        type DustRemoval = ();
        type Event = ();
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type TransactionPayment = ();
        type TransferPayment = ();
    }

    impl consensus::Trait for Test {
        type Log = DigestItem;
        type SessionKey = UintAuthorityId;
        type InherentOfflineReport = ();
    }
    impl general_tm::Trait for Test {
        type Event = ();
        type Asset = Module<Test>;
    }

    impl identity::Trait for Test {
        type Event = ();
    }

    impl percentage_tm::Trait for Test {
        type Event = ();
        type Asset = Module<Test>;
    }

    impl session::Trait for Test {
        type ConvertAccountIdToSessionKey = ();
        type Event = ();
        type OnSessionChange = ();
    }

    impl timestamp::Trait for Test {
        type Moment = u64;
        type OnTimestampSet = ();
    }

    impl utils::Trait for Test {
        type TokenBalance = u128;
    }

    impl Trait for Test {
        type Event = ();
    }
    impl asset::HasOwner<<Test as system::Trait>::AccountId> for Module<Test> {
        fn is_owner(_ticker: Vec<u8>, sender: <Test as system::Trait>::AccountId) -> bool {
            if let Some(token) = TOKEN_MAP.lock().unwrap().get(&_ticker) {
                token.owner == sender
            } else {
                false
            }
        }
    }

    lazy_static! {
        static ref TOKEN_MAP: Arc<
            Mutex<
            HashMap<
            Vec<u8>,
            SecurityToken<
                <Test as utils::Trait>::TokenBalance,
                <Test as system::Trait>::AccountId,
                >,
                >,
                >,
                > = Arc::new(Mutex::new(HashMap::new()));
        /// Because Rust's Mutex is not recursive a second symbolic lock is necessary
        static ref TOKEN_MAP_OUTER_LOCK: Arc<Mutex<()>> = Arc::new(Mutex::new(()));
    }

    type DividendModule = Module<Test>;
    type Balances = balances::Module<Test>;
    type Asset = asset::Module<Test>;

    /// Build a genesis identity instance owned by the specified account
    fn identity_owned_by(id: u64) -> runtime_io::TestExternalities<Blake2Hasher> {
        let mut t = system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap()
            .0;
        t.extend(
            identity::GenesisConfig::<Test> { owner: id }
                .build_storage()
                .unwrap()
                .0,
        );
        t.into()
    }

    #[test]
    fn correct_dividend_must_work() {
        let identity_owner_id = 1;
        with_externalities(&mut identity_owned_by(identity_owner_id), || {
            // A token representing 1M shares
            let token = SecurityToken {
                name: vec![0x01],
                owner: 1,
                total_supply: 1_000_000,
            };

            // A token used for payout
            let payout_token = SecurityToken {
                name: vec![0x02],
                owner: 2,
                total_supply: 200_000_000,
            };

            // Raise the owners' base currency balance
            Balances::make_free_balance_be(&token.owner, 1_000_000);
            Balances::make_free_balance_be(&payout_token.owner, 1_000_000);

            identity::Module::<Test>::do_create_issuer(token.owner)
                .expect("Could not make token.owner an issuer");
            identity::Module::<Test>::do_create_issuer(payout_token.owner)
                .expect("Could not make payout_token.owner an issuer");
            identity::Module::<Test>::do_create_investor(token.owner)
                .expect("Could not make token.owner an investor");
            identity::Module::<Test>::do_create_investor(payout_token.owner)
                .expect("Could not make payout_token.owner an investor");

            // Share issuance is successful
            assert_ok!(Asset::issue_token(
                Origin::signed(token.owner),
                token.name.clone(),
                token.name.clone(),
                token.total_supply
            ));

            // Issuance for payout token is successful
            assert_ok!(Asset::issue_token(
                Origin::signed(payout_token.owner),
                payout_token.name.clone(),
                payout_token.name.clone(),
                payout_token.total_supply
            ));

            // Prepare a whitelisted investor
            let investor_id = 3;
            let amount_invested = 50_000;
            Balances::make_free_balance_be(&investor_id, 1_000_000);

            let now = Utc::now();
            <timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);

            // We need a lock to exist till assertions are done
            let outer = TOKEN_MAP_OUTER_LOCK.lock().unwrap();
            *TOKEN_MAP.lock().unwrap() = {
                let mut map = HashMap::new();
                map.insert(token.name.clone(), token.clone());
                map.insert(payout_token.name.clone(), payout_token.clone());
                map
            };

            // Add all whitelist entries for investor, token owner and payout_token owner
            assert_ok!(general_tm::Module::<Test>::add_to_whitelist(
                Origin::signed(token.owner),
                token.name.clone(),
                0,
                investor_id,
                (now - Duration::hours(1)).timestamp() as u64,
            ));
            assert_ok!(general_tm::Module::<Test>::add_to_whitelist(
                Origin::signed(token.owner),
                token.name.clone(),
                0,
                token.owner,
                (now - Duration::hours(1)).timestamp() as u64,
            ));
            assert_ok!(general_tm::Module::<Test>::add_to_whitelist(
                Origin::signed(payout_token.owner),
                payout_token.name.clone(),
                0,
                investor_id,
                (now - Duration::hours(1)).timestamp() as u64,
            ));
            assert_ok!(general_tm::Module::<Test>::add_to_whitelist(
                Origin::signed(payout_token.owner),
                payout_token.name.clone(),
                0,
                token.owner,
                (now - Duration::hours(1)).timestamp() as u64,
            ));
            assert_ok!(general_tm::Module::<Test>::add_to_whitelist(
                Origin::signed(payout_token.owner),
                payout_token.name.clone(),
                0,
                payout_token.owner,
                (now - Duration::hours(1)).timestamp() as u64,
            ));
            drop(outer);

            // Create checkpoint for token
            assert_ok!(Asset::create_checkpoint(
                Origin::signed(token.owner),
                token.name.clone()
            ));

            // Transfer tokens to investor
            assert_ok!(Asset::transfer(
                Origin::signed(token.owner),
                token.name.clone(),
                investor_id,
                amount_invested
            ));

            let checkpoint_id = 0;

            let dividend = Dividend {
                amount: 500_000,
                active: false,
                maturates_at: Some((now - Duration::hours(1)).timestamp() as u64),
                expires_at: Some((now + Duration::hours(1)).timestamp() as u64),
                payout_currency: Some(payout_token.name.clone()),
                checkpoint_id,
            };

            // Transfer payout tokens to token owner
            assert_ok!(Asset::transfer(
                Origin::signed(payout_token.owner),
                payout_token.name.clone(),
                token.owner,
                dividend.amount
            ));

            // Create the dividend for token
            assert_ok!(DividendModule::new(
                Origin::signed(token.owner),
                dividend.amount,
                token.name.clone(),
                dividend.maturates_at.clone(),
                dividend.expires_at.clone(),
                dividend.payout_currency.clone(),
                dividend.checkpoint_id
            ));

            // Compare created dividend with the expected structure
            assert_eq!(
                DividendModule::dividends((token.name.clone(), dividend.checkpoint_id))[0],
                dividend
            );

            // Start payout
            assert_ok!(DividendModule::activate(
                Origin::signed(token.owner),
                token.name.clone(),
                dividend.checkpoint_id,
                0
            ));

            // Claim investor's share
            assert_ok!(DividendModule::claim(
                Origin::signed(investor_id),
                token.name.clone(),
                dividend.checkpoint_id,
                0
            ));

            // Check if the correct amount was added to investor balance
            let share = dividend.amount * amount_invested / token.total_supply;
            assert_eq!(
                Asset::balance_of((payout_token.name.clone(), investor_id)),
                share
            );
        });
    }
}
