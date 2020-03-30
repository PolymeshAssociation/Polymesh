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

use crate::{asset, simple_token};

use polymesh_primitives::{AccountKey, IdentityId, Signatory, Ticker};
use polymesh_runtime_common::{
    balances::Trait as BalancesTrait,
    identity::Trait as IdentityTrait,
    protocol_fee::{ChargeProtocolFee, ProtocolOp},
    CommonTrait, Context,
};
use polymesh_runtime_identity as identity;

use codec::Encode;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use frame_system::{self as system, ensure_signed};
use sp_runtime::traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Zero};
use sp_std::{convert::TryFrom, prelude::*};

/// The module's configuration trait.
pub trait Trait:
    asset::Trait + BalancesTrait + simple_token::Trait + frame_system::Trait + pallet_timestamp::Trait
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
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
    payout_currency: Ticker,
    /// The checkpoint
    checkpoint_id: u64,
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as dividend {
        /// Dividend records; (ticker, dividend ID) => dividend entry
        /// Note: contrary to checkpoint IDs, dividend IDs are 0-indexed.
        Dividends get(fn dividends): map (Ticker, u32) => Dividend<T::Balance, T::Moment>;
        /// How many dividends were created for a ticker so far; (ticker) => count
        DividendCount get(fn dividend_count): map Ticker => u32;
        /// Payout flags, decide whether a user already was paid their dividend
        /// (DID, ticker, dividend_id) -> whether they got their payout
        UserPayoutCompleted get(fn payout_completed): map (IdentityId, Ticker, u32) => bool;
    }
}

type Identity<T> = identity::Module<T>;

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        type Error = Error<T>;

        // Initializing events
        fn deposit_event() = default;

        /// Creates a new dividend entry without payout. Token must have at least one checkpoint.
        pub fn new(origin,
            amount: T::Balance,
            ticker: Ticker,
            matures_at: T::Moment,
            expires_at: T::Moment,
            payout_ticker: Ticker,
            checkpoint_id: u64
        ) -> DispatchResult {
            let sender_key = AccountKey::try_from( ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            let sender = Signatory::AccountKey(sender_key);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &sender),
                Error::<T>::SenderMustBeSigningKeyForDid
            );
            // Check that sender owns the asset token
            ensure!(<asset::Module<T>>::_is_owner(&ticker, did), Error::<T>::NotAnOwner);

            // Check if sender has enough funds in payout currency
            let balance = <simple_token::BalanceOf<T>>::get((payout_ticker, did));
            ensure!(balance >= amount, Error::<T>::InsufficientFunds);

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
            ensure!(
                <asset::Module<T>>::total_checkpoints_of(&ticker) >= checkpoint_id,
                Error::<T>::NoSuchCheckpoint
            );

            let now = <pallet_timestamp::Module<T>>::get();
            let zero_ts = Zero::zero(); // A 0 timestamp
            // Check maturity/expiration dates
            match (&matures_at, &expires_at) {
                (_start, end) if  end == &zero_ts => {
                },
                (start, end) if start == &zero_ts => {
                    // Ends in the future
                    ensure!(end > &now, Error::<T>::PayoutMustEndInFuture);
                },
                (start, end) if start == &zero_ts && end == &zero_ts => {}
                (start, end) => {
                    // Ends in the future
                    ensure!(end > &now, Error::<T>::PayoutMustEndInFuture);
                    // Ends after start
                    ensure!(end > start, Error::<T>::PayoutMustEndAfterStart);
                },
            }

            // Subtract the amount
            let new_balance = balance.checked_sub(&amount).ok_or(Error::<T>::BalanceUnderflow)?;
            <<T as IdentityTrait>::ProtocolFee>::charge_fee(
                &sender,
                ProtocolOp::DividendNew
            )?;
            <simple_token::BalanceOf<T>>::insert((payout_ticker, did), new_balance);

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
        pub fn cancel(origin, ticker: Ticker, dividend_id: u32) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            let sender = Signatory::AccountKey(sender_key);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &sender),
                Error::<T>::SenderMustBeSigningKeyForDid
            );
            // Check that sender owns the asset token
            ensure!(<asset::Module<T>>::_is_owner(&ticker, did), Error::<T>::NotAnOwner);

            // Check that the dividend has not started yet
            let entry: Dividend<_, _> = Self::get_dividend(&ticker, dividend_id)
                .ok_or(Error::<T>::NoSuchDividend)?;
            let now = <pallet_timestamp::Module<T>>::get();
            ensure!(
                entry.matures_at.map_or(false, |ref start| *start > now),
                Error::<T>::MustMatureInFuture
            );

            // Pay amount back to owner
            <simple_token::BalanceOf<T>>::mutate(
                (entry.payout_currency, did),
                |balance: &mut T::Balance| -> DispatchResult {
                    *balance  = balance
                        .checked_add(&entry.amount)
                        .ok_or(Error::<T>::FailedToPayBackToOwner)?;
                    Ok(())
                }
            )?;

            <Dividends<T>>::remove((ticker, dividend_id));

            Self::deposit_event(RawEvent::DividendCanceled(ticker, dividend_id));

            Ok(())
        }

        /// Withdraws from a dividend the adequate share of the `amount` field. All dividend shares
        /// are rounded by truncation (down to first integer below)
        pub fn claim(origin, ticker: Ticker, dividend_id: u32) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            let sender = Signatory::AccountKey(sender_key);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &sender),
                Error::<T>::SenderMustBeSigningKeyForDid
            );
            // Check if sender wasn't already paid their share
            ensure!(
                !<UserPayoutCompleted>::get((did, ticker, dividend_id)),
                Error::<T>::HasAlreadyBeenPaid
            );

            // Look dividend entry up
            let dividend = Self::get_dividend(&ticker, dividend_id).ok_or(Error::<T>::NoSuchDividend)?;

            let balance_at_checkpoint =
                <asset::Module<T>>::get_balance_at(ticker, did, dividend.checkpoint_id);

            // Check if the owner hadn't yanked the remaining amount out
            ensure!(!dividend.remaining_claimed, Error::<T>::RemainingFundsAlreadyClaimed);

            let now = <pallet_timestamp::Module<T>>::get();

            // Check if the current time is within maturity/expiration bounds
            if let Some(start) = dividend.matures_at.as_ref() {
                ensure!(now > *start, Error::<T>::CannotPayBeforeMaturity);
            }

            if let Some(end) = dividend.expires_at.as_ref() {
                ensure!(*end > now, Error::<T>::CannotPayAfterExpiration);
            }

            // Compute the share
            ensure!(<asset::Tokens<T>>::exists(&ticker), Error::<T>::NoSuchToken);
            let supply_at_checkpoint = <asset::CheckpointTotalSupply<T>>::get((ticker, dividend.checkpoint_id));

            let balance_amount_product = balance_at_checkpoint
                .checked_mul(&dividend.amount)
                .ok_or(Error::<T>::BalanceAmountProductOverflowed)?;

            let share = balance_amount_product
                .checked_div(&supply_at_checkpoint)
                .ok_or(Error::<T>::BalanceAmountProductSupplyDivisionFailed)?;

            // Adjust the paid_out amount
            <Dividends<T>>::mutate((ticker, dividend_id), |entry| -> DispatchResult {
                entry.amount_left = entry.amount_left.checked_sub(&share)
                    .ok_or(Error::<T>::CouldNotIncreaseAmount)?;
                Ok(())
            })?;

            // Perform the payout in designated tokens
            <simple_token::BalanceOf<T>>::mutate(
                (dividend.payout_currency, did),
                |balance| -> DispatchResult {
                    *balance = balance.checked_add(&share).ok_or(Error::<T>::CouldNotAddShare)?;
                    Ok(())
                }
            )?;

            // Create payout entry
            <UserPayoutCompleted>::insert((did, ticker, dividend_id), true);

            // Dispatch event
            Self::deposit_event(RawEvent::DividendPaidOutToUser(did, ticker, dividend_id, share));
            Ok(())
        }

        /// After a dividend had expired, collect the remaining amount to owner address
        pub fn claim_unclaimed(origin, ticker: Ticker, dividend_id: u32) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            let sender = Signatory::AccountKey(sender_key);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &sender),
                Error::<T>::SenderMustBeSigningKeyForDid
            );
            // Check that sender owns the asset token
            ensure!(<asset::Module<T>>::_is_owner(&ticker, did), Error::<T>::NotAnOwner);

            let entry = Self::get_dividend(&ticker, dividend_id).ok_or(Error::<T>::NoSuchDividend)?;

            // Check that the expiry date had passed
            let now = <pallet_timestamp::Module<T>>::get();
            ensure!(entry.expires_at.map_or(false, |ref end| *end < now), Error::<T>::NotEnded);
            // Transfer the computed amount
            <simple_token::BalanceOf<T>>::mutate(
                (entry.payout_currency, did),
                |balance: &mut T::Balance| -> DispatchResult {
                    *balance = balance
                        .checked_add(&entry.amount_left)
                        .ok_or(Error::<T>::FailedToPayBackToOwner)?;
                    Ok(())
                }
            )?;

            // Set amount_left, flip remaining_claimed
            <Dividends<T>>::mutate((ticker, dividend_id), |entry| -> DispatchResult {
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
        Balance = <T as CommonTrait>::Balance,
    {
        /// A new dividend was created (ticker, amount, dividend ID)
        DividendCreated(Ticker, Balance, u32),

        /// A dividend was canceled (ticker, dividend ID)
        DividendCanceled(Ticker, u32),

        /// Dividend was paid to a user (who, ticker, dividend ID, share)
        DividendPaidOutToUser(IdentityId, Ticker, u32, Balance),

        /// Unclaimed dividend was claimed back (ticker, dividend ID, amount)
        DividendRemainingClaimed(Ticker, u32, Balance),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Claiming unclaimed payouts requires an end date
        NotEnded,
        /// The sender must be a signing key for the DID.
        SenderMustBeSigningKeyForDid,
        /// The dividend was not found.
        NoSuchDividend,
        /// The user is not an owner of the asset.
        NotAnOwner,
        /// Insufficient funds.
        InsufficientFunds,
        /// The checkpoint for the dividend does not exist.
        NoSuchCheckpoint,
        /// Dividend payout must end in the future.
        PayoutMustEndInFuture,
        /// Dividend payout must end after it starts.
        PayoutMustEndAfterStart,
        /// Underflow while calculating the new value for balance.
        BalanceUnderflow,
        /// Dividend must mature in the future.
        MustMatureInFuture,
        /// Failed to pay back to the owner account.
        FailedToPayBackToOwner,
        /// The user has already been paid their share.
        HasAlreadyBeenPaid,
        /// The remaining payout funds were already claimed.
        RemainingFundsAlreadyClaimed,
        /// Attempted to pay out before maturity.
        CannotPayBeforeMaturity,
        /// Attempted to pay out after expiration.
        CannotPayAfterExpiration,
        /// The dividend token entry was not found.
        NoSuchToken,
        /// Multiplication of the balance with the total payout amount overflowed.
        BalanceAmountProductOverflowed,
        /// A failed division of the balance amount product by the total supply.
        BalanceAmountProductSupplyDivisionFailed,
        /// Could not increase the paid out amount.
        CouldNotIncreaseAmount,
        /// Could not add the share to sender's balance.
        CouldNotAddShare,
    }
}

impl<T: Trait> Module<T> {
    /// A helper method for dividend creation. Returns dividend ID
    /// #[inline]
    fn add_dividend_entry(
        ticker: &Ticker,
        d: Dividend<T::Balance, T::Moment>,
    ) -> core::result::Result<u32, &'static str> {
        let old_count = <DividendCount>::get(ticker);
        let new_count = old_count
            .checked_add(1)
            .ok_or("Could not add 1 to dividend count")?;

        <Dividends<T>>::insert((*ticker, old_count), d);
        <DividendCount>::insert(*ticker, new_count);

        Ok(old_count)
    }

    /// Retrieves a dividend checking that it exists beforehand.
    pub fn get_dividend(
        ticker: &Ticker,
        dividend_id: u32,
    ) -> Option<Dividend<T::Balance, T::Moment>> {
        // Check that the dividend entry exists
        let ticker_div_id = (*ticker, dividend_id);
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
    use frame_support::traits::Currency;
    use frame_support::{
        assert_ok, dispatch::DispatchResult, impl_outer_dispatch, impl_outer_origin,
        parameter_types, weights::DispatchInfo,
    };
    use lazy_static::lazy_static;
    use sp_core::{crypto::key_types, H256};
    use sp_runtime::{
        testing::{Header, UintAuthorityId},
        traits::{BlakeTwo256, ConvertInto, IdentityLookup, OpaqueKeys, Verify},
        transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction},
        AnySignature, KeyTypeId, Perbill,
    };
    use test_client::{self, AccountKeyring};

    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };
    use system::EnsureSignedBy;

    use polymesh_primitives::IdentityId;
    use polymesh_protocol_fee as protocol_fee;
    use polymesh_runtime_balances as balances;
    use polymesh_runtime_common::traits::{
        asset::AcceptTransfer,
        group::{GroupTrait, InactiveMember},
        multisig::AddSignerMultiSig,
    };
    use polymesh_runtime_group as group;
    use polymesh_runtime_identity as identity;

    use crate::{
        asset::{AssetType, SecurityToken, TickerRegistrationConfig},
        exemption, general_tm, percentage_tm,
        simple_token::SimpleTokenRecord,
        statistics,
    };

    type SessionIndex = u32;
    type AuthorityId = <AnySignature as Verify>::Signer;
    type BlockNumber = u64;
    type OffChainSignature = AnySignature;
    type AccountId = <AnySignature as Verify>::Signer;
    type Moment = <Test as pallet_timestamp::Trait>::Moment;

    pub struct TestOnSessionEnding;
    impl pallet_session::OnSessionEnding<AuthorityId> for TestOnSessionEnding {
        fn on_session_ending(_: SessionIndex, _: SessionIndex) -> Option<Vec<AuthorityId>> {
            None
        }
    }

    pub struct TestSessionHandler;
    impl pallet_session::SessionHandler<AuthorityId> for TestSessionHandler {
        const KEY_TYPE_IDS: &'static [KeyTypeId] = &[key_types::DUMMY];
        fn on_new_session<Ks: OpaqueKeys>(
            _changed: bool,
            _validators: &[(AuthorityId, Ks)],
            _queued_validators: &[(AuthorityId, Ks)],
        ) {
        }

        fn on_disabled(_validator_index: usize) {}

        fn on_genesis_session<Ks: OpaqueKeys>(_validators: &[(AuthorityId, Ks)]) {}

        fn on_before_session_ending() {}
    }

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    impl_outer_dispatch! {
        pub enum Call for Test where origin: Origin {
            pallet_contracts::Contracts,
            identity::Identity,
        }
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

    impl frame_system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Call = Call;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = AccountId;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
        type ModuleToIndex = ();
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

    impl pallet_session::Trait for Test {
        type OnSessionEnding = TestOnSessionEnding;
        type Keys = UintAuthorityId;
        type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
        type SessionHandler = TestSessionHandler;
        type Event = ();
        type ValidatorId = AuthorityId;
        type ValidatorIdOf = ConvertInto;
        type SelectInitialValidators = ();
        type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
    }

    impl pallet_session::historical::Trait for Test {
        type FullIdentification = ();
        type FullIdentificationOf = ();
    }

    impl CommonTrait for Test {
        type Balance = u128;
        type CreationFee = CreationFee;
        type AcceptTransferTarget = Test;

        type BlockRewardsReserve = balances::Module<Test>;
    }

    impl balances::Trait for Test {
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();
        type DustRemoval = ();
        type TransferPayment = ();
        type ExistentialDeposit = ExistentialDeposit;
        type TransferFee = TransferFee;
        type Identity = identity::Module<Test>;
    }

    impl group::Trait<group::Instance2> for Test {
        type Event = ();
        type AddOrigin = EnsureSignedBy<One, AccountId>;
        type RemoveOrigin = EnsureSignedBy<Two, AccountId>;
        type SwapOrigin = EnsureSignedBy<Three, AccountId>;
        type ResetOrigin = EnsureSignedBy<Four, AccountId>;
        type MembershipInitialized = ();
        type MembershipChanged = ();
    }

    parameter_types! {
        pub const One: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Two: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Three: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Four: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Five: AccountId = AccountId::from(AccountKeyring::Dave);
    }

    impl group::Trait<group::Instance1> for Test {
        type Event = ();
        type AddOrigin = EnsureSignedBy<One, AccountId>;
        type RemoveOrigin = EnsureSignedBy<Two, AccountId>;
        type SwapOrigin = EnsureSignedBy<Three, AccountId>;
        type ResetOrigin = EnsureSignedBy<Four, AccountId>;
        type MembershipInitialized = ();
        type MembershipChanged = ();
    }

    impl simple_token::Trait for Test {
        type Event = ();
    }

    impl protocol_fee::Trait for Test {
        type Event = ();
        type Currency = Balances;
        type OnProtocolFeePayment = ();
    }

    impl asset::Trait for Test {
        type Event = ();
        type Currency = balances::Module<Test>;
    }

    impl AcceptTransfer for Test {
        fn accept_ticker_transfer(_to_did: IdentityId, _auth_id: u64) -> DispatchResult {
            unimplemented!();
        }

        fn accept_token_ownership_transfer(_to_did: IdentityId, _auth_id: u64) -> DispatchResult {
            unimplemented!();
        }
    }

    impl statistics::Trait for Test {}

    impl identity::Trait for Test {
        type Event = ();
        type Proposal = Call;
        type AddSignerMultiSigTarget = Test;
        type CddServiceProviders = Test;
        type Balances = balances::Module<Test>;
        type ChargeTxFeeTarget = Test;
        type CddHandler = Test;
        type Public = AccountId;
        type OffChainSignature = OffChainSignature;
        type ProtocolFee = protocol_fee::Module<Test>;
    }

    impl pallet_transaction_payment::CddAndFeeDetails<Call> for Test {
        fn get_valid_payer(
            _: &Call,
            _: &Signatory,
        ) -> Result<Option<Signatory>, InvalidTransaction> {
            Ok(None)
        }
        fn clear_context() {}
        fn set_payer_context(_: Option<Signatory>) {}
        fn get_payer_from_context() -> Option<Signatory> {
            None
        }
        fn set_current_identity(_: &IdentityId) {}
    }

    impl pallet_transaction_payment::ChargeTxFee for Test {
        fn charge_fee(_len: u32, _info: DispatchInfo) -> TransactionValidity {
            Ok(ValidTransaction::default())
        }
    }

    impl GroupTrait<Moment> for Test {
        fn get_members() -> Vec<IdentityId> {
            vec![]
        }
        fn get_inactive_members() -> Vec<InactiveMember<Moment>> {
            unimplemented!();
        }
        fn disable_member(
            _who: IdentityId,
            _expiry: Option<Moment>,
            _at: Option<Moment>,
        ) -> DispatchResult {
            unimplemented!();
        }
    }

    impl AddSignerMultiSig for Test {
        fn accept_multisig_signer(_: Signatory, _: u64) -> DispatchResult {
            unimplemented!()
        }
    }

    impl exemption::Trait for Test {
        type Event = ();
        type Asset = asset::Module<Test>;
    }

    impl general_tm::Trait for Test {
        type Event = ();
        type Asset = asset::Module<Test>;
    }

    impl percentage_tm::Trait for Test {
        type Event = ();
    }

    parameter_types! {
        pub const SignedClaimHandicap: u64 = 2;
        pub const TombstoneDeposit: u64 = 16;
        pub const StorageSizeOffset: u32 = 8;
        pub const RentByteFee: u64 = 4;
        pub const RentDepositOffset: u64 = 10_000;
        pub const SurchargeReward: u64 = 150;
        pub const ContractTransactionBaseFee: u64 = 2;
        pub const ContractTransactionByteFee: u64 = 6;
        pub const ContractFee: u64 = 21;
        pub const CallBaseFee: u64 = 135;
        pub const InstantiateBaseFee: u64 = 175;
        pub const MaxDepth: u32 = 100;
        pub const MaxValueSize: u32 = 16_384;
        pub const ContractTransferFee: u64 = 50000;
        pub const ContractCreationFee: u64 = 50;
        pub const BlockGasLimit: u64 = 10000000;
    }

    impl pallet_contracts::Trait for Test {
        type Currency = Balances;
        type Time = Timestamp;
        type Randomness = Randomness;
        type Call = Call;
        type Event = ();
        type DetermineContractAddress = pallet_contracts::SimpleAddressDeterminator<Test>;
        type ComputeDispatchFee = pallet_contracts::DefaultDispatchFeeComputor<Test>;
        type TrieIdGenerator = pallet_contracts::TrieIdFromParentCounter<Test>;
        type GasPayment = ();
        type RentPayment = ();
        type SignedClaimHandicap = SignedClaimHandicap;
        type TombstoneDeposit = TombstoneDeposit;
        type StorageSizeOffset = StorageSizeOffset;
        type RentByteFee = RentByteFee;
        type RentDepositOffset = RentDepositOffset;
        type SurchargeReward = SurchargeReward;
        type TransferFee = ContractTransferFee;
        type CreationFee = ContractCreationFee;
        type TransactionBaseFee = ContractTransactionBaseFee;
        type TransactionByteFee = ContractTransactionByteFee;
        type ContractFee = ContractFee;
        type CallBaseFee = CallBaseFee;
        type InstantiateBaseFee = InstantiateBaseFee;
        type MaxDepth = MaxDepth;
        type MaxValueSize = MaxValueSize;
        type BlockGasLimit = BlockGasLimit;
    }

    parameter_types! {
        pub const MinimumPeriod: u64 = 3;
    }

    impl pallet_timestamp::Trait for Test {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = MinimumPeriod;
    }

    impl Trait for Test {
        type Event = ();
    }

    impl asset::AssetTrait<<Test as CommonTrait>::Balance, AccountId> for Module<Test> {
        fn is_owner(ticker: &Ticker, sender_did: IdentityId) -> bool {
            if let Some(token) = TOKEN_MAP.lock().unwrap().get(ticker) {
                token.owner_did == sender_did
            } else {
                false
            }
        }

        fn _mint_from_sto(
            _ticker: &Ticker,
            _caller: AccountId,
            _sender_did: IdentityId,
            _tokens_purchased: <Test as CommonTrait>::Balance,
        ) -> DispatchResult {
            unimplemented!();
        }

        /// Get the asset `id` balance of `who`.
        fn balance(_ticker: &Ticker, _did: IdentityId) -> <Test as CommonTrait>::Balance {
            unimplemented!();
        }

        // Get the total supply of an asset `id`
        fn total_supply(_ticker: &Ticker) -> <Test as CommonTrait>::Balance {
            unimplemented!();
        }

        fn get_balance_at(
            _ticker: &Ticker,
            _did: IdentityId,
            _at: u64,
        ) -> <Test as CommonTrait>::Balance {
            unimplemented!();
        }
    }

    lazy_static! {
        static ref TOKEN_MAP: Arc<
            Mutex<
            HashMap<
            Ticker,
            SecurityToken<
                <Test as CommonTrait>::Balance,
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
    type Timestamp = pallet_timestamp::Module<Test>;
    type Randomness = pallet_randomness_collective_flip::Module<Test>;
    type Contracts = pallet_contracts::Module<Test>;

    /// Build a genesis identity instance owned by the specified account
    fn identity_owned_by_1() -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        identity::GenesisConfig::<Test> {
            owner: AccountKeyring::Alice.public().into(),
            ..Default::default()
        }
        .assimilate_storage(&mut t)
        .unwrap();
        asset::GenesisConfig::<Test> {
            asset_creation_fee: 0,
            ticker_registration_fee: 0,
            ticker_registration_config: TickerRegistrationConfig {
                max_ticker_length: 12,
                registration_length: Some(10000),
            },
            fee_collector: AccountKeyring::Dave.public().into(),
        }
        .assimilate_storage(&mut t)
        .unwrap();
        sp_io::TestExternalities::new(t)
    }

    fn make_account(
        account_id: &AccountId,
    ) -> StdResult<(<Test as frame_system::Trait>::Origin, IdentityId), &'static str> {
        let signed_id = Origin::signed(account_id.clone());
        Balances::make_free_balance_be(&account_id, 1_000_000);
        let _ = Identity::register_did(signed_id.clone(), vec![]);
        let did = Identity::get_identity(&AccountKey::try_from(account_id.encode())?).unwrap();
        Ok((signed_id, did))
    }

    #[test]
    fn correct_dividend_must_work() {
        identity_owned_by_1().execute_with(|| {
            let token_owner_acc = AccountId::from(AccountKeyring::Alice);
            let (token_owner_signed, token_owner_did) = make_account(&token_owner_acc).unwrap();

            let payout_owner_acc = AccountId::from(AccountKeyring::Bob);
            let (payout_owner_signed, payout_owner_did) = make_account(&payout_owner_acc).unwrap();

            // A token representing 1M shares
            let token = SecurityToken {
                name: [b'A'; 12].into(),
                owner_did: token_owner_did,
                total_supply: 1_000_000,
                divisible: true,
                asset_type: AssetType::default(),
                ..Default::default()
            };
            let ticker = Ticker::from(token.name.as_slice());
            // A token used for payout
            let payout_token = SimpleTokenRecord {
                ticker: Ticker::from(&[b'B'; 12][..]),
                owner_did: payout_owner_did,
                total_supply: 200_000_000,
            };

            Balances::make_free_balance_be(&token_owner_acc, 1_000_000);

            Balances::make_free_balance_be(&payout_owner_acc, 1_000_000);
            // Share issuance is successful
            assert_ok!(Asset::create_token(
                token_owner_signed.clone(),
                token.name.clone(),
                ticker,
                token.total_supply,
                true,
                token.asset_type.clone(),
                vec![],
                None
            ));

            // Issuance for payout token is successful
            assert_ok!(SimpleToken::create_token(
                payout_owner_signed.clone(),
                payout_token.ticker,
                payout_token.total_supply
            ));

            // Prepare a whitelisted investor
            let investor_acc = AccountId::from(AccountKeyring::Charlie);
            let (investor_signed, investor_did) = make_account(&investor_acc).unwrap();
            Balances::make_free_balance_be(&investor_acc, 1_000_000);

            let amount_invested = 50_000;

            let now = Utc::now();
            <pallet_timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);

            // We need a lock to exist till assertions are done
            let outer = TOKEN_MAP_OUTER_LOCK.lock().unwrap();
            *TOKEN_MAP.lock().unwrap() = {
                let mut map = HashMap::new();
                map.insert(ticker, token.clone());
                map
            };

            drop(outer);

            // Allow all transfers
            assert_ok!(GeneralTM::add_active_rule(
                token_owner_signed.clone(),
                ticker,
                vec![],
                vec![]
            ));

            // Transfer tokens to investor
            assert_ok!(Asset::transfer(
                token_owner_signed.clone(),
                ticker,
                investor_did,
                amount_invested
            ));

            // Create checkpoint for token
            assert_ok!(Asset::create_checkpoint(token_owner_signed.clone(), ticker));

            // Checkpoints are 1-indexed
            let checkpoint_id = 1;

            let dividend = Dividend {
                amount: 500_000,
                amount_left: 500_000,
                remaining_claimed: false,
                matures_at: Some((now - Duration::hours(1)).timestamp() as u64),
                expires_at: Some((now + Duration::hours(1)).timestamp() as u64),
                payout_currency: payout_token.ticker,
                checkpoint_id,
            };

            // Transfer payout tokens to asset owner
            assert_ok!(SimpleToken::transfer(
                payout_owner_signed.clone(),
                payout_token.ticker,
                token_owner_did,
                dividend.amount
            ));

            // Create the dividend for asset
            assert_ok!(DividendModule::new(
                token_owner_signed.clone(),
                dividend.amount,
                ticker,
                dividend.matures_at.clone().unwrap(),
                dividend.expires_at.clone().unwrap(),
                dividend.payout_currency.clone(),
                dividend.checkpoint_id
            ));

            // Compare created dividend with the expected structure
            assert_eq!(
                DividendModule::get_dividend(&ticker, 0),
                Some(dividend.clone())
            );

            // Claim investor's share
            assert_ok!(DividendModule::claim(investor_signed.clone(), ticker, 0,));

            // Check if the correct amount was added to investor balance
            let share = dividend.amount * amount_invested / token.total_supply;
            assert_eq!(
                SimpleToken::balance_of((payout_token.ticker, investor_did)),
                share
            );

            // Check if amount_left was adjusted correctly
            let current_entry =
                DividendModule::get_dividend(&ticker, 0).expect("Could not retrieve dividend");
            assert_eq!(current_entry.amount_left, current_entry.amount - share);
        });
    }
}
