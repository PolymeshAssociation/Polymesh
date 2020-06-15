// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Test utilities

use crate::{
    inflation, EraIndex, ErasRewardPoints, ErasStakers, Exposure, GenesisConfig, Module,
    Nominators, RewardDestination, SessionInterface, StakerStatus, Trait, ValidatorPrefs,
};
use chrono::prelude::Utc;
use codec::Encode;
use frame_support::{
    assert_ok,
    dispatch::DispatchResult,
    impl_outer_dispatch, impl_outer_origin, ord_parameter_types, parameter_types,
    traits::{Currency, FindAuthor, Get},
    weights::{DispatchInfo, Weight},
    StorageDoubleMap, StorageLinkedMap, StorageMap, StorageValue,
};
use frame_system::{self as system, EnsureSignedBy};
use pallet_balances as balances;
use pallet_group as group;
use pallet_identity::{self as identity};
use pallet_protocol_fee as protocol_fee;
use polymesh_common_utilities::traits::{
    asset::AcceptTransfer,
    balances::{AccountData, CheckCdd},
    group::{GroupTrait, InactiveMember},
    identity::Trait as IdentityTrait,
    multisig::MultiSigSubTrait,
    CommonTrait,
};
use primitives::traits::BlockRewardsReserveCurrency;
use primitives::{AccountKey, Claim, IdentityId, Signatory};
use sp_core::{
    crypto::{key_types, Pair as PairTrait},
    sr25519::Pair,
    H256,
};
use sp_io;
use sp_runtime::curve::PiecewiseLinear;
use sp_runtime::testing::{sr25519::Public, Header, UintAuthorityId};
use sp_runtime::traits::{
    Convert, IdentityLookup, OnFinalize, OnInitialize, OpaqueKeys, SaturatedConversion, Verify,
    Zero,
};
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction};
use sp_runtime::{AnySignature, KeyTypeId, Perbill};
use sp_staking::{
    offence::{OffenceDetails, OnOffenceHandler},
    SessionIndex,
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
    convert::TryFrom,
};
use test_client::AccountKeyring;

/// The AccountId alias in this test module.
pub type AccountId = <AnySignature as Verify>::Signer;
pub type BlockNumber = u64;
pub type Balance = u128;
type OffChainSignature = AnySignature;
type Moment = <Test as pallet_timestamp::Trait>::Moment;

/// Simple structure that exposes how u64 currency can be represented as... u64.
pub struct CurrencyToVoteHandler;
impl Convert<u64, u64> for CurrencyToVoteHandler {
    fn convert(x: u64) -> u64 {
        x
    }
}
impl Convert<u128, u64> for CurrencyToVoteHandler {
    fn convert(x: u128) -> u64 {
        x.saturated_into()
    }
}
impl Convert<u128, u128> for CurrencyToVoteHandler {
    fn convert(x: u128) -> u128 {
        x.saturated_into()
    }
}

thread_local! {
    static SESSION: RefCell<(Vec<AccountId>, HashSet<AccountId>)> = RefCell::new(Default::default());
    static EXISTENTIAL_DEPOSIT: RefCell<u128> = RefCell::new(0);
    static SLASH_DEFER_DURATION: RefCell<EraIndex> = RefCell::new(0);
}

pub struct TestSessionHandler;
impl pallet_session::SessionHandler<AccountId> for TestSessionHandler {
    const KEY_TYPE_IDS: &'static [KeyTypeId] = &[key_types::DUMMY];

    fn on_genesis_session<Ks: OpaqueKeys>(_validators: &[(AccountId, Ks)]) {}

    fn on_new_session<Ks: OpaqueKeys>(
        _changed: bool,
        validators: &[(AccountId, Ks)],
        _queued_validators: &[(AccountId, Ks)],
    ) {
        SESSION.with(|x| {
            *x.borrow_mut() = (
                validators.iter().map(|x| x.0.clone()).collect(),
                HashSet::new(),
            )
        });
    }

    fn on_disabled(validator_index: usize) {
        SESSION.with(|d| {
            let mut d = d.borrow_mut();
            let value = d.0[validator_index];
            d.1.insert(value);
        })
    }
}

pub fn is_disabled(controller: AccountId) -> bool {
    let stash = Staking::ledger(&controller).unwrap().stash;
    SESSION.with(|d| d.borrow().1.contains(&stash))
}

pub struct ExistentialDeposit;
impl Get<u128> for ExistentialDeposit {
    fn get() -> u128 {
        EXISTENTIAL_DEPOSIT.with(|v| *v.borrow())
    }
}

pub struct SlashDeferDuration;
impl Get<EraIndex> for SlashDeferDuration {
    fn get() -> EraIndex {
        SLASH_DEFER_DURATION.with(|v| *v.borrow())
    }
}

impl_outer_origin! {
    pub enum Origin for Test {}
}

impl_outer_dispatch! {
    pub enum Call for Test where origin: Origin {
        identity::Identity,
    }
}

/// Author of block is always 11
pub struct Author11;
impl FindAuthor<AccountId> for Author11 {
    fn find_author<'a, I>(_digests: I) -> Option<AccountId>
    where
        I: 'a + IntoIterator<Item = (frame_support::ConsensusEngineId, &'a [u8])>,
    {
        //Some(11)
        Some(account_from(11))
    }
}

// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Test;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
}
impl frame_system::Trait for Test {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Call = Call;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type AvailableBlockRatio = AvailableBlockRatio;
    type MaximumBlockLength = MaximumBlockLength;
    type Version = ();
    type ModuleToIndex = ();
    type AccountData = AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
}

impl CommonTrait for Test {
    type Balance = Balance;
    type AcceptTransferTarget = Test;
    type BlockRewardsReserve = balances::Module<Test>;
}

parameter_types! {
    pub const TransactionBaseFee: Balance = 0;
    pub const TransactionByteFee: Balance = 0;
}

impl balances::Trait for Test {
    type DustRemoval = ();
    type Event = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = frame_system::Module<Test>;
    type Identity = identity::Module<Test>;
    type CddChecker = Test;
}

ord_parameter_types! {
    pub const One: AccountId = AccountId::from(AccountKeyring::Dave);
    pub const Two: AccountId = AccountId::from(AccountKeyring::Dave);
    pub const Three: AccountId = AccountId::from(AccountKeyring::Dave);
    pub const Four: AccountId = AccountId::from(AccountKeyring::Dave);
    pub const Five: AccountId = AccountId::from(AccountKeyring::Dave);
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

impl protocol_fee::Trait for Test {
    type Event = ();
    type Currency = Balances;
    type OnProtocolFeePayment = ();
}

impl IdentityTrait for Test {
    type Event = ();
    type Proposal = Call;
    type MultiSig = Test;
    type CddServiceProviders = group::Module<Test, group::Instance2>;
    type Balances = balances::Module<Test>;
    type ChargeTxFeeTarget = Test;
    type CddHandler = Test;
    type Public = AccountId;
    type OffChainSignature = OffChainSignature;
    type ProtocolFee = protocol_fee::Module<Test>;
}

impl pallet_transaction_payment::CddAndFeeDetails<Call> for Test {
    fn get_valid_payer(_: &Call, _: &Signatory) -> Result<Option<Signatory>, InvalidTransaction> {
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
        return Group::active_members();
    }

    fn get_inactive_members() -> Vec<InactiveMember<Moment>> {
        vec![]
    }

    fn disable_member(
        _who: IdentityId,
        _expiry: Option<Moment>,
        _at: Option<Moment>,
    ) -> DispatchResult {
        unimplemented!();
    }

    fn get_active_members() -> Vec<IdentityId> {
        Self::get_members()
    }

    /// Current set size
    fn member_count() -> usize {
        Self::get_members().len()
    }

    fn is_member(member_id: &IdentityId) -> bool {
        Self::get_members().contains(member_id)
    }

    /// It returns the current "active members" and any "inactive member" which its
    /// expiration time-stamp is greater than `moment`.
    fn get_valid_members_at(moment: Moment) -> Vec<IdentityId> {
        Self::get_active_members()
            .into_iter()
            .chain(
                Self::get_inactive_members()
                    .into_iter()
                    .filter(|m| !Self::is_member_expired(&m, moment))
                    .map(|m| m.id),
            )
            .collect::<Vec<_>>()
    }

    fn is_member_expired(member: &InactiveMember<Moment>, now: Moment) -> bool {
        false
    }
}

impl AcceptTransfer for Test {
    fn accept_ticker_transfer(_: IdentityId, _: u64) -> DispatchResult {
        Ok(())
    }
    fn accept_asset_ownership_transfer(_: IdentityId, _: u64) -> DispatchResult {
        Ok(())
    }
}

impl MultiSigSubTrait for Test {
    fn accept_multisig_signer(_: Signatory, _: u64) -> DispatchResult {
        unimplemented!()
    }
    fn get_key_signers(multisig: AccountKey) -> Vec<AccountKey> {
        unimplemented!()
    }
    fn is_multisig(account: AccountKey) -> bool {
        unimplemented!()
    }
}

impl CheckCdd for Test {
    fn check_key_cdd(key: &AccountKey) -> bool {
        true
    }
    fn get_key_cdd_did(key: &AccountKey) -> Option<IdentityId> {
        None
    }
}

parameter_types! {
    pub const Period: BlockNumber = 1;
    pub const Offset: BlockNumber = 0;
    pub const UncleGenerations: u64 = 0;
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(25);
}
impl pallet_session::Trait for Test {
    type Event = ();
    type ValidatorId = AccountId;
    type ValidatorIdOf = crate::StashOf<Test>;
    type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
    type SessionManager = pallet_session::historical::NoteHistoricalRoot<Test, Staking>;
    type SessionHandler = TestSessionHandler;
    type Keys = UintAuthorityId;
    type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
}

impl pallet_session::historical::Trait for Test {
    type FullIdentification = crate::Exposure<AccountId, Balance>;
    type FullIdentificationOf = crate::ExposureOf<Test>;
}
impl pallet_authorship::Trait for Test {
    type FindAuthor = Author11;
    type UncleGenerations = UncleGenerations;
    type FilterUncle = ();
    type EventHandler = Module<Test>;
}

parameter_types! {
    pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Trait for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
}

parameter_types! {
    pub const EpochDuration: u64 = 10;
    pub const ExpectedBlockTime: u64 = 1;
}

impl pallet_babe::Trait for Test {
    type EpochDuration = EpochDuration;
    type ExpectedBlockTime = ExpectedBlockTime;
    type EpochChangeTrigger = pallet_babe::ExternalTrigger;
}

pallet_staking_reward_curve::build! {
    const I_NPOS: PiecewiseLinear<'static> = curve!(
        min_inflation: 0_025_000,
        max_inflation: 0_100_000,
        ideal_stake: 0_500_000,
        falloff: 0_050_000,
        max_piece_count: 40,
        test_precision: 0_005_000,
    );
}

parameter_types! {
    pub const SessionsPerEra: SessionIndex = 3;
    pub const BondingDuration: EraIndex = 3;
    pub const RewardCurve: &'static PiecewiseLinear<'static> = &I_NPOS;
    pub const MaxNominatorRewardedPerValidator: u32 = 64;
}

ord_parameter_types! {
    pub const OneThousand: Public = account_from(1000);
    pub const TwoThousand: Public = account_from(2000);
    pub const ThreeThousand: Public = account_from(3000);
    pub const FourThousand: Public = account_from(4000);
    pub const FiveThousand: Public = account_from(5000);
}

impl Trait for Test {
    type Currency = balances::Module<Self>;
    type Time = pallet_timestamp::Module<Self>;
    type CurrencyToVote = CurrencyToVoteHandler;
    type RewardRemainder = ();
    type Event = ();
    type Slash = ();
    type Reward = ();
    type SessionsPerEra = SessionsPerEra;
    type BondingDuration = BondingDuration;
    type SlashDeferDuration = SlashDeferDuration;
    type SlashCancelOrigin = frame_system::EnsureRoot<Self::AccountId>;
    type SessionInterface = Self;
    type RewardCurve = RewardCurve;
    type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
    type RequiredAddOrigin = frame_system::EnsureRoot<AccountId>;
    type RequiredRemoveOrigin = EnsureSignedBy<TwoThousand, Self::AccountId>;
    type RequiredComplianceOrigin = EnsureSignedBy<ThreeThousand, Self::AccountId>;
    type RequiredCommissionOrigin = EnsureSignedBy<FourThousand, Self::AccountId>;
    type RequiredChangeHistoryDepthOrigin = EnsureSignedBy<FiveThousand, Self::AccountId>;
}

pub struct ExtBuilder {
    existential_deposit: u128,
    validator_pool: bool,
    nominate: bool,
    validator_count: u32,
    minimum_validator_count: u32,
    slash_defer_duration: EraIndex,
    fair: bool,
    num_validators: Option<u32>,
    invulnerables: Vec<AccountId>,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            existential_deposit: 0,
            validator_pool: false,
            nominate: true,
            validator_count: 2,
            minimum_validator_count: 0,
            slash_defer_duration: 0,
            fair: true,
            num_validators: None,
            invulnerables: vec![],
        }
    }
}

impl ExtBuilder {
    pub fn existential_deposit(mut self, existential_deposit: u128) -> Self {
        self.existential_deposit = existential_deposit;
        self
    }
    pub fn validator_pool(mut self, validator_pool: bool) -> Self {
        self.validator_pool = validator_pool;
        self
    }
    pub fn nominate(mut self, nominate: bool) -> Self {
        self.nominate = nominate;
        self
    }
    pub fn validator_count(mut self, count: u32) -> Self {
        self.validator_count = count;
        self
    }
    pub fn minimum_validator_count(mut self, count: u32) -> Self {
        self.minimum_validator_count = count;
        self
    }
    pub fn slash_defer_duration(mut self, eras: EraIndex) -> Self {
        self.slash_defer_duration = eras;
        self
    }
    pub fn fair(mut self, is_fair: bool) -> Self {
        self.fair = is_fair;
        self
    }
    pub fn num_validators(mut self, num_validators: u32) -> Self {
        self.num_validators = Some(num_validators);
        self
    }
    pub fn invulnerables(mut self, invulnerables: Vec<AccountId>) -> Self {
        self.invulnerables = invulnerables;
        self
    }
    pub fn set_associated_consts(&self) {
        EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = self.existential_deposit);
        SLASH_DEFER_DURATION.with(|v| *v.borrow_mut() = self.slash_defer_duration);
    }
    pub fn build(self) -> sp_io::TestExternalities {
        self.set_associated_consts();
        let mut storage = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        let balance_factor = if self.existential_deposit > 0 { 256 } else { 1 };

        let num_validators = self.num_validators.unwrap_or(self.validator_count);
        let validators = (0..num_validators)
            .map(|x| ((x + 1) * 10 + 1) as u64)
            .collect::<Vec<_>>();

        let account_key_ring: BTreeMap<u64, Public> = [
            1, 2, 3, 4, 10, 11, 20, 21, 30, 31, 40, 41, 100, 101, 999, 1005,
        ]
        .iter()
        .map(|id| (*id, account_from(*id)))
        .collect();

        let _ = balances::GenesisConfig::<Test> {
            balances: vec![
                (AccountKeyring::Alice.public(), 10 * balance_factor),
                (AccountKeyring::Bob.public(), 20 * balance_factor),
                (AccountKeyring::Charlie.public(), 300 * balance_factor),
                (AccountKeyring::Dave.public(), 400 * balance_factor),
                (
                    account_key_ring.get(&1).unwrap().clone(),
                    10 * balance_factor,
                ),
                (
                    account_key_ring.get(&2).unwrap().clone(),
                    20 * balance_factor,
                ),
                (
                    account_key_ring.get(&3).unwrap().clone(),
                    300 * balance_factor,
                ),
                (
                    account_key_ring.get(&4).unwrap().clone(),
                    400 * balance_factor,
                ),
                (account_key_ring.get(&10).unwrap().clone(), balance_factor),
                (
                    account_key_ring.get(&11).unwrap().clone(),
                    balance_factor * 1000,
                ),
                (account_key_ring.get(&20).unwrap().clone(), balance_factor),
                (
                    account_key_ring.get(&21).unwrap().clone(),
                    balance_factor * 2000,
                ),
                (account_key_ring.get(&30).unwrap().clone(), balance_factor),
                (
                    account_key_ring.get(&31).unwrap().clone(),
                    balance_factor * 2000,
                ),
                (account_key_ring.get(&40).unwrap().clone(), balance_factor),
                (
                    account_key_ring.get(&41).unwrap().clone(),
                    balance_factor * 2000,
                ),
                (
                    account_key_ring.get(&100).unwrap().clone(),
                    2000 * balance_factor,
                ),
                (
                    account_key_ring.get(&101).unwrap().clone(),
                    2000 * balance_factor,
                ),
                // This allow us to have a total_payout different from 0.
                (
                    account_key_ring.get(&999).unwrap().clone(),
                    1_000_000_000_000,
                ),
            ],
            identity_balances: vec![],
        }
        .assimilate_storage(&mut storage);

        group::GenesisConfig::<Test, group::Instance2> {
            active_members: vec![IdentityId::from(1), IdentityId::from(2)],
            phantom: Default::default(),
        }
        .assimilate_storage(&mut storage);

        let _ = identity::GenesisConfig::<Test> {
            identities: vec![
                /// (master_account_id, service provider did, target did, expiry time of CustomerDueDiligence claim i.e 10 days is ms)
                /// Provide Identity
                (
                    account_key_ring.get(&1005).unwrap().clone(),
                    IdentityId::from(1),
                    IdentityId::from(1),
                    None,
                ),
                (
                    account_key_ring.get(&11).unwrap().clone(),
                    IdentityId::from(1),
                    IdentityId::from(11),
                    None,
                ),
                (
                    account_key_ring.get(&21).unwrap().clone(),
                    IdentityId::from(1),
                    IdentityId::from(21),
                    None,
                ),
                (
                    account_key_ring.get(&31).unwrap().clone(),
                    IdentityId::from(1),
                    IdentityId::from(31),
                    None,
                ),
                (
                    account_key_ring.get(&41).unwrap().clone(),
                    IdentityId::from(1),
                    IdentityId::from(41),
                    None,
                ),
                (
                    account_key_ring.get(&101).unwrap().clone(),
                    IdentityId::from(1),
                    IdentityId::from(101),
                    None,
                ),
            ],
            ..Default::default()
        }
        .assimilate_storage(&mut storage);

        let stake_21 = if self.fair { 1000 } else { 2000 };
        let stake_31 = if self.validator_pool {
            balance_factor * 1000
        } else {
            1
        };
        let status_41 = if self.validator_pool {
            StakerStatus::<AccountId>::Validator
        } else {
            StakerStatus::<AccountId>::Idle
        };
        let nominated = if self.nominate {
            vec![
                account_key_ring.get(&11).unwrap().clone(),
                account_key_ring.get(&21).unwrap().clone(),
            ]
        } else {
            vec![]
        };
        let _ = GenesisConfig::<Test> {
            stakers: vec![
                // (stash, controller, staked_amount, status)
                (
                    account_key_ring.get(&11).unwrap().clone(),
                    account_key_ring.get(&10).unwrap().clone(),
                    balance_factor * 1000,
                    StakerStatus::<AccountId>::Validator,
                ),
                (
                    account_key_ring.get(&21).unwrap().clone(),
                    account_key_ring.get(&20).unwrap().clone(),
                    stake_21,
                    StakerStatus::<AccountId>::Validator,
                ),
                (
                    account_key_ring.get(&31).unwrap().clone(),
                    account_key_ring.get(&30).unwrap().clone(),
                    stake_31,
                    StakerStatus::<AccountId>::Validator,
                ),
                (
                    account_key_ring.get(&41).unwrap().clone(),
                    account_key_ring.get(&40).unwrap().clone(),
                    balance_factor * 1000,
                    status_41,
                ),
                // nominator
                (
                    account_key_ring.get(&101).unwrap().clone(),
                    account_key_ring.get(&100).unwrap().clone(),
                    balance_factor * 500,
                    StakerStatus::<AccountId>::Nominator(nominated),
                ),
            ],
            validator_count: self.validator_count,
            minimum_validator_count: self.minimum_validator_count,
            invulnerables: self.invulnerables,
            slash_reward_fraction: Perbill::from_percent(10),
            ..Default::default()
        }
        .assimilate_storage(&mut storage);

        let _ = pallet_session::GenesisConfig::<Test> {
            keys: validators
                .iter()
                .map(|x| {
                    let uint_auth_id = UintAuthorityId(*x);
                    (account_from(*x), account_from(*x), uint_auth_id)
                })
                .collect(),
        }
        .assimilate_storage(&mut storage);

        let mut ext = sp_io::TestExternalities::from(storage);
        ext.execute_with(|| {
            let validators = Session::validators();
            SESSION.with(|x| *x.borrow_mut() = (validators.clone(), HashSet::new()));
        });
        ext
    }
}

pub type System = frame_system::Module<Test>;
pub type Session = pallet_session::Module<Test>;
pub type Timestamp = pallet_timestamp::Module<Test>;
pub type Identity = identity::Module<Test>;
pub type Balances = balances::Module<Test>;
pub type Group = group::Module<Test, group::Instance2>;
pub type Staking = Module<Test>;

pub fn account_from(id: u64) -> AccountId {
    let mut enc_id_vec = id.encode();
    enc_id_vec.resize_with(32, Default::default);

    let mut enc_id = [0u8; 32];
    enc_id.copy_from_slice(enc_id_vec.as_slice());

    Pair::from_seed(&enc_id).public()
}

pub fn check_exposure_all(era: EraIndex) {
    ErasStakers::<Test>::iter_prefix(era).for_each(check_exposure)
}

pub fn check_nominator_all(era: EraIndex) {
    <Nominators<Test>>::enumerate().for_each(|(acc, _)| check_nominator_exposure(era, acc));
}

/// Check for each selected validator: expo.total = Sum(expo.other) + expo.own
pub fn check_exposure(expo: Exposure<AccountId, Balance>) {
    assert_eq!(
        expo.total as u128,
        expo.own as u128 + expo.others.iter().map(|e| e.value as u128).sum::<u128>(),
        "wrong total exposure {:?}",
        expo,
    );
}

/// Check that for each nominator: slashable_balance > sum(used_balance)
/// Note: we might not consume all of a nominator's balance, but we MUST NOT over spend it.
pub fn check_nominator_exposure(era: EraIndex, stash: AccountId) {
    assert_is_stash(stash);
    let mut sum = 0;
    ErasStakers::<Test>::iter_prefix(era).for_each(|exposure| {
        exposure
            .others
            .iter()
            .filter(|i| i.who == stash)
            .for_each(|i| sum += i.value)
    });
    let nominator_stake = Staking::slashable_balance_of(&stash);
    // a nominator cannot over-spend.
    assert!(
        nominator_stake >= sum,
        "failed: Nominator({}) stake({}) >= sum divided({})",
        stash,
        nominator_stake,
        sum,
    );
}

pub fn assert_is_stash(acc: AccountId) {
    assert!(Staking::bonded(&acc).is_some(), "Not a stash.");
}

pub fn assert_ledger_consistent(stash_acc: u64) {
    let stash = account_from(stash_acc);
    assert_is_stash(stash);
    let ledger = Staking::ledger(account_from(stash_acc - 1)).unwrap();

    let real_total: Balance = ledger
        .unlocking
        .iter()
        .fold(ledger.active, |a, c| a + c.value);
    assert_eq!(real_total, ledger.total);
}

pub fn get_account_key_ring(acc: u64) -> Public {
    let account_key_ring: BTreeMap<u64, Public> = [10, 11, 20, 21, 30, 31, 40, 41, 100, 101, 999]
        .iter()
        .map(|id| (*id, account_from(*id)))
        .collect();
    account_key_ring.get(&(acc)).unwrap().clone()
}

pub fn bond_validator(acc: u64, val: u128) {
    // a = controller
    // a + 1 = stash
    let controller = account_from(acc);
    let stash = account_from(acc + 1);
    let _ = Balances::make_free_balance_be(&(stash), val);
    assert_ok!(Staking::bond(
        Origin::signed(stash),
        controller,
        val,
        RewardDestination::Controller
    ));
    create_did_and_add_claim(stash);
    assert_ok!(Staking::add_permissioned_validator(
        Origin::system(frame_system::RawOrigin::Root),
        stash
    ));
    assert_ok!(Staking::validate(
        Origin::signed(controller),
        ValidatorPrefs::default()
    ));
}

pub fn bond_nominator(acc: u64, val: u128, target: Vec<AccountId>) {
    // a = controller
    // a + 1 = stash
    let controller = account_from(acc);
    let stash = account_from(acc + 1);
    let _ = Balances::make_free_balance_be(&(stash), val);
    assert_ok!(Staking::bond(
        Origin::signed(controller),
        controller,
        val,
        RewardDestination::Controller
    ));
    create_did_and_add_claim(stash);
    assert_ok!(Staking::nominate(Origin::signed(controller), target));
}

pub fn bond_nominator_with_expiry(acc: u64, val: u128, claim_expiry: u64, target: Vec<AccountId>) {
    // a = controller
    // a + 1 = stash
    let controller = account_from(acc);
    let stash = account_from(acc + 1);
    let _ = Balances::make_free_balance_be(&(stash), val);
    assert_ok!(Staking::bond(
        Origin::signed(stash),
        controller,
        val,
        RewardDestination::Controller
    ));
    create_did_and_add_claim_with_expiry(stash, claim_expiry);
    assert_ok!(Staking::nominate(Origin::signed(controller), target));
}

pub fn add_nominator_claim(
    claim_issuer: IdentityId,
    idendity_id: IdentityId,
    claim_issuer_account_id: AccountId,
) {
    let signed_claim_issuer_id = Origin::signed(claim_issuer_account_id.clone());
    let now = Utc::now();
    assert_ok!(Identity::add_claim(
        signed_claim_issuer_id,
        idendity_id,
        Claim::CustomerDueDiligence,
        Some((now.timestamp() as u64 + 10000_u64).into()),
    ));
}

pub fn add_nominator_claim_with_expiry(
    claim_issuer: IdentityId,
    idendity_id: IdentityId,
    claim_issuer_account_id: AccountId,
    expiry: u64,
) {
    let signed_claim_issuer_id = Origin::signed(claim_issuer_account_id.clone());
    assert_ok!(Identity::add_claim(
        signed_claim_issuer_id,
        idendity_id,
        Claim::CustomerDueDiligence,
        Some(expiry.into()),
    ));
}

pub fn add_trusted_cdd_provider(cdd_sp: IdentityId) {
    let signed_id = Origin::signed(AccountId::from(AccountKeyring::Dave));
    assert_ok!(Group::add_member(signed_id, cdd_sp));
}

pub fn fix_nominator_genesis(cdd_sp: IdentityId, did: IdentityId, acc: u64) {
    let controller = account_from(acc);
    let stash = account_from(acc + 1);
    let signed_id = Origin::signed(AccountId::from(AccountKeyring::Dave));
    let now = Utc::now();
    add_nominator_claim(cdd_sp, did, AccountId::from(AccountKeyring::Dave));
    assert_ok!(Staking::nominate(
        Origin::signed(controller),
        vec![account_from(11), account_from(21)]
    ));
    assert_eq!(
        Staking::nominators(stash).unwrap().targets,
        vec![account_from(11), account_from(21)]
    );
}

pub fn create_did_and_add_claim(stash: AccountId) {
    Balances::make_free_balance_be(&account_from(1005), 1_000_000);
    assert_ok!(Identity::cdd_register_did(
        Origin::signed(account_from(1005)),
        stash,
        None,
        vec![]
    ));
}

pub fn create_did_and_add_claim_with_expiry(stash: AccountId, expiry: u64) {
    Balances::make_free_balance_be(&account_from(1005), 1_000_000);
    assert_ok!(Identity::cdd_register_did(
        Origin::signed(account_from(1005)),
        stash,
        Some(expiry.into()),
        vec![]
    ));
}

pub fn make_account(
    id: AccountId,
) -> Result<(<Test as frame_system::Trait>::Origin, IdentityId), &'static str> {
    make_account_with_balance(id, 1_000_000)
}

/// It creates an Account and registers its DID.
pub fn make_account_with_balance(
    id: AccountId,
    balance: <Test as CommonTrait>::Balance,
) -> Result<(<Test as frame_system::Trait>::Origin, IdentityId), &'static str> {
    let signed_id = Origin::signed(id.clone());
    Balances::make_free_balance_be(&id, balance);

    Identity::register_did(signed_id.clone(), vec![]).map_err(|_| "Register DID failed")?;
    let did = Identity::get_identity(&AccountKey::try_from(id.encode())?).unwrap();

    Ok((signed_id, did))
}

pub fn check_cdd(id: AccountId) -> Result<bool, &'static str> {
    let did = Identity::get_identity(&AccountKey::try_from(id.encode())?).unwrap();
    let is_cdd = Identity::fetch_cdd(did, Zero::zero()).is_some();
    Ok(is_cdd)
}

pub fn advance_session() {
    let current_index = Session::current_index();
    start_session(current_index + 1);
}

pub fn start_session(session_index: SessionIndex) {
    for i in Session::current_index()..session_index {
        Staking::on_finalize(System::block_number());
        System::set_block_number((i + 1).into());
        Timestamp::set_timestamp(System::block_number() * 1000);
        Session::on_initialize(System::block_number());
    }

    assert_eq!(Session::current_index(), session_index);
}

pub fn start_era(era_index: EraIndex) {
    start_session((era_index * 3).into());
    assert_eq!(Staking::active_era().unwrap().index, era_index);
}

pub fn current_total_payout_for_duration(duration: u64) -> u128 {
    inflation::compute_total_payout(
        <Test as Trait>::RewardCurve::get(),
        Staking::eras_total_stake(Staking::active_era().unwrap().index),
        Balances::total_issuance().saturating_sub(Balances::block_rewards_reserve_balance()),
        duration,
    )
    .0
}

pub fn reward_all_elected() {
    let rewards = <Test as Trait>::SessionInterface::validators()
        .into_iter()
        .map(|v| (v, 1));
    <Module<Test>>::reward_by_ids(rewards)
}

pub fn validator_controllers() -> Vec<AccountId> {
    Session::validators()
        .into_iter()
        .map(|s| Staking::bonded(&s).expect("no controller for validator"))
        .collect()
}

pub fn on_offence_in_era(
    offenders: &[OffenceDetails<
        AccountId,
        pallet_session::historical::IdentificationTuple<Test>,
    >],
    slash_fraction: &[Perbill],
    era: EraIndex,
) {
    let bonded_eras = crate::BondedEras::get();
    for &(bonded_era, start_session) in bonded_eras.iter() {
        if bonded_era == era {
            Staking::on_offence(offenders, slash_fraction, start_session);
            return;
        } else if bonded_era > era {
            break;
        }
    }

    if Staking::active_era().unwrap().index == era {
        Staking::on_offence(
            offenders,
            slash_fraction,
            Staking::eras_start_session_index(era).unwrap(),
        );
    } else {
        panic!("cannot slash in era {}", era);
    }
}

pub fn on_offence_now(
    offenders: &[OffenceDetails<
        AccountId,
        pallet_session::historical::IdentificationTuple<Test>,
    >],
    slash_fraction: &[Perbill],
) {
    let now = Staking::active_era().unwrap().index;
    on_offence_in_era(offenders, slash_fraction, now)
}

/// Make all validator and nominator request their payment
pub fn make_all_reward_payment(era: EraIndex) {
    let validators_with_reward = ErasRewardPoints::<Test>::get(era)
        .individual
        .keys()
        .cloned()
        .collect::<Vec<_>>();

    // reward nominators
    let mut nominator_controllers = HashMap::new();
    for validator in Staking::eras_reward_points(era).individual.keys() {
        let validator_exposure = Staking::eras_stakers_clipped(era, validator);
        for (nom_index, nom) in validator_exposure.others.iter().enumerate() {
            if let Some(nom_ctrl) = Staking::bonded(nom.who) {
                nominator_controllers
                    .entry(nom_ctrl)
                    .or_insert(vec![])
                    .push((validator.clone(), nom_index as u32));
            }
        }
    }
    for (nominator_controller, validators_with_nom_index) in nominator_controllers {
        assert_ok!(Staking::payout_nominator(
            Origin::signed(nominator_controller),
            era,
            validators_with_nom_index,
        ));
    }

    // reward validators
    for validator_controller in validators_with_reward.iter().filter_map(Staking::bonded) {
        assert_ok!(Staking::payout_validator(
            Origin::signed(validator_controller),
            era
        ));
    }
}

pub fn fix_nominator_genesis_problem(value: u128) {
    let nominator_controller_account = 100;
    let nominator_stash_account = 101;
    let (_nominator_signed, nominator_did) =
        make_account_with_balance(account_from(nominator_stash_account), value).unwrap();

    let service_provider_account = AccountId::from(AccountKeyring::Dave);
    let (service_provider_signed, service_provider_did) =
        make_account(service_provider_account.clone()).unwrap();
    add_trusted_cdd_provider(service_provider_did);

    fix_nominator_genesis(
        service_provider_did,
        nominator_did,
        nominator_controller_account,
    );
}

pub fn add_claim_for_nominator(
    stash: AccountId,
    service_provider_account: AccountId,
    balance: u128,
) {
    let (nominator_signed, nominator_did) = make_account_with_balance(stash, 1_000_000).unwrap();

    let (service_provider_signed, service_provider_did) =
        make_account(service_provider_account.clone()).unwrap();
    add_trusted_cdd_provider(service_provider_did);

    let now = Utc::now();
    add_nominator_claim(
        service_provider_did,
        nominator_did,
        service_provider_account,
    );
}
