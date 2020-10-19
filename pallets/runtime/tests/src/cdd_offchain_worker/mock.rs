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

use codec::Encode;
use frame_support::traits::KeyOwnerProofSystem;
use frame_support::{
    assert_ok,
    dispatch::DispatchResult,
    impl_outer_dispatch, impl_outer_origin, parameter_types,
    traits::{Currency, FindAuthor, Get},
    weights::{DispatchInfo, RuntimeDbWeight, Weight},
    StorageDoubleMap, StorageMap,
};
use frame_system::offchain::*;
use pallet_cdd_offchain_worker::crypto;
use pallet_corporate_actions as corporate_actions;
use pallet_group as group;
use pallet_identity::{self as identity};
use pallet_portfolio as portfolio;
use pallet_protocol_fee as protocol_fee;
use pallet_staking::{EraIndex, Exposure, ExposureOf, StakerStatus};
use polymesh_common_utilities::traits::{
    asset::AssetSubTrait,
    balances::{AccountData, CheckCdd},
    group::{GroupTrait, InactiveMember},
    identity::Trait as IdentityTrait,
    multisig::MultiSigSubTrait,
    portfolio::PortfolioSubTrait,
    transaction_payment::{CddAndFeeDetails, ChargeTxFee},
    CommonTrait, PermissionChecker,
};
use polymesh_primitives::{
    Authorization, AuthorizationData, CddId, Claim, IdentityId, InvestorUid, Permissions,
    PortfolioId, ScopeId, Signatory, Ticker,
};
use sp_core::{
    crypto::{key_types, Pair as PairTrait},
    sr25519::{Pair, Signature},
    H256,
};
use sp_runtime::curve::PiecewiseLinear;
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction};
use sp_runtime::{
    testing::{Header, TestXt, UintAuthorityId},
    traits::{
        Convert, ConvertInto, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, OpaqueKeys,
        SaturatedConversion, Verify,
    },
    KeyTypeId, Perbill,
};
use sp_staking::SessionIndex;
use sp_std::convert::From;
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashSet},
};
use test_client::AccountKeyring;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type BlockNumber = u64;
pub type Balance = u128;
type OffChainSignature = Signature;
pub type Moment = <Test as pallet_timestamp::Trait>::Moment;

impl_outer_origin! {
    pub enum Origin for Test  where system = frame_system {}
}

impl_outer_dispatch! {
    pub enum Call for Test where origin: Origin {
        identity::Identity,
        pallet_staking::Staking,
        pallet_babe::Babe,
        pallet_cdd_offchain_worker::CddOffchainWorker,
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
    static ELECTION_LOOKAHEAD: RefCell<BlockNumber> = RefCell::new(0);
    static MAX_ITERATIONS: RefCell<u32> = RefCell::new(0);
    static EXTRINSIC_BASE_WEIGHT: RefCell<u64> = RefCell::new(0);
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
            let value = d.0[validator_index].clone();
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Test;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
    pub const MaximumExtrinsicWeight: u64 = 2800;
    pub const BlockExecutionWeight: u64 = 10;
    pub ExtrinsicBaseWeight: u64 = EXTRINSIC_BASE_WEIGHT.with(|v| *v.borrow());
    pub const DbWeight: RuntimeDbWeight = RuntimeDbWeight {
        read: 10,
        write: 100,
    };
}

pub struct TestAppCryptoId;

impl AppCrypto<<Signature as Verify>::Signer, Signature> for TestAppCryptoId {
    type RuntimeAppPublic = crypto::SignerId;
    type GenericSignature = sp_core::sr25519::Signature;
    type GenericPublic = sp_core::sr25519::Public;
}

impl frame_system::Trait for Test {
    /// The basic call filter to use in dispatchable.
    type BaseCallFilter = ();
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The aggregated dispatch type that is available for extrinsics.
    type Call = Call;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = IdentityLookup<Self::AccountId>;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = u64;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The type for hashing blocks and tries.
    type Hash = H256;
    /// The hashing algorithm used.
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    /// The header type.
    type Header = Header;
    /// The ubiquitous event type.
    type Event = ();
    /// The ubiquitous origin type.
    type Origin = Origin;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// Maximum weight of each block.
    type MaximumBlockWeight = MaximumBlockWeight;
    /// Maximum size of all encoded transactions (in bytes) that are allowed in one block.
    type MaximumBlockLength = MaximumBlockLength;
    /// Portion of the block weight that is available to all normal transactions.
    type AvailableBlockRatio = AvailableBlockRatio;
    /// Version of the runtime.
    type Version = ();
    /// Converts a module to the index of the module in `construct_runtime!`.
    ///
    /// This type is being generated by `construct_runtime!`.
    type PalletInfo = ();
    /// What to do if a new account is created.
    type OnNewAccount = ();
    /// What to do if an account is fully reaped from the system.
    type OnKilledAccount = ();
    /// The data to be stored in an account.
    type AccountData = AccountData<<Test as CommonTrait>::Balance>;
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = DbWeight;
    /// The weight of the overhead invoked on the block import process, independent of the
    /// extrinsics included in that block.
    type BlockExecutionWeight = BlockExecutionWeight;
    /// The base weight of any extrinsic processed by the runtime, independent of the
    /// logic of that extrinsic. (Signature verification, nonce increment, fee, etc...)
    type ExtrinsicBaseWeight = ExtrinsicBaseWeight;
    /// The maximum weight that a single extrinsic of `Normal` dispatch class can have,
    /// independent of the logic of that extrinsics. (Roughly max block weight - average on
    /// initialize cost).
    type MaximumExtrinsicWeight = MaximumExtrinsicWeight;
    type SystemWeightInfo = ();
}

impl CommonTrait for Test {
    type Balance = Balance;
    type AssetSubTraitTarget = Test;
    type BlockRewardsReserve = pallet_balances::Module<Test>;
}

parameter_types! {
    pub const TransactionBaseFee: Balance = 0;
    pub const TransactionByteFee: Balance = 0;
    pub const ExistentialDeposit: Balance = 0;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Trait for Test {
    type DustRemoval = ();
    type Event = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = frame_system::Module<Test>;
    type Identity = identity::Module<Test>;
    type CddChecker = Test;
    type WeightInfo = ();
    type MaxLocks = MaxLocks;
}

impl group::Trait<group::Instance2> for Test {
    type Event = ();
    type LimitOrigin = frame_system::EnsureRoot<AccountId>;
    type AddOrigin = frame_system::EnsureRoot<AccountId>;
    type RemoveOrigin = frame_system::EnsureRoot<AccountId>;
    type SwapOrigin = frame_system::EnsureRoot<AccountId>;
    type ResetOrigin = frame_system::EnsureRoot<AccountId>;
    type MembershipInitialized = identity::Module<Test>;
    type MembershipChanged = identity::Module<Test>;
}

impl corporate_actions::Trait for Test {
    type Event = ();
    type WeightInfo = polymesh_weights::pallet_corporate_actions::WeightInfo;
}

impl protocol_fee::Trait for Test {
    type Event = ();
    type Currency = Balances;
    type OnProtocolFeePayment = ();
}

impl PermissionChecker for Test {
    type Call = Call;
    type Checker = Identity;
}

impl IdentityTrait for Test {
    type Event = ();
    type Proposal = Call;
    type MultiSig = Test;
    type Portfolio = portfolio::Module<Test>;
    type CddServiceProviders = group::Module<Test, group::Instance2>;
    type Balances = pallet_balances::Module<Test>;
    type ChargeTxFeeTarget = Test;
    type CddHandler = Test;
    type Public = <Signature as Verify>::Signer;
    type OffChainSignature = OffChainSignature;
    type ProtocolFee = protocol_fee::Module<Test>;
    type GCVotingMajorityOrigin = frame_system::EnsureRoot<AccountId>;
    type CorporateAction = CorporateActions;
}

impl CddAndFeeDetails<AccountId, Call> for Test {
    fn get_valid_payer(_: &Call, _: &AccountId) -> Result<Option<AccountId>, InvalidTransaction> {
        Ok(None)
    }
    fn clear_context() {}
    fn set_payer_context(_: Option<AccountId>) {}
    fn get_payer_from_context() -> Option<AccountId> {
        None
    }
    fn set_current_identity(_: &IdentityId) {}
}

impl ChargeTxFee for Test {
    fn charge_fee(_len: u32, _info: DispatchInfo) -> TransactionValidity {
        Ok(ValidTransaction::default())
    }
}

impl GroupTrait<Moment> for Test {
    fn get_members() -> Vec<IdentityId> {
        return CddServiceProvider::active_members();
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

    fn is_member_expired(_member: &InactiveMember<Moment>, _now: Moment) -> bool {
        false
    }
}

impl AssetSubTrait for Test {
    fn accept_ticker_transfer(_: IdentityId, _: u64) -> DispatchResult {
        unimplemented!()
    }

    fn accept_primary_issuance_agent_transfer(_: IdentityId, _: u64) -> DispatchResult {
        unimplemented!()
    }

    fn accept_asset_ownership_transfer(_: IdentityId, _: u64) -> DispatchResult {
        unimplemented!()
    }

    fn update_balance_of_scope_id(
        _of: ScopeId,
        _whom: IdentityId,
        _ticker: Ticker,
    ) -> DispatchResult {
        unimplemented!()
    }
}

impl MultiSigSubTrait<AccountId> for Test {
    fn accept_multisig_signer(_: Signatory<AccountId>, _: u64) -> DispatchResult {
        unimplemented!()
    }
    fn get_key_signers(_multisig: &AccountId) -> Vec<AccountId> {
        unimplemented!()
    }
    fn is_multisig(_account: &AccountId) -> bool {
        unimplemented!()
    }
    fn is_signer(_key: &AccountId) -> bool {
        false
    }
}

impl PortfolioSubTrait<Balance> for Test {
    fn accept_portfolio_custody(_: IdentityId, _: u64) -> DispatchResult {
        unimplemented!()
    }
    fn ensure_portfolio_custody(_portfolio: PortfolioId, _custodian: IdentityId) -> DispatchResult {
        unimplemented!()
    }

    fn lock_tokens(
        _portfolio: &PortfolioId,
        _ticker: &Ticker,
        _amount: &Balance,
    ) -> DispatchResult {
        unimplemented!()
    }

    fn unlock_tokens(
        _portfolio: &PortfolioId,
        _ticker: &Ticker,
        _amount: &Balance,
    ) -> DispatchResult {
        unimplemented!()
    }
}

impl CheckCdd<AccountId> for Test {
    fn check_key_cdd(_key: &AccountId) -> bool {
        true
    }
    fn get_key_cdd_did(_key: &AccountId) -> Option<IdentityId> {
        None
    }
}

parameter_types! {
    pub const Period: BlockNumber = 1;
    pub const Offset: BlockNumber = 0;
    pub const UncleGenerations: u64 = 0;
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(25);
}

thread_local! {
    pub static FORCE_SESSION_END: RefCell<bool> = RefCell::new(false);
    pub static SESSION_LENGTH: RefCell<u64> = RefCell::new(2);
}

pub struct TestShouldEndSession;
impl pallet_session::ShouldEndSession<BlockNumber> for TestShouldEndSession {
    fn should_end_session(now: BlockNumber) -> bool {
        let l = SESSION_LENGTH.with(|l| *l.borrow());
        now % l == 0
            || FORCE_SESSION_END.with(|l| {
                let r = *l.borrow();
                *l.borrow_mut() = false;
                r
            })
    }
}

pub struct TestSessionManager;
impl pallet_session::SessionManager<AccountId> for TestSessionManager {
    fn end_session(_: SessionIndex) {}
    fn start_session(_: SessionIndex) {}
    fn new_session(_: SessionIndex) -> Option<Vec<AccountId>> {
        None
    }
}

impl pallet_session::Trait for Test {
    type Event = ();
    type ValidatorId = AccountId;
    type ValidatorIdOf = ConvertInto;
    type ShouldEndSession = TestShouldEndSession;
    type NextSessionRotation = ();
    type SessionManager = TestSessionManager;
    type SessionHandler = TestSessionHandler;
    type Keys = UintAuthorityId;
    type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
    type WeightInfo = ();
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
    type WeightInfo = ();
}

impl pallet_babe::Trait for Test {
    type WeightInfo = ();
    type EpochDuration = EpochDuration;
    type ExpectedBlockTime = ExpectedBlockTime;
    type EpochChangeTrigger = pallet_babe::ExternalTrigger;

    type KeyOwnerProofSystem = ();
    type KeyOwnerProof = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        pallet_babe::AuthorityId,
    )>>::Proof;
    type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        pallet_babe::AuthorityId,
    )>>::IdentificationTuple;
    type HandleEquivocation = pallet_babe::EquivocationHandler<Self::KeyOwnerIdentification, ()>;
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
    pub const UnsignedPriority: u64 = 1 << 20;
    pub const MinSolutionScoreBump: Perbill = Perbill::zero();
}

pub struct ElectionLookahead;
impl Get<BlockNumber> for ElectionLookahead {
    fn get() -> BlockNumber {
        ELECTION_LOOKAHEAD.with(|v| *v.borrow())
    }
}

pub struct MaxIterations;
impl Get<u32> for MaxIterations {
    fn get() -> u32 {
        MAX_ITERATIONS.with(|v| *v.borrow())
    }
}

impl pallet_staking::Trait for Test {
    type Currency = Balances;
    type UnixTime = Timestamp;
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
    type NextNewSession = Session;
    type ElectionLookahead = ElectionLookahead;
    type Call = Call;
    type MaxIterations = MaxIterations;
    type MinSolutionScoreBump = MinSolutionScoreBump;
    type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
    type UnsignedPriority = UnsignedPriority;
    type RequiredAddOrigin = frame_system::EnsureRoot<AccountId>;
    type RequiredRemoveOrigin = frame_system::EnsureRoot<AccountId>;
    type RequiredComplianceOrigin = frame_system::EnsureRoot<AccountId>;
    type RequiredCommissionOrigin = frame_system::EnsureRoot<AccountId>;
    type RequiredChangeHistoryDepthOrigin = frame_system::EnsureRoot<AccountId>;
    type WeightInfo = ();
}

impl portfolio::Trait for Test {
    type Event = ();
}

pub type Extrinsic = TestXt<Call, ()>;

impl pallet_cdd_offchain_worker::Trait for Test {
    type AuthorityId = TestAppCryptoId;
    type Event = ();
    type Call = Call;
    type CoolingInterval = CoolingInterval;
    type BufferInterval = BufferInterval;
    type UnsignedPriority = UnsignedPriority;
    type StakingInterface = Staking;
}

impl SigningTypes for Test {
    type Public = <Signature as Verify>::Signer;
    type Signature = Signature;
}

impl<LocalCall> SendTransactionTypes<LocalCall> for Test
where
    Call: From<LocalCall>,
{
    type OverarchingCall = Call;
    type Extrinsic = Extrinsic;
}

impl<LocalCall> CreateSignedTransaction<LocalCall> for Test
where
    Call: From<LocalCall>,
{
    fn create_transaction<C: AppCrypto<Self::Public, Self::Signature>>(
        call: Call,
        _public: <Signature as Verify>::Signer,
        _account: AccountId,
        nonce: u64,
    ) -> Option<(Call, <Extrinsic as ExtrinsicT>::SignaturePayload)> {
        Some((call, (nonce, ())))
    }
}

pub struct ExtBuilder {
    session_per_era: SessionIndex,
    bonding_duration: EraIndex,
    nominate: bool,
    expected_block_time: u64,
    epoch_duration: u64,
    cooling_interval: u64,
    buffer_interval: u64,
    cdd_providers: Vec<AccountId>,
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
            cdd_providers: vec![],
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
    /// It sets `providers` as CDD providers.
    pub fn cdd_providers(mut self, providers: Vec<AccountId>) -> Self {
        self.cdd_providers = providers;
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

        let account_key_ring: BTreeMap<u64, AccountId> = [
            1, 2, 3, 4, 10, 11, 20, 21, 30, 31, 40, 41, 100, 101, 999, 1005,
        ]
        .iter()
        .map(|id| (*id, account_from(*id)))
        .collect();

        let _ = pallet_balances::GenesisConfig::<Test> {
            balances: vec![
                (AccountKeyring::Alice.public(), 10 * balance_factor),
                (AccountKeyring::Bob.public(), 20 * balance_factor),
                (AccountKeyring::Charlie.public(), 300 * balance_factor),
                (AccountKeyring::Dave.public(), 400 * balance_factor),
                (
                    account_key_ring.get(&11).unwrap().clone(),
                    1500 * balance_factor,
                ),
                (
                    account_key_ring.get(&21).unwrap().clone(),
                    1000 * balance_factor,
                ),
                (
                    account_key_ring.get(&20).unwrap().clone(),
                    1000 * balance_factor,
                ),
                (
                    account_key_ring.get(&10).unwrap().clone(),
                    1000 * balance_factor,
                ),
            ],
        }
        .assimilate_storage(&mut storage);

        let mut system_ids = self
            .cdd_providers
            .clone()
            .into_iter()
            .enumerate()
            .map(|(index, prov)| {
                (
                    prov,
                    IdentityId::from(1),
                    IdentityId::from((100 + index) as u128),
                    InvestorUid::from(b"abc".as_ref()),
                    None,
                )
            })
            .collect::<Vec<_>>();
        system_ids = system_ids
            .into_iter()
            .chain(vec![
                (
                    account_key_ring.get(&11).unwrap().clone(),
                    IdentityId::from(1),
                    IdentityId::from(1),
                    InvestorUid::from(b"uid1".as_ref()),
                    None,
                ),
                (
                    account_key_ring.get(&21).unwrap().clone(),
                    IdentityId::from(1),
                    IdentityId::from(2),
                    InvestorUid::from(b"uid2".as_ref()),
                    None,
                ),
            ])
            .collect::<Vec<_>>();

        let _ = identity::GenesisConfig::<Test> {
            identities: system_ids.clone(),
            secondary_keys: vec![
                (
                    account_key_ring.get(&10).unwrap().clone(),
                    IdentityId::from(1),
                ),
                (
                    account_key_ring.get(&20).unwrap().clone(),
                    IdentityId::from(2),
                ),
            ],
            ..Default::default()
        }
        .assimilate_storage(&mut storage);

        let mut members = system_ids
            .into_iter()
            .filter(|(acc, _, _, _, _)| {
                (self.cdd_providers.iter().find(|key| *key == acc)).is_some()
            })
            .map(|(_, _, id, _, _)| id)
            .collect::<Vec<_>>();

        members.push(IdentityId::from(1));

        let _ = group::GenesisConfig::<Test, group::Instance2> {
            active_members_limit: u32::MAX,
            active_members: members,
            phantom: Default::default(),
        }
        .assimilate_storage(&mut storage);

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
                    IdentityId::from(11),
                    account_key_ring.get(&11).unwrap().clone(),
                    account_key_ring.get(&10).unwrap().clone(),
                    balance_factor * 1000,
                    StakerStatus::<AccountId>::Validator,
                ),
                // nominator
                (
                    IdentityId::from(21),
                    account_key_ring.get(&21).unwrap().clone(),
                    account_key_ring.get(&20).unwrap().clone(),
                    balance_factor * 500,
                    StakerStatus::<AccountId>::Nominator(nominated),
                ),
            ],
            validator_count: 1,
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

pub type CddOffchainWorker = pallet_cdd_offchain_worker::Module<Test>;
pub type System = frame_system::Module<Test>;
pub type Session = pallet_session::Module<Test>;
pub type Timestamp = pallet_timestamp::Module<Test>;
pub type Identity = identity::Module<Test>;
pub type Balances = pallet_balances::Module<Test>;
pub type CddServiceProvider = group::Module<Test, group::Instance2>;
pub type Staking = pallet_staking::Module<Test>;
pub type Babe = pallet_babe::Module<Test>;
pub type CorporateActions = corporate_actions::Module<Test>;

pub fn account_from(id: u64) -> AccountId {
    let mut enc_id_vec = id.encode();
    enc_id_vec.resize_with(32, Default::default);

    let mut enc_id = [0u8; 32];
    enc_id.copy_from_slice(enc_id_vec.as_slice());

    Pair::from_seed(&enc_id).public()
}

pub fn make_account(
    id: AccountId,
    uid: InvestorUid,
    expiry: Option<Moment>,
) -> Result<(<Test as frame_system::Trait>::Origin, IdentityId), &'static str> {
    make_account_with_balance(id, uid, expiry, 1_000_000)
}

/// It creates an Account and registers its DID and its InvestorUid.
pub fn make_account_with_balance(
    id: AccountId,
    uid: InvestorUid,
    expiry: Option<Moment>,
    balance: <Test as CommonTrait>::Balance,
) -> Result<(<Test as frame_system::Trait>::Origin, IdentityId), &'static str> {
    let signed_id = Origin::signed(id.clone());
    Balances::make_free_balance_be(&id, balance);

    // If we have CDD providers, first of them executes the registration.
    let cdd_providers = CddServiceProvider::get_members();
    let did = match cdd_providers.into_iter().nth(0) {
        Some(cdd_provider) => {
            let cdd_acc = Identity::did_records(&cdd_provider).primary_key;
            let _ = Identity::cdd_register_did(Origin::signed(cdd_acc), id, vec![])
                .map_err(|_| "CDD register DID failed")?;

            // Add CDD Claim
            let did = Identity::get_identity(&id).unwrap();
            let cdd_claim = Claim::CustomerDueDiligence(CddId::new(did, uid));
            Identity::add_claim(Origin::signed(cdd_acc), did, cdd_claim, expiry)
                .map_err(|_| "CDD provider cannot add the CDD claim")?;
            did
        }
        _ => {
            let _ = Identity::register_did(signed_id.clone(), uid, vec![])
                .map_err(|_| "Register DID failed")?;
            Identity::get_identity(&id).unwrap()
        }
    };

    Ok((signed_id, did))
}

pub fn add_secondary_key(stash_key: AccountId, to_secondary_key: AccountId) {
    if !get_identity(to_secondary_key) {
        let _did = Identity::get_identity(&stash_key).unwrap();
        assert!(
            Identity::add_authorization(
                Origin::signed(stash_key),
                Signatory::Account(to_secondary_key),
                AuthorizationData::JoinIdentity(Permissions::default()),
                None
            )
            .is_ok(),
            "Error in providing the authorization"
        );
        let auth_id = get_last_auth_id(&Signatory::Account(to_secondary_key));
        assert_ok!(Identity::join_identity_as_key(
            Origin::signed(to_secondary_key),
            auth_id
        ));
    }
}

pub fn get_last_auth(signatory: &Signatory<AccountId>) -> Authorization<AccountId, u64> {
    <identity::Authorizations<Test>>::iter_prefix_values(signatory)
        .into_iter()
        .max_by_key(|x| x.auth_id)
        .expect("there are no authorizations")
}

pub fn get_last_auth_id(signatory: &Signatory<AccountId>) -> u64 {
    get_last_auth(signatory).auth_id
}

pub fn get_identity(key: AccountId) -> bool {
    <identity::KeyToIdentityIds<Test>>::contains_key(&key)
}
