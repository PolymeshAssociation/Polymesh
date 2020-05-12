// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2020 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use crate::*;

use chrono::prelude::Utc;
use codec::Encode;
use frame_support::{
    assert_ok,
    dispatch::DispatchResult,
    impl_outer_dispatch, impl_outer_origin, ord_parameter_types, parameter_types,
    traits::{Currency, FindAuthor, Get},
    weights::{DispatchInfo, Weight},
};
use frame_system::EnsureSignedBy;
use pallet_group as group;
use pallet_identity::{self as identity};
use pallet_protocol_fee as protocol_fee;
use pallet_staking::{EraIndex, Exposure, ExposureOf, StakerStatus, StashOf};
use polymesh_common_utilities::traits::{
    asset::AcceptTransfer,
    balances::{AccountData, CheckCdd},
    group::{GroupTrait, InactiveMember},
    identity::Trait as IdentityTrait,
    multisig::AddSignerMultiSig,
    CommonTrait,
};
use primitives::{AccountKey, IdentityId, Signatory};
use sp_core::{
    crypto::{key_types, Pair as PairTrait},
    offchain::{testing, OffchainExt, TransactionPoolExt},
    sr25519::Pair,
    testing::KeyStore,
    traits::KeystoreExt,
    H256,
};
use sp_runtime::curve::PiecewiseLinear;
use sp_runtime::{
    testing::{sr25519::Public, Header, TestXt, UintAuthorityId},
    traits::{
        Convert, Extrinsic as ExtrinsicsT, IdentityLookup, OpaqueKeys, SaturatedConversion, Verify,
        Zero,
    },
    AnySignature, KeyTypeId, Perbill,
};

use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction};
use sp_staking::SessionIndex;
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashSet},
};
use test_client::AccountKeyring;

pub type AccountId = <AnySignature as Verify>::Signer;
pub type BlockNumber = u64;
pub type Balance = u128;
type OffChainSignature = AnySignature;
type Moment = <Test as pallet_timestamp::Trait>::Moment;

impl_outer_origin! {
    pub enum Origin for Test  where system = frame_system {}
}

impl_outer_dispatch! {
    pub enum Call for Test where origin: Origin {
        identity::Identity,
        pallet_staking::Staking,
        cddoffchainworker::CddOffchainWorker,
    }
}

// Simple structure that exposes how u64 currency can be represented as... u64.
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
    static BONDING_DURATION: RefCell<EraIndex> = RefCell::new(0);
    static SESSION_PER_ERA: RefCell<SessionIndex> = RefCell::new(0);
    static EPOCH_DURATION: RefCell<u64> = RefCell::new(0);
    static EXPECTED_BLOCK_TIME: RefCell<u64> = RefCell::new(0);
    static COOLING_INTERVAL: RefCell<u64> = RefCell::new(0);
    static BUFFER_INTERVAL: RefCell<u64> = RefCell::new(0);
}

pub struct BondingDuration;
impl Get<EraIndex> for BondingDuration {
    fn get() -> EraIndex {
        BONDING_DURATION.with(|v| *v.borrow())
    }
}

pub struct SessionsPerEra;
impl Get<SessionIndex> for SessionsPerEra {
    fn get() -> SessionIndex {
        SESSION_PER_ERA.with(|v| *v.borrow())
    }
}

pub struct EpochDuration;
impl Get<u64> for EpochDuration {
    fn get() -> u64 {
        EPOCH_DURATION.with(|v| *v.borrow())
    }
}

pub struct ExpectedBlockTime;
impl Get<u64> for ExpectedBlockTime {
    fn get() -> u64 {
        EXPECTED_BLOCK_TIME.with(|v| *v.borrow())
    }
}

pub struct CoolingInterval;
impl Get<u64> for CoolingInterval {
    fn get() -> u64 {
        COOLING_INTERVAL.with(|v| *v.borrow())
    }
}

pub struct BufferInterval;
impl Get<u64> for BufferInterval {
    fn get() -> u64 {
        BUFFER_INTERVAL.with(|v| *v.borrow())
    }
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
    type BlockRewardsReserve = pallet_balances::Module<Test>;
}

parameter_types! {
    pub const TransactionBaseFee: Balance = 0;
    pub const TransactionByteFee: Balance = 0;
    pub const ExistentialDeposit: Balance = 0;
}

impl pallet_balances::Trait for Test {
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
    type AddSignerMultiSigTarget = Test;
    type CddServiceProviders = group::Module<Test, group::Instance2>;
    type Balances = pallet_balances::Module<Test>;
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

impl AddSignerMultiSig for Test {
    fn accept_multisig_signer(_: Signatory, _: u64) -> DispatchResult {
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
    type ValidatorIdOf = StashOf<Test>;
    type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
    type SessionManager = Staking;
    type SessionHandler = TestSessionHandler;
    type Keys = UintAuthorityId;
    type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
}

impl pallet_session::historical::Trait for Test {
    type FullIdentification = Exposure<AccountId, Balance>;
    type FullIdentificationOf = ExposureOf<Test>;
}
impl pallet_authorship::Trait for Test {
    type FindAuthor = Author11;
    type UncleGenerations = UncleGenerations;
    type FilterUncle = ();
    type EventHandler = pallet_staking::Module<Test>;
}

parameter_types! {
    pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Trait for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
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
    pub const RewardCurve: &'static PiecewiseLinear<'static> = &I_NPOS;
    pub const MaxNominatorRewardedPerValidator: u32 = 64;
    pub const SlashDeferDuration: EraIndex = 0;
}

ord_parameter_types! {
    pub const OneThousand: Public = account_from(1000);
    pub const TwoThousand: Public = account_from(2000);
    pub const ThreeThousand: Public = account_from(3000);
    pub const FourThousand: Public = account_from(4000);
    pub const FiveThousand: Public = account_from(5000);
}

impl pallet_staking::Trait for Test {
    type Currency = pallet_balances::Module<Self>;
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

pub type Extrinsic = TestXt<Call, ()>;
pub type SubmitTransaction =
    frame_system::offchain::TransactionSubmitter<crypto::SignerId, Call, Extrinsic>;

impl Trait for Test {
    type SignerId = UintAuthorityId;
    type Event = ();
    type Call = Call;
    type SubmitUnsignedTransaction = SubmitTransaction;
    type CoolingInterval = CoolingInterval;
    type BufferInterval = BufferInterval;
}

pub struct ExtBuilder {
    session_per_era: SessionIndex,
    bonding_duration: EraIndex,
    nominate: bool,
    expected_block_time: u64,
    epoch_duration: u64,
    cooling_interval: u64,
    buffer_interval: u64,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            session_per_era: 3,
            bonding_duration: 3,
            nominate: true,
            expected_block_time: 1,
            epoch_duration: 10,
            cooling_interval: 3,
            buffer_interval: 0,
        }
    }
}

impl ExtBuilder {
    pub fn session_per_era(mut self, session_per_era: SessionIndex) -> Self {
        self.session_per_era = session_per_era;
        self
    }
    pub fn bonding_duration(mut self, bonding_duration: EraIndex) -> Self {
        self.bonding_duration = bonding_duration;
        self
    }
    pub fn nominate(mut self, nominate: bool) -> Self {
        self.nominate = nominate;
        self
    }
    pub fn expected_block_time(mut self, expected_block_time: u64) -> Self {
        self.expected_block_time = expected_block_time;
        self
    }
    pub fn epoch_duration(mut self, epoch_duration: u64) -> Self {
        self.epoch_duration = epoch_duration;
        self
    }
    pub fn cooling_interval(mut self, cooling_interval: u64) -> Self {
        self.cooling_interval = cooling_interval;
        self
    }
    pub fn buffer_interval(mut self, buffer_interval: u64) -> Self {
        self.buffer_interval = buffer_interval;
        self
    }
    pub fn set_associated_consts(&self) {
        BONDING_DURATION.with(|v| *v.borrow_mut() = self.bonding_duration);
        SESSION_PER_ERA.with(|v| *v.borrow_mut() = self.session_per_era);
        EPOCH_DURATION.with(|v| *v.borrow_mut() = self.epoch_duration);
        EXPECTED_BLOCK_TIME.with(|v| *v.borrow_mut() = self.expected_block_time);
        COOLING_INTERVAL.with(|v| *v.borrow_mut() = self.cooling_interval);
        BUFFER_INTERVAL.with(|v| *v.borrow_mut() = self.buffer_interval);
    }
    pub fn build(self) -> sp_io::TestExternalities {
        self.set_associated_consts();
        let mut storage = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        let balance_factor = 1;

        let num_validators = 2; // validator_count
        let validators = (0..num_validators)
            .map(|x| ((x + 1) * 10 + 1) as u64)
            .collect::<Vec<_>>();

        let account_key_ring: BTreeMap<u64, Public> = [
            1, 2, 3, 4, 10, 11, 20, 21, 30, 31, 40, 41, 100, 101, 999, 1005,
        ]
        .iter()
        .map(|id| (*id, account_from(*id)))
        .collect();

        pallet_balances::GenesisConfig::<Test> {
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

        let stake_21 = 1000;
        let stake_31 = 1;
        let status_41 = StakerStatus::<AccountId>::Idle;
        let nominated = if self.nominate {
            vec![
                account_key_ring.get(&11).unwrap().clone(),
                account_key_ring.get(&21).unwrap().clone(),
            ]
        } else {
            vec![]
        };
        let _ = pallet_staking::GenesisConfig::<Test> {
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
            validator_count: 2,
            minimum_validator_count: 0,
            invulnerables: vec![],
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

pub type CddOffchainWorker = Module<Test>;
pub type System = frame_system::Module<Test>;
pub type Session = pallet_session::Module<Test>;
pub type Timestamp = pallet_timestamp::Module<Test>;
pub type Identity = identity::Module<Test>;
pub type Balances = pallet_balances::Module<Test>;
pub type Group = group::Module<Test, group::Instance2>;
pub type Staking = pallet_staking::Module<Test>;

pub fn account_from(id: u64) -> AccountId {
    let mut enc_id_vec = id.encode();
    enc_id_vec.resize_with(32, Default::default);

    let mut enc_id = [0u8; 32];
    enc_id.copy_from_slice(enc_id_vec.as_slice());

    Pair::from_seed(&enc_id).public()
}

pub fn create_did_and_add_claim(stash: AccountId, expiry: u64) {
    Balances::make_free_balance_be(&account_from(1005), 1_000_000);
    assert_ok!(Identity::cdd_register_did(
        Origin::signed(account_from(1005)),
        stash,
        Some(expiry.saturated_into::<Moment>()),
        vec![]
    ));
}
