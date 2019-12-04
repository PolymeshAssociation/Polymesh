//! # Dividend Module
//!
//! The Dividend module provides functionality for distributing dividends to tokenholders.
//!
//! ## Overview
//!
//! The Balances module provides functions for:
//!
//! - Paying dividends
//! - Termination existing dividends
//! - claiming dividends
//! - Claiming back unclaimed dividends
//!
//! ### Terminology
//!
//! - **Payout Currency:** It is the ticker of the currency in which dividends are to be paid.
//! - **Dividend maturity date:** It is the date after which dividends can be claimed by tokenholders
//! - **Dividend expiry date:** Tokenholders can claim dividends before this date.
//! After this date, issuer can reclaim the remaining dividend.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `new` - Creates a new dividend
//! - `cancel` - Cancels an existing dividend
//! - `claim` - Allows tokenholders to claim/collect their fair share of the dividend
//! - `claim_unclaimed` - Allows token issuer to claim unclaimed dividend
//!
//! ### Public Functions
//!
//! - `get_dividend` - Returns details about a dividend

use crate::{asset, balances, identity, simple_token, utils};
use codec::Encode;
use primitives::{IdentityId, Key};
use rstd::{convert::TryFrom, prelude::*};
use sr_primitives::traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};
use srml_support::{decl_event, decl_module, decl_storage, dispatch::Result, ensure};
use system::ensure_signed;

/// The module's configuration trait.
pub trait Trait:
    asset::Trait
    + balances::Trait
    + simple_token::Trait
    + system::Trait
    + utils::Trait
    + timestamp::Trait
{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

/// Details about the dividend
#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct Dividend<U, V> {
    /// Total amount to be distributed
    amount: U,
    /// Amount left to distribute
    amount_left: U,
    /// Whether the owner has claimed remaining funds
    remaining_claimed: bool,
    /// An optional timestamp of payout start
    matures_at: Option<V>,
    /// An optional timestamp for payout end
    expires_at: Option<V>,
    /// The payout SimpleToken currency ticker.
    payout_currency: Vec<u8>,
    /// The checkpoint
    checkpoint_id: u64,
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as dividend {
        /// Dividend records; (ticker, dividend ID) => dividend entry
        /// Note: contrary to checkpoint IDs, dividend IDs are 0-indexed.
        Dividends get(dividends): map (Vec<u8>, u32) => Dividend<T::Balance, T::Moment>;

        /// How many dividends were created for a ticker so far; (ticker) => count
        DividendCount get(dividend_count): map (Vec<u8>) => u32;

        /// Payout flags, decide whether a user already was paid their dividend
        /// (DID, ticker, dividend_id) -> whether they got their payout
        UserPayoutCompleted get(payout_completed): map (IdentityId, Vec<u8>, u32) => bool;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        fn deposit_event() = default;

        /// Creates a new dividend entry without payout. Token must have at least one checkpoint.
        pub fn new(origin,
            did: IdentityId,
            amount: T::Balance,
            ticker: Vec<u8>,
            matures_at: T::Moment,
            expires_at: T::Moment,
            payout_ticker: Vec<u8>,
            checkpoint_id: u64
        ) -> Result {
            let sender = ensure_signed(origin)?;
            let ticker = utils::bytes_to_upper(ticker.as_slice());

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

            // Check that sender owns the asset token
            ensure!(<asset::Module<T>>::_is_owner(&ticker, did), "User is not the owner of the asset");

            // Check if sender has enough funds in payout currency
            let balance = <simple_token::BalanceOf<T>>::get((payout_ticker.clone(), did));
            ensure!(balance >= amount, "Insufficient funds for payout");

            // Unpack the checkpoint ID, use the latest or create a new one, in that order
            let checkpoint_id = if checkpoint_id > 0 {
                checkpoint_id
            } else {
                let count = <asset::TotalCheckpoints>::get(&ticker);
                if count > 0 {
                    count
                } else {
                    <asset::Module<T>>::_create_checkpoint(&ticker)?;
                    <asset::TotalCheckpoints>::get(&ticker)
                }
            };
            // Check if checkpoint exists
            ensure!(<asset::Module<T>>::total_checkpoints_of(&ticker) >= checkpoint_id,
            "Checkpoint for dividend does not exist");

            let now = <timestamp::Module<T>>::get();
            let zero_ts = now - now; // A 0 timestamp

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
            let new_balance = balance.checked_sub(&amount).ok_or("Underflow calculating new owner balance")?;
            <simple_token::BalanceOf<T>>::insert((payout_ticker.clone(), did), new_balance);

            // Insert dividend entry into storage
            let new_dividend = Dividend {
                amount,
                amount_left: amount,
                remaining_claimed: false,
                matures_at: if matures_at > zero_ts { Some(matures_at) } else { None },
                expires_at: if expires_at > zero_ts { Some(expires_at) } else { None },
                payout_currency: payout_ticker,
                checkpoint_id,
            };

            let dividend_id = Self::add_dividend_entry(&ticker, new_dividend)?;

            // Dispatch event
            Self::deposit_event(RawEvent::DividendCreated(ticker, amount, dividend_id));

            Ok(())
        }

        /// Lets the owner cancel a dividend before start/maturity date
        pub fn cancel(origin, did: IdentityId, ticker: Vec<u8>, dividend_id: u32) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

            // Check that sender owns the asset token
            ensure!(<asset::Module<T>>::_is_owner(&ticker, did), "User is not the owner of the asset");

            // Check that the dividend has not started yet
            let entry: Dividend<_, _> = Self::get_dividend(&ticker, dividend_id).ok_or("Dividend not found")?;
            let now = <timestamp::Module<T>>::get();

            let starts_in_future = if let Some(ref start) = entry.matures_at {
                (*start) > now
            } else {
                false
            };

            ensure!(starts_in_future, "Cancellable dividend must mature in the future");

            // Pay amount back to owner
            <simple_token::BalanceOf<T>>::mutate((entry.payout_currency.clone(), did), |balance: &mut T::Balance| -> Result {
                *balance  = balance
                    .checked_add(&entry.amount)
                    .ok_or("Could not add amount back to asset owner account")?;
                Ok(())
            })?;

            <Dividends<T>>::remove((ticker.clone(), dividend_id));

            Self::deposit_event(RawEvent::DividendCanceled(ticker, dividend_id));

            Ok(())
        }

        /// Withdraws from a dividend the adequate share of the `amount` field. All dividend shares
        /// are rounded by truncation (down to first integer below)
        pub fn claim(origin, did: IdentityId, ticker: Vec<u8>, dividend_id: u32) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

            // Check if sender wasn't already paid their share
            ensure!(!<UserPayoutCompleted>::get((did, ticker.clone(), dividend_id)), "User was already paid their share");

            // Look dividend entry up
            let dividend = Self::get_dividend(&ticker, dividend_id).ok_or("Dividend not found")?;

            let balance_at_checkpoint =
                <asset::Module<T>>::get_balance_at(&ticker, did, dividend.checkpoint_id);

            // Check if the owner hadn't yanked the remaining amount out
            ensure!(!dividend.remaining_claimed, "The remaining payout funds were already claimed");

            let now = <timestamp::Module<T>>::get();

            // Check if the current time is within maturity/expiration bounds
            if let Some(start) = dividend.matures_at.as_ref() {
                ensure!(now > *start, "Attempted payout before maturity");
            }

            if let Some(end) = dividend.expires_at.as_ref() {
                ensure!(*end > now, "Attempted payout after expiration");
            }

            // Compute the share
            ensure!(<asset::Tokens<T>>::exists(&ticker), "Dividend token entry not found");
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

            // Perform the payout in designated tokens
            <simple_token::BalanceOf<T>>::mutate(
                (dividend.payout_currency.clone(), did),
                |balance| -> Result {
                    *balance = balance
                        .checked_add(&share)
                        .ok_or("Could not add share to sender balance")?;
                    Ok(())
                }
            )?;

            // Create payout entry
            <UserPayoutCompleted>::insert((did, ticker.clone(), dividend_id), true);

            // Dispatch event
            Self::deposit_event(RawEvent::DividendPaidOutToUser(did, ticker, dividend_id, share));
            Ok(())
        }

        /// After a dividend had expired, collect the remaining amount to owner address
        pub fn claim_unclaimed(origin, did: IdentityId, ticker: Vec<u8>, dividend_id: u32) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

            // Check that sender owns the asset token
            ensure!(<asset::Module<T>>::_is_owner(&ticker, did), "User is not the owner of the asset");

            let entry = Self::get_dividend(&ticker, dividend_id).ok_or("Could not retrieve dividend")?;

            // Check that the expiry date had passed
            let now = <timestamp::Module<T>>::get();

            if let Some(ref end) = entry.expires_at {
                ensure!(*end < now, "Dividend not finished for returning unclaimed payout");
            } else {
                return Err("Claiming unclaimed payouts requires an end date");
            }

            // Transfer the computed amount
            <simple_token::BalanceOf<T>>::mutate((entry.payout_currency.clone(), did), |balance: &mut T::Balance| -> Result {
                let new_balance = balance.checked_add(&entry.amount_left).ok_or("Could not add amount back to asset owner DID")?;
                *balance  = new_balance;
                Ok(())
            })?;

            // Set amount_left, flip remaining_claimed
            <Dividends<T>>::mutate((ticker.clone(), dividend_id), |entry| -> Result {
                entry.amount_left = 0.into();
                entry.remaining_claimed = true;
                Ok(())
            })?;

            Self::deposit_event(RawEvent::DividendRemainingClaimed(ticker, dividend_id, entry.amount_left));

            Ok(())
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        Balance = <T as balances::Trait>::Balance,
    {
        /// A new dividend was created (ticker, amount, dividend ID)
        DividendCreated(Vec<u8>, Balance, u32),

        /// A dividend was canceled (ticker, dividend ID)
        DividendCanceled(Vec<u8>, u32),

        /// Dividend was paid to a user (who, ticker, dividend ID, share)
        DividendPaidOutToUser(IdentityId, Vec<u8>, u32, Balance),

        /// Unclaimed dividend was claimed back (ticker, dividend ID, amount)
        DividendRemainingClaimed(Vec<u8>, u32, Balance),
    }
);

impl<T: Trait> Module<T> {
    /// A helper method for dividend creation. Returns dividend ID
    /// #[inline]
    fn add_dividend_entry(
        ticker: &Vec<u8>,
        d: Dividend<T::Balance, T::Moment>,
    ) -> core::result::Result<u32, &'static str> {
        let old_count = <DividendCount>::get(ticker);
        let new_count = old_count
            .checked_add(1)
            .ok_or("Could not add 1 to dividend count")?;

        <Dividends<T>>::insert((ticker.clone(), old_count), d);
        <DividendCount>::insert(ticker, new_count);

        Ok(old_count)
    }

    /// Retrieves a dividend checking that it exists beforehand.
    pub fn get_dividend(
        ticker: &[u8],
        dividend_id: u32,
    ) -> Option<Dividend<T::Balance, T::Moment>> {
        // Check that the dividend entry exists
        let ticker_div_id = (ticker.to_vec(), dividend_id);
        if <Dividends<T>>::exists(&ticker_div_id) {
            Some(<Dividends<T>>::get(&ticker_div_id))
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
    use core::result::Result as StdResult;
    use lazy_static::lazy_static;
    use sr_io::with_externalities;
    use sr_primitives::traits::Verify;
    use sr_primitives::{
        testing::{Header, UintAuthorityId},
        traits::{BlakeTwo256, ConvertInto, IdentityLookup, OpaqueKeys},
        AnySignature, Perbill,
    };
    use srml_support::traits::Currency;
    use srml_support::{
        assert_ok,
        dispatch::{DispatchError, DispatchResult},
        impl_outer_origin, parameter_types,
    };
    use substrate_primitives::{Blake2Hasher, H256};
    use test_client::{self, AccountKeyring};

    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use crate::{
        asset::SecurityToken, balances, exemption, general_tm, identity, percentage_tm, registry,
        simple_token::SimpleTokenRecord,
    };

    type SessionIndex = u32;
    type AuthorityId = <AnySignature as Verify>::Signer;
    type BlockNumber = u64;
    type OffChainSignature = AnySignature;
    type AccountId = <AnySignature as Verify>::Signer;

    pub struct TestOnSessionEnding;
    impl session::OnSessionEnding<AuthorityId> for TestOnSessionEnding {
        fn on_session_ending(_: SessionIndex, _: SessionIndex) -> Option<Vec<AuthorityId>> {
            None
        }
    }

    pub struct TestSessionHandler;
    impl session::SessionHandler<AuthorityId> for TestSessionHandler {
        fn on_new_session<Ks: OpaqueKeys>(
            _changed: bool,
            _validators: &[(AuthorityId, Ks)],
            _queued_validators: &[(AuthorityId, Ks)],
        ) {
        }

        fn on_disabled(_validator_index: usize) {}

        fn on_genesis_session<Ks: OpaqueKeys>(_validators: &[(AuthorityId, Ks)]) {}
    }

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
        pub const BlockHashCount: u32 = 250;
        pub const MaximumBlockWeight: u32 = 4 * 1024 * 1024;
        pub const MaximumBlockLength: u32 = 4 * 1024 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }
    impl system::Trait for Test {
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = AccountId;
        type Lookup = IdentityLookup<AccountId>;
        type WeightMultiplierUpdate = ();
        type Header = Header;
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type AvailableBlockRatio = AvailableBlockRatio;
        type MaximumBlockLength = MaximumBlockLength;
        type Version = ();
    }

    parameter_types! {
        pub const Period: BlockNumber = 1;
        pub const Offset: BlockNumber = 0;
        pub const ExistentialDeposit: u64 = 0;
        pub const TransferFee: u64 = 0;
        pub const CreationFee: u64 = 0;
        pub const TransactionBaseFee: u64 = 0;
        pub const TransactionByteFee: u64 = 0;
    }

    parameter_types! {
        pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(33);
    }

    impl session::Trait for Test {
        type OnSessionEnding = TestOnSessionEnding;
        type Keys = UintAuthorityId;
        type ShouldEndSession = session::PeriodicSessions<Period, Offset>;
        type SessionHandler = TestSessionHandler;
        type Event = ();
        type ValidatorId = AuthorityId;
        type ValidatorIdOf = ConvertInto;
        type SelectInitialValidators = ();
        type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
    }

    impl session::historical::Trait for Test {
        type FullIdentification = ();
        type FullIdentificationOf = ();
    }

    impl balances::Trait for Test {
        type Balance = u128;
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();
        type TransactionPayment = ();
        type DustRemoval = ();
        type TransferPayment = ();
        type ExistentialDeposit = ExistentialDeposit;
        type TransferFee = TransferFee;
        type CreationFee = CreationFee;
        type TransactionBaseFee = TransactionBaseFee;
        type TransactionByteFee = TransactionByteFee;
        type WeightToFee = ConvertInto;
        type Identity = identity::Module<Test>;
    }

    impl simple_token::Trait for Test {
        type Event = ();
    }

    impl asset::Trait for Test {
        type Event = ();
        type Currency = balances::Module<Test>;
    }

    #[derive(codec::Encode, codec::Decode, Debug, Clone, Eq, PartialEq)]
    pub struct IdentityProposal {
        pub dummy: u8,
    }

    impl sr_primitives::traits::Dispatchable for IdentityProposal {
        type Origin = Origin;
        type Trait = Test;
        type Error = DispatchError;

        fn dispatch(self, _origin: Self::Origin) -> DispatchResult<Self::Error> {
            Ok(())
        }
    }

    impl identity::Trait for Test {
        type Event = ();
        type Proposal = IdentityProposal;
    }

    impl exemption::Trait for Test {
        type Event = ();
        type Asset = Module<Test>;
    }

    impl general_tm::Trait for Test {
        type Event = ();
        type Asset = Module<Test>;
    }

    impl percentage_tm::Trait for Test {
        type Event = ();
    }

    parameter_types! {
        pub const MinimumPeriod: u64 = 3;
    }

    impl timestamp::Trait for Test {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = MinimumPeriod;
    }

    impl utils::Trait for Test {
        type OffChainSignature = OffChainSignature;
        fn validator_id_to_account_id(v: <Self as session::Trait>::ValidatorId) -> Self::AccountId {
            v
        }
    }

    impl registry::Trait for Test {}

    impl Trait for Test {
        type Event = ();
    }
    impl asset::AssetTrait<<Test as balances::Trait>::Balance> for Module<Test> {
        fn is_owner(ticker: &Vec<u8>, sender_did: IdentityId) -> bool {
            if let Some(token) = TOKEN_MAP.lock().unwrap().get(ticker) {
                token.owner_did == sender_did
            } else {
                false
            }
        }

        fn _mint_from_sto(
            _ticker: &[u8],
            _sender_did: IdentityId,
            _tokens_purchased: <Test as balances::Trait>::Balance,
        ) -> Result {
            unimplemented!();
        }

        /// Get the asset `id` balance of `who`.
        fn balance(_ticker: &[u8], _did: IdentityId) -> <Test as balances::Trait>::Balance {
            unimplemented!();
        }

        // Get the total supply of an asset `id`
        fn total_supply(_ticker: &[u8]) -> <Test as balances::Trait>::Balance {
            unimplemented!();
        }

        fn get_balance_at(
            _ticker: &Vec<u8>,
            _did: IdentityId,
            _at: u64,
        ) -> <Test as balances::Trait>::Balance {
            unimplemented!();
        }
    }

    lazy_static! {
        static ref TOKEN_MAP: Arc<
            Mutex<
            HashMap<
            Vec<u8>,
            SecurityToken<
                <Test as balances::Trait>::Balance,
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
    type GeneralTM = general_tm::Module<Test>;
    type SimpleToken = simple_token::Module<Test>;
    type Identity = identity::Module<Test>;

    /// Build a genesis identity instance owned by the specified account
    fn identity_owned_by_1() -> sr_io::TestExternalities<Blake2Hasher> {
        let mut t = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        identity::GenesisConfig::<Test> {
            owner: AccountKeyring::Alice.public().into(),
            did_creation_fee: 250,
        }
        .assimilate_storage(&mut t)
        .unwrap();
        sr_io::TestExternalities::new(t)
    }

    fn make_account(
        account_id: &AccountId,
    ) -> StdResult<(<Test as system::Trait>::Origin, IdentityId), &'static str> {
        let signed_id = Origin::signed(account_id.clone());
        Balances::make_free_balance_be(&account_id, 1_000_000);
        Identity::register_did(signed_id.clone(), vec![])?;
        let did = Identity::get_identity(&Key::try_from(account_id.encode())?).unwrap();
        Ok((signed_id, did))
    }

    #[test]
    fn correct_dividend_must_work() {
        with_externalities(&mut identity_owned_by_1(), || {
            let token_owner_acc = AccountId::from(AccountKeyring::Alice);
            let (token_owner_signed, token_owner_did) = make_account(&token_owner_acc).unwrap();

            let payout_owner_acc = AccountId::from(AccountKeyring::Bob);
            let (payout_owner_signed, payout_owner_did) = make_account(&payout_owner_acc).unwrap();

            // A token representing 1M shares
            let token = SecurityToken {
                name: vec![0x01],
                owner_did: token_owner_did,
                total_supply: 1_000_000,
                divisible: true,
            };

            // A token used for payout
            let payout_token = SimpleTokenRecord {
                ticker: vec![0x02],
                owner_did: payout_owner_did,
                total_supply: 200_000_000,
            };

            Balances::make_free_balance_be(&token_owner_acc, 1_000_000);

            Balances::make_free_balance_be(&payout_owner_acc, 1_000_000);

            // Share issuance is successful
            assert_ok!(Asset::create_token(
                token_owner_signed.clone(),
                token_owner_did,
                token.name.clone(),
                token.name.clone(),
                token.total_supply,
                true
            ));

            // Issuance for payout token is successful
            assert_ok!(SimpleToken::create_token(
                payout_owner_signed.clone(),
                payout_owner_did,
                payout_token.ticker.clone(),
                payout_token.total_supply
            ));

            // Prepare a whitelisted investor
            let investor_acc = AccountId::from(AccountKeyring::Charlie);
            let (investor_signed, investor_did) = make_account(&investor_acc).unwrap();
            Balances::make_free_balance_be(&investor_acc, 1_000_000);

            let amount_invested = 50_000;

            let now = Utc::now();
            <timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);

            // We need a lock to exist till assertions are done
            let outer = TOKEN_MAP_OUTER_LOCK.lock().unwrap();
            *TOKEN_MAP.lock().unwrap() = {
                let mut map = HashMap::new();
                map.insert(token.name.clone(), token.clone());
                map
            };

            drop(outer);

            let asset_rule = general_tm::AssetRule {
                sender_rules: vec![],
                receiver_rules: vec![],
            };

            // Allow all transfers
            assert_ok!(GeneralTM::add_active_rule(
                token_owner_signed.clone(),
                token_owner_did,
                token.name.clone(),
                asset_rule
            ));

            // Transfer tokens to investor
            assert_ok!(Asset::transfer(
                token_owner_signed.clone(),
                token_owner_did,
                token.name.clone(),
                investor_did,
                amount_invested
            ));

            // Create checkpoint for token
            assert_ok!(Asset::create_checkpoint(
                token_owner_signed.clone(),
                token_owner_did,
                token.name.clone()
            ));

            // Checkpoints are 1-indexed
            let checkpoint_id = 1;

            let dividend = Dividend {
                amount: 500_000,
                amount_left: 500_000,
                remaining_claimed: false,
                matures_at: Some((now - Duration::hours(1)).timestamp() as u64),
                expires_at: Some((now + Duration::hours(1)).timestamp() as u64),
                payout_currency: payout_token.ticker.clone(),
                checkpoint_id,
            };

            // Transfer payout tokens to asset owner
            assert_ok!(SimpleToken::transfer(
                payout_owner_signed.clone(),
                payout_owner_did,
                payout_token.ticker.clone(),
                token_owner_did,
                dividend.amount
            ));

            // Create the dividend for asset
            assert_ok!(DividendModule::new(
                token_owner_signed.clone(),
                token_owner_did,
                dividend.amount,
                token.name.clone(),
                dividend.matures_at.clone().unwrap(),
                dividend.expires_at.clone().unwrap(),
                dividend.payout_currency.clone(),
                dividend.checkpoint_id
            ));

            // Compare created dividend with the expected structure
            assert_eq!(
                DividendModule::get_dividend(&token.name, 0),
                Some(dividend.clone())
            );

            // Claim investor's share
            assert_ok!(DividendModule::claim(
                investor_signed.clone(),
                investor_did,
                token.name.clone(),
                0,
            ));

            // Check if the correct amount was added to investor balance
            let share = dividend.amount * amount_invested / token.total_supply;
            assert_eq!(
                SimpleToken::balance_of((payout_token.ticker.clone(), investor_did)),
                share
            );

            // Check if amount_left was adjusted correctly
            let current_entry =
                DividendModule::get_dividend(&token.name, 0).expect("Could not retrieve dividend");
            assert_eq!(current_entry.amount_left, current_entry.amount - share);
        });
    }
}
