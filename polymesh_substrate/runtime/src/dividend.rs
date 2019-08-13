use crate::balances;
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

use crate::{asset, erc20, utils};

/// The module's configuration trait.
pub trait Trait:
    asset::Trait + balances::Trait + erc20::Trait + system::Trait + utils::Trait + timestamp::Trait
{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[derive(parity_codec::Encode, parity_codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct Dividend<U, V> {
    /// Total amount to be distributed
    amount: U,
    /// Amount left to distribute
    amount_left: U,
    /// Whether the owner has claimed remaining funds
    remaining_claimed: bool,
    /// Whether claiming dividends is enabled
    active: bool,
    /// Whether the dividend was cancelled
    canceled: bool,
    /// An optional timestamp of payout start
    matures_at: Option<V>,
    /// An optional timestamp for payout end
    expires_at: Option<V>,
    /// The payout ERC20 currency ticker. None means POLY
    payout_currency: Option<Vec<u8>>,
    /// The checkpoint
    checkpoint_id: u32,
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as dividend {
        // Dividend records; (ticker, dividend ID) => dividend entry
        // Note: contrary to checkpoint IDs, dividend IDs are 0-indexed.
        Dividends get(dividends): map (Vec<u8>, u32) => Dividend<T::TokenBalance, T::Moment>;

        // How many dividends were created for a ticker so far; (ticker) => count
        DividendCount get(dividend_count): map (Vec<u8>) => u32;

        // Payout flags, decide whether a user already was paid their dividend
        // (who, ticker, checkpoint_id, dividend_id)
        UserPayoutCompleted get(payout_completed): map (T::AccountId, Vec<u8>, u32) => bool;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event<T>() = default;

        /// Creates a new dividend entry without payout. Token must have at least one checkpoint.
        /// None in payout_currency means POLY payout.
        pub fn new(origin,
                   amount: T::TokenBalance,
                   ticker: Vec<u8>,
                   matures_at: T::Moment,
                   expires_at: T::Moment,
                   payout_ticker: Vec<u8>,
                   checkpoint_id: u32
                  ) -> Result {
            let sender = ensure_signed(origin)?;
            let ticker = utils::bytes_to_upper(ticker.as_slice());

            // Check that sender owns the asset token
            ensure!(<asset::Module<T>>::_is_owner(ticker.clone(), sender.clone()), "User is not the owner of the asset");

            // Check if sender has enough funds in payout currency
            let balance = if payout_ticker.is_empty() {
                // Check for POLY
                <T::TokenBalance as As<T::Balance>>::sa(<balances::FreeBalance<T>>::get(sender.clone()))
            } else {
                // Check for token
                <erc20::BalanceOf<T>>::get((payout_ticker.clone(), sender.clone()))
            };
            ensure!(balance >= amount, "Insufficient funds for payout");

            // Unpack the checkpoint ID, use the latest or create a new one, in that order
            let checkpoint_id = if checkpoint_id > 0 {
                checkpoint_id
            } else {
                let count = <asset::TotalCheckpoints<T>>::get(ticker.clone());
                if count > 0 {
                    count
                } else {
                    <asset::Module<T>>::_create_checkpoint(ticker.clone())?;
                    1 // Caution: relies on 1-indexing
                }
            };
            // Check if checkpoint exists
            ensure!(<asset::Module<T>>::total_checkpoints_of(ticker.clone()) >= checkpoint_id,
            "Checkpoint for dividend does not exist");

            let now = <timestamp::Module<T>>::get();
            let zero_ts = now.clone() - now.clone(); // A 0 timestamp

            // Check maturity/expiration dates
            match (&matures_at, &expires_at) {
                (_start, end) if  end == &zero_ts => {
                },
                (start, end) if start == &zero_ts => {
                    // Ends in the future
                    ensure!(end > &now, "Dividend payout must end in the future");
                },
                (start, end) if start == &zero_ts && end == &zero_ts => {}
                (start, end) => {
                    // Ends in the future
                    ensure!(end > &now, "Dividend payout should end in the future");
                    // Ends after start
                    ensure!(end > start, "Dividend payout must end after it starts");
                },
            }

            // Subtract the amount
            if payout_ticker.is_empty() {
                let _imbalance = <balances::Module<T> as Currency<_>>::withdraw(
                    &sender,
                    <T::TokenBalance as As<T::Balance>>::as_(amount),
                    WithdrawReason::Reserve,
                    ExistenceRequirement::KeepAlive)?;
            } else {
                let new_balance = balance.checked_sub(&amount).ok_or("Overflow calculating new owner balance")?;
                <erc20::BalanceOf<T>>::insert((payout_ticker.clone(), sender.clone()), new_balance);
            }

            // Insert dividend entry into storage
            let new_dividend = Dividend {
                amount,
                amount_left: amount,
                remaining_claimed: false,
                active: false,
                canceled: false,
                matures_at: if matures_at > zero_ts { Some(matures_at) } else { None },
                expires_at: if expires_at > zero_ts { Some(expires_at) } else { None },
                payout_currency: if payout_ticker.is_empty() { None } else { Some(payout_ticker.clone())},
                checkpoint_id,
            };

            let dividend_id = Self::add_dividend_entry(ticker.clone(), new_dividend)?;

            // Dispatch event
            Self::deposit_event(RawEvent::DividendCreated(ticker, amount, dividend_id));

            Ok(())
        }

        /// Lets the owner cancel a dividend before start date or activation
        pub fn cancel(origin, ticker: Vec<u8>, dividend_id: u32) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender owns the asset token
            ensure!(<asset::Module<T>>::_is_owner(ticker.clone(), sender.clone()), "User is not the owner of the asset");

            // Check that the dividend has not started yet or is not active
            let entry: Dividend<_, _> = Self::get_dividend(ticker.clone(), dividend_id).ok_or("Dividend not found")?;
            let now = <timestamp::Module<T>>::get();

            let starts_in_future = if let Some(start) = entry.matures_at.clone() {
                start > now
            } else {
                false
            };

            ensure!(starts_in_future || !entry.active, "Cancellable dividend must mature in the future or be inactive");

            // Flip `canceled`
            <Dividends<T>>::mutate((ticker.clone(), dividend_id), |entry| -> Result {
                entry.canceled = true;
                Ok(())
            })?;

            // Pay amount back to owner
            if let Some(payout_ticker) = entry.payout_currency.clone() {
                <erc20::BalanceOf<T>>::mutate((payout_ticker.clone(), sender.clone()), |balance: &mut T::TokenBalance| -> Result {
                    *balance  = balance
                        .checked_add(&entry.amount)
                        .ok_or("Could not add amount back to asset owner account")?;
                    Ok(())
                })?;
            } else {
                let _imbalance = <balances::Module<T> as Currency<_>>::deposit_into_existing(
                    &sender,
                    <T::TokenBalance as As<T::Balance>>::as_(entry.amount)
                    )?;
            }
            Ok(())
        }

        /// Enables withdrawal of dividend funds for asset `ticker`.
        pub fn activate(origin, ticker: Vec<u8>, dividend_id: u32) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender owns the asset token
            ensure!(<asset::Module<T>>::_is_owner(ticker.clone(), sender.clone()), "User is not the owner of the asset");

            // Check that the dividend exists
            ensure!(<Dividends<T>>::exists((ticker.clone(), dividend_id)), "No dividend entry for supplied ticker and ID");

            // Flip `active`
            <Dividends<T>>::mutate((ticker.clone(), dividend_id), |entry| -> Result {
                entry.active = true;
                Ok(())
            })?;

            // Dispatch event
            Self::deposit_event(RawEvent::DividendActivated(ticker.clone(), dividend_id));

            Ok(())
        }

        /// Withdraws from a dividend the adequate share of the `amount` field. All dividend shares
        /// are rounded by truncation (down to first integer below)
        pub fn claim(origin, ticker: Vec<u8>, dividend_id: u32) -> Result {
            let sender = ensure_signed(origin)?;

            // Check if sender wasn't already paid their share
            ensure!(!<UserPayoutCompleted<T>>::get((sender.clone(), ticker.clone(), dividend_id)), "User was already paid their share");

            // Look dividend entry up
            let dividend = Self::get_dividend(ticker.clone(), dividend_id).ok_or("Dividend not found")?;

            let balance_at_checkpoint =
                <asset::Module<T>>::get_balance_at(ticker.clone(), sender.clone(), dividend.checkpoint_id);

            // Check if the owner hadn't yanked the remaining amount out
            ensure!(!dividend.remaining_claimed, "The remaining payout funds were already claimed");

            // Check if the dividend is active
            ensure!(dividend.active, "Dividend not active");

            // Check if the dividend was not canceled
            ensure!(!dividend.canceled, "Dividend was canceled");

            let now = <timestamp::Module<T>>::get();

            // Check if the current time is within maturity/expiration bounds
            if let Some(start) = dividend.matures_at.as_ref() {
                ensure!(now > *start, "Attempted payout before maturity");
            }

            if let Some(end) = dividend.expires_at.as_ref() {
                ensure!(*end > now, "Attempted payout after expiration");
            }

            // Compute the share
            ensure!(<asset::Tokens<T>>::exists(ticker.clone()), "Dividend token entry not found");
            let supply_at_checkpoint = <asset::CheckpointTotalSupply<T>>::get((ticker.clone(), dividend.checkpoint_id));

            let balance_amount_product = balance_at_checkpoint
                .checked_mul(&dividend.amount)
                .ok_or("multiplying balance and total payout amount failed")?;

            let share = balance_amount_product
                .checked_div(&supply_at_checkpoint)
                .ok_or("balance_amount_product division failed")?;

            // Adjust the paid_out amount
            <Dividends<T>>::mutate((ticker.clone(), dividend_id), |entry| -> Result {
                entry.amount_left = entry.amount_left.checked_sub(&share).ok_or("Could not increase paid_out")?;
                Ok(())
            })?;

            // Perform the payout in designated tokens or base currency depending on setting
            if let Some(payout_ticker) = dividend.payout_currency.as_ref() {
                <erc20::BalanceOf<T>>::mutate(
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
            <UserPayoutCompleted<T>>::insert((sender.clone(), ticker.clone(), dividend_id), true);

            // Dispatch event
            Self::deposit_event(RawEvent::DividendPaidOutToUser(sender.clone(), ticker.clone(), dividend_id, share));
            Ok(())
        }

        /// After a dividend had expired, collect the remaining amount to owner address
        pub fn claim_unclaimed(origin, ticker: Vec<u8>, dividend_id: u32) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender owns the asset token
            ensure!(<asset::Module<T>>::_is_owner(ticker.clone(), sender.clone()), "User is not the owner of the asset");

            let entry = Self::get_dividend(ticker.clone(), dividend_id).ok_or("Could not retrieve dividend")?;

            // Check that the expiry date had passed
            let now = <timestamp::Module<T>>::get();

            if let Some(end) = entry.expires_at.clone() {
                ensure!(end < now, "Dividend not finished for returning unclaimed payout");
            } else {
                return Err("Claiming unclaimed payouts requires an end date");
            }

            // Transfer the computed amount
            if let Some(payout_ticker) = entry.payout_currency.clone() {
                <erc20::BalanceOf<T>>::mutate((payout_ticker.clone(), sender.clone()), |balance: &mut T::TokenBalance| -> Result {
                    *balance  = balance
                        .checked_add(&entry.amount_left)
                        .ok_or("Could not add amount back to asset owner account")?;
                    Ok(())
                })?;
            } else {
                let _imbalance = <balances::Module<T> as Currency<_>>::deposit_into_existing(
                    &sender,
                    <T::TokenBalance as As<T::Balance>>::as_(entry.amount_left)
                    )?;
            }

            // Set paid_out to maximum value, flip remaining_claimed
            <Dividends<T>>::mutate((ticker.clone(), dividend_id), |entry| -> Result {
                entry.amount_left = <<T as utils::Trait>::TokenBalance as As<u64>>::sa(0);
                entry.remaining_claimed = true;
                Ok(())
            })?;

            Self::deposit_event(RawEvent::DividendRemainingClaimed(ticker.clone(), dividend_id, entry.amount_left));

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
        // ticker, amount, dividend ID
        DividendCreated(Vec<u8>, TokenBalance, u32),

        // ticker, dividend ID
        DividendActivated(Vec<u8>, u32),

        // who, ticker, dividend ID, share
        DividendPaidOutToUser(AccountId, Vec<u8>, u32, TokenBalance),

        // ticker, dividend ID, amount
        DividendRemainingClaimed(Vec<u8>, u32, TokenBalance),
    }
);

impl<T: Trait> Module<T> {
    /// A helper method for dividend creation. Returns dividend ID
    /// #[inline]
    fn add_dividend_entry(
        ticker: Vec<u8>,
        d: Dividend<T::TokenBalance, T::Moment>,
    ) -> core::result::Result<u32, &'static str> {
        let old_count = <DividendCount<T>>::get(ticker.clone());
        let new_count = old_count
            .checked_add(1)
            .ok_or("Could not add 1 to dividend count")?;

        <Dividends<T>>::insert((ticker.clone(), old_count), d);
        <DividendCount<T>>::insert(ticker.clone(), new_count);

        Ok(old_count)
    }

    /// Retrieves a dividend checking that it exists beforehand.
    pub fn get_dividend(
        ticker: Vec<u8>,
        dividend_id: u32,
    ) -> Option<Dividend<T::TokenBalance, T::Moment>> {
        // Check that the dividend entry exists
        if <Dividends<T>>::exists((ticker.clone(), dividend_id)) {
            Some(<Dividends<T>>::get((ticker.clone(), dividend_id)))
        } else {
            None
        }
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

    use crate::{
        asset::SecurityToken, erc20::ERC20Token, exemption, general_tm, identity, percentage_tm,
        registry,
    };

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

    impl erc20::Trait for Test {
        type Event = ();
        type Currency = balances::Module<Test>;
    }

    impl exemption::Trait for Test {
        type Event = ();
        type Asset = Module<Test>;
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
        fn as_u128(v: Self::TokenBalance) -> u128 {
            v
        }
        fn as_tb(v: u128) -> Self::TokenBalance {
            v
        }
    }

    impl registry::Trait for Test {}

    impl Trait for Test {
        type Event = ();
    }
    impl asset::AssetTrait<<Test as system::Trait>::AccountId, <Test as utils::Trait>::TokenBalance>
        for Module<Test>
    {
        fn is_owner(_ticker: Vec<u8>, sender: <Test as system::Trait>::AccountId) -> bool {
            if let Some(token) = TOKEN_MAP.lock().unwrap().get(&_ticker) {
                token.owner == sender
            } else {
                false
            }
        }

        fn _mint_from_sto(
            _ticker: Vec<u8>,
            _sender: <Test as system::Trait>::AccountId,
            _tokens_purchased: <Test as utils::Trait>::TokenBalance,
        ) -> Result {
            unimplemented!();
        }

        /// Get the asset `id` balance of `who`.
        fn balance(
            _ticker: Vec<u8>,
            _who: <Test as system::Trait>::AccountId,
        ) -> <Test as utils::Trait>::TokenBalance {
            unimplemented!();
        }

        // Get the total supply of an asset `id`
        fn total_supply(_ticker: Vec<u8>) -> <Test as utils::Trait>::TokenBalance {
            unimplemented!();
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
    type ERC20 = erc20::Module<Test>;

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
                granularity: 1,
                decimals: 18,
            };

            // A token used for payout
            let payout_token = ERC20Token {
                ticker: vec![0x02],
                owner: 2,
                total_supply: 200_000_000,
            };

            // Raise the owners' base currency balance
            Balances::make_free_balance_be(&token.owner, 1_000_000);
            Balances::make_free_balance_be(&payout_token.owner, 1_000_000);

            identity::Module::<Test>::do_create_issuer(token.owner)
                .expect("Could not make token.owner an issuer");
            identity::Module::<Test>::do_create_erc20_issuer(payout_token.owner)
                .expect("Could not make payout_token.owner an issuer");
            identity::Module::<Test>::do_create_investor(token.owner)
                .expect("Could not make token.owner an investor");

            // Share issuance is successful
            assert_ok!(Asset::issue_token(
                Origin::signed(token.owner),
                token.name.clone(),
                token.name.clone(),
                token.total_supply,
                true
            ));

            // Issuance for payout token is successful
            assert_ok!(ERC20::create_token(
                Origin::signed(payout_token.owner),
                payout_token.ticker.clone(),
                payout_token.total_supply
            ));

            // Prepare a whitelisted investor
            let investor_id = 3;
            identity::Module::<Test>::do_create_investor(investor_id)
                .expect("Could not make token.owner an investor");
            let amount_invested = 50_000;
            Balances::make_free_balance_be(&investor_id, 1_000_000);

            let now = Utc::now();
            <timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);

            // We need a lock to exist till assertions are done
            let outer = TOKEN_MAP_OUTER_LOCK.lock().unwrap();
            *TOKEN_MAP.lock().unwrap() = {
                let mut map = HashMap::new();
                map.insert(token.name.clone(), token.clone());
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
            drop(outer);

            // Transfer tokens to investor
            assert_ok!(Asset::transfer(
                Origin::signed(token.owner),
                token.name.clone(),
                investor_id,
                amount_invested
            ));

            // Create checkpoint for token
            assert_ok!(Asset::create_checkpoint(
                Origin::signed(token.owner),
                token.name.clone()
            ));

            // Checkpoints are 1-indexed
            let checkpoint_id = 1;

            let dividend = Dividend {
                amount: 500_000,
                amount_left: 500_000,
                remaining_claimed: false,
                active: false,
                canceled: false,
                matures_at: Some((now - Duration::hours(1)).timestamp() as u64),
                expires_at: Some((now + Duration::hours(1)).timestamp() as u64),
                payout_currency: Some(payout_token.ticker.clone()),
                checkpoint_id,
            };

            // Transfer payout tokens to asset owner
            assert_ok!(ERC20::transfer(
                Origin::signed(payout_token.owner),
                payout_token.ticker.clone(),
                token.owner,
                dividend.amount
            ));

            // Create the dividend for asset
            assert_ok!(DividendModule::new(
                Origin::signed(token.owner),
                dividend.amount,
                token.name.clone(),
                dividend.matures_at.clone().unwrap(),
                dividend.expires_at.clone().unwrap(),
                dividend.payout_currency.clone().unwrap(),
                dividend.checkpoint_id
            ));

            // Compare created dividend with the expected structure
            assert_eq!(
                DividendModule::get_dividend(token.name.clone(), 0),
                Some(dividend.clone())
            );

            // Start payout
            assert_ok!(DividendModule::activate(
                Origin::signed(token.owner),
                token.name.clone(),
                0
            ));

            // Claim investor's share
            assert_ok!(DividendModule::claim(
                Origin::signed(investor_id),
                token.name.clone(),
                0,
            ));

            // Check if the correct amount was added to investor balance
            let share = dividend.amount * amount_invested / token.total_supply;
            assert_eq!(
                ERC20::balance_of((payout_token.ticker.clone(), investor_id)),
                share
            );

            // Check if amount_left was adjusted correctly
            let current_entry = DividendModule::get_dividend(token.name.clone(), 0)
                .expect("Could not retrieve dividend");
            assert_eq!(current_entry.amount_left, current_entry.amount - share);
        });
    }
}
